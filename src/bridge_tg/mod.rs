use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use lazy_static::lazy_static;
use proc_qq::re_exports::image;
use teleser::re_exports::async_trait::async_trait;
use teleser::re_exports::grammers_client::types::{Chat, Media, Message};
use teleser::re_exports::grammers_client::{Client, InitParams, InputMessage};
use teleser::re_exports::grammers_session::PackedChat;
use teleser::re_exports::grammers_tl_types::enums::MessageEntity;
use teleser::{Auth, ClientBuilder, FileSessionStore, NewMessageProcess, Process, StaticBotToken};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tracing::{debug, error, warn};

use crate::bridge;
use crate::bridge::MessageContent::Plain;
use crate::bridge::{BridgeClient, Image, MessageContent};
use crate::config::{BridgeConfig, Config};

pub async fn start(config: Arc<Config>, bridge: Arc<bridge::BridgeClient>) {
    // 还原pack
    let folder = format!(
        "tg.pack.{}",
        config.telegram_config.botToken.split(":").next().unwrap()
    );
    if !Path::new(folder.as_str()).exists() {
        tokio::fs::create_dir(folder.as_str()).await.unwrap();
    }
    let mut lock = PACK_MAP.lock().await;
    let mut rd = tokio::fs::read_dir(folder.as_str()).await.unwrap();
    while let Some(file) = rd.next_entry().await.unwrap() {
        let id = file.file_name().to_str().unwrap().parse::<i64>().unwrap();
        let data = tokio::fs::read(file.path()).await.unwrap();
        match PackedChat::from_bytes(&data) {
            Ok(chat) => {
                lock.insert(id, chat);
            }
            Err(_) => {}
        }
    }
    drop(lock);
    // 初始化
    tracing::info!("[TG] 初始化TG桥");
    let module = teleser::Module {
        id: "tg_new_message".to_owned(),
        name: "tg_new_message".to_owned(),
        handlers: vec![teleser::Handler {
            id: "tg_new_message".to_owned(),
            process: Process::NewMessageProcess(Box::new(TgNewMessage {
                config: config.clone(),
                bridge: bridge.clone(),
                pack_folder: folder.clone(),
            })),
        }],
    };
    let client = ClientBuilder::new()
        .with_api_id(config.telegram_config.apiId.clone())
        .with_api_hash(config.telegram_config.apiHash.clone())
        .with_session_store(Box::new(FileSessionStore {
            path: "telegram.session".to_string(),
        }))
        .with_auth(Auth::AuthWithBotToken(Box::new(StaticBotToken {
            token: config.telegram_config.botToken.clone(),
        })))
        .with_init_params(Some({
            let mut params = InitParams::default();
            params.device_model = "message_bridge_rs".to_owned();
            if let Ok(url) = std::env::var("BRIDGE_TG_PROXY") {
                params.proxy_url = Some(url);
            }
            params
        }))
        .with_modules(Arc::new(vec![module]))
        .build()
        .unwrap();
    let arc = Arc::new(client);
    tokio::select! {
        _ = teleser::run_client_and_reconnect(arc.clone()) => {
            tracing::warn!("[TG] TG客户端退出");
        },
        _ = sync_message(bridge.clone(), arc) => {
            tracing::warn!("[TG] TG桥关闭");
        },
    }
}

pub struct TgNewMessage {
    pub config: Arc<Config>,
    pub bridge: Arc<BridgeClient>,
    pub pack_folder: String,
}

impl TgNewMessage {
    fn find_cfg_by_group(&self, group_id: i64) -> Option<&BridgeConfig> {
        let bridge_config = self
            .config
            .bridges
            .iter()
            .find(|b| group_id == b.tgGroup && b.enable);
        Some(bridge_config?)
    }
    async fn pack_chat(&self, event: &Message) {
        let chat = event.chat();
        let mut lock = PACK_MAP.lock().await;
        if !lock.contains_key(&chat.id()) {
            let pack = chat.pack();
            let _ = tokio::fs::write(
                Path::new(self.pack_folder.as_str()).join(format!("{}", chat.id())),
                pack.to_bytes(),
            )
            .await;
            lock.insert(chat.id(), pack);
        }
    }
}

#[async_trait]
impl NewMessageProcess for TgNewMessage {
    async fn handle(&self, client: &mut Client, event: &Message) -> Result<bool> {
        self.pack_chat(event).await;
        if !event.outgoing() {
            if let Chat::Group(group) = event.chat() {
                if let Some(config) = self.find_cfg_by_group(group.id()) {
                    if let Some(Chat::User(user)) = event.sender() {
                        // 为发送者申请桥用户
                        let bridge_user = apply_bridge_user(user.id(), user.full_name()).await;
                        // 组装向桥发送的消息体表单
                        let mut bridge_message = bridge::pojo::BridgeSendMessageForm {
                            sender_id: bridge_user.id,
                            avatar_url: None,
                            bridge_config: config.clone(),
                            message_chain: Vec::new(),
                            origin_message: bridge::pojo::BridgeMessageRefPO {
                                origin_id: "".to_string(),
                                platform: "TG".to_string(),
                            },
                        };
                        // 下载图片
                        let media = event.media();
                        if let Some(Media::Photo(_)) = &media {
                            // download media 存在一定时间以后不能使用的BUG, 已经使用临时仓库解决
                            // see: https://github.com/Lonami/grammers/issues/166
                            match download_media(client, &media.unwrap()).await {
                                Ok(data) => bridge_message
                                    .message_chain
                                    .push(MessageContent::Image(Image::Buff(data))),
                                Err(err) => {
                                    error!("下载TG图片失败 : {:?}", err)
                                }
                            }
                        }
                        if !event.text().is_empty() {
                            if let Some(entities) = event.fmt_entities() {
                                let text = event.text();
                                let mut offset: usize = 0;
                                for x in entities {
                                    match x {
                                        MessageEntity::Mention(m) => {
                                            // todo 用数据库和更新用户名和id的对应关系, 因为机器人api不允许使用用户名转id的功能
                                            bridge_message.message_chain.push(Plain {
                                                text: text[offset..(m.offset) as usize].to_string(),
                                            });
                                            offset = (m.offset + m.length) as usize;
                                            // todo @
                                            // bridge_message.message_chain.push(At {
                                            // });
                                            bridge_message.message_chain.push(Plain {
                                                text: text[(m.offset as usize)
                                                    ..((m.offset + m.length) as usize)]
                                                    .to_string(),
                                            });
                                        }
                                        MessageEntity::MentionName(m) => {
                                            if offset < m.offset as usize {
                                                bridge_message.message_chain.push(Plain {
                                                    text: text[offset..(m.offset as usize)]
                                                        .to_string(),
                                                });
                                                offset = (m.offset + m.length) as usize;
                                                // todo @
                                                // bridge_message.message_chain.push(At {
                                                // });
                                                bridge_message.message_chain.push(Plain {
                                                    text: format!(
                                                        "@{}",
                                                        text[(m.offset as usize)
                                                            ..((m.offset + m.length) as usize)]
                                                            .to_string()
                                                    ),
                                                });
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            } else {
                                bridge_message.message_chain.push(Plain {
                                    text: event.text().to_owned(),
                                });
                            }
                        }
                        if !bridge_message.message_chain.is_empty() {
                            self.bridge.send_message(bridge_message).await;
                        }
                    }
                }
            }
        }
        Ok(false)
    }
}

lazy_static! {
    static ref PACK_MAP: Mutex<HashMap::<i64, PackedChat>> = Mutex::new(HashMap::new());
}

pub async fn sync_message(bridge: Arc<bridge::BridgeClient>, teleser_client: Arc<teleser::Client>) {
    let mut subs = bridge.sender.subscribe();
    loop {
        let message = match subs.recv().await {
            Ok(m) => m,
            Err(err) => {
                error!(?err, "[tg] 消息同步失败");
                continue;
            }
        };
        // 配置发送者头像
        if let Some(avatar_url) = &message.avatar_url {
            debug!("用户头像: {:?}", avatar_url);
        }
        let bridge_user = bridge::manager::BRIDGE_USER_MANAGER
            .lock()
            .await
            .get(&message.sender_id)
            .await
            .unwrap();
        // telegram 每条消息只能带一个附件或一个图片
        // 同时可以发一组图片消息，但是只有第一个图片消息可以带文字，文字会显示到一组消息的最下方
        // todo 发送图片消息和 @
        let mut builder = vec![];
        let mut images = vec![];
        for x in &message.message_chain {
            match x {
                MessageContent::Reply { .. } => {}
                MessageContent::Plain { text } => builder.push(text.as_str()),
                MessageContent::At { .. } => {
                    // todo
                }
                MessageContent::AtAll => {}
                MessageContent::Image(image) => images.push(image),
                MessageContent::Err { .. } => {}
                MessageContent::Othen => {}
            }
        }
        // 获取PACK
        let map_lock = PACK_MAP.lock().await;
        let chat = match map_lock.get(&message.bridge_config.tgGroup) {
            Some(chat) => Some(chat.clone()),
            None => {
                warn!("PACK 未找到 : {}", message.bridge_config.tgGroup);
                None
            }
        };
        drop(map_lock);
        //
        if let Some(chat) = chat {
            let lock = teleser_client.inner_client.lock().await;
            let inner_client = lock.clone();
            drop(lock);
            if let Some(inner_client) = inner_client {
                // send message
                if !images.is_empty() {
                    for x in images {
                        match x.clone().load_data().await {
                            Ok(data) => {
                                match image::guess_format(&data) {
                                    Ok(format) => {
                                        let len = data.len();
                                        let mut reader = std::io::Cursor::new(data);
                                        let upload = inner_client
                                            .upload_stream(
                                                &mut reader,
                                                len,
                                                format!("file.{}", format.extensions_str()[0]),
                                            )
                                            .await;
                                        match upload {
                                            Ok(img) => {
                                                let _ = inner_client
                                                    .send_message(
                                                        chat.clone(),
                                                        InputMessage::text(format!(
                                                            "{}:",
                                                            bridge_user.to_string(),
                                                        ))
                                                        .photo(img),
                                                    )
                                                    .await;
                                            }
                                            Err(_) => {}
                                        }
                                    }
                                    Err(_) => {}
                                };
                            }
                            Err(_) => {}
                        }
                    }
                }
                if !builder.is_empty() {
                    let send = builder.join("");
                    if !send.is_empty() {
                        let _ = inner_client
                            .send_message(
                                chat.clone(),
                                format!("{} : {}", bridge_user.to_string(), send),
                            )
                            .await;
                    }
                }
            }
        }
    }
}

async fn download_media(c: &mut teleser::InnerClient, media: &Media) -> Result<Vec<u8>> {
    let mut data = Vec::<u8>::new();
    let mut download = c.iter_download(&media);
    while let Some(chunk) = download.next().await? {
        data.write(chunk.as_slice()).await?;
    }
    Ok(data)
}

/**
 * 申请桥用户
 */
async fn apply_bridge_user(id: i64, name: String) -> bridge::user::BridgeUser {
    let bridge_user = bridge::manager::BRIDGE_USER_MANAGER
        .lock()
        .await
        .likeAndSave(bridge::pojo::BridgeUserSaveForm {
            origin_id: id.to_string(),
            platform: "TG".to_string(),
            display_text: format!("{}({})", name, id),
        })
        .await;
    bridge_user.unwrap()
}
