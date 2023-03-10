use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use teleser::re_exports::async_trait::async_trait;
use teleser::re_exports::grammers_client::types::{Chat, Message};
use teleser::re_exports::grammers_client::{Client, InitParams};
use teleser::re_exports::grammers_session::PackedChat;
use teleser::{Auth, ClientBuilder, FileSessionStore, NewMessageProcess, Process, StaticBotToken};
use tracing::{debug, error, warn};

use crate::bridge;
use crate::bridge::{BridgeClient, MessageContent};
use crate::config::{BridgeConfig, Config};

pub async fn start(config: Arc<Config>, bridge: Arc<bridge::BridgeClient>) {
    tracing::info!("[TG] 初始化TG桥");
    let module = teleser::Module {
        id: "tg_new_message".to_owned(),
        name: "tg_new_message".to_owned(),
        handlers: vec![teleser::Handler {
            id: "tg_new_message".to_owned(),
            process: Process::NewMessageProcess(Box::new(TgNewMessage {
                config: config.clone(),
                bridge: bridge.clone(),
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
}

#[async_trait]
impl NewMessageProcess for TgNewMessage {
    async fn handle(&self, _client: &mut Client, event: &Message) -> Result<bool> {
        if !event.outgoing() {
            if let Chat::Group(group) = event.chat() {
                if let Some(_config) = self.find_cfg_by_group(group.id()) {
                    // todo
                    // let mut bridge_message = BridgeMessage {
                    //     id: uuid::Uuid::new_v4().to_string(),
                    //     bridge_config: config.clone(),
                    //     message_chain: Vec::new(),
                    //     user: bridge::User {
                    //         name: format!("[TG] {}({})", sender_nickname, sender_id),
                    //         avatar_url: None,
                    //         unique_id: sender_id,
                    //         platform: BridgeClientPlatform::Telegram,
                    //         display_id: sender_id,
                    //         platform_id: group_id,
                    //     },
                    // };
                    // bridge.send(bridge_message);
                }
            }
        }
        Ok(false)
    }
}

pub async fn sync_message(bridge: Arc<bridge::BridgeClient>, teleser_client: Arc<teleser::Client>) {
    let mut pack_time = 0;
    let mut pack_map = HashMap::<i64, PackedChat>::new();
    let mut subs = bridge.sender.subscribe();
    'outer: loop {
        let message = match subs.recv().await {
            Ok(m) => m,
            Err(err) => {
                error!(?err, "[tg] 消息同步失败");
                continue;
            }
        };
        // 配置发送者头像
        if let Some(avatar_url) = &message.user.avatar_url {
            debug!("用户头像: {:?}", avatar_url);
        }
        //
        let mut builder = vec![];
        for x in &message.message_chain {
            match x {
                MessageContent::Plain { text } => builder.push(text.as_str()),
                _ => {}
            }
        }
        if !builder.is_empty() {
            let send = builder.join("");
            if !send.is_empty() {
                let lock = teleser_client.inner_client.lock().await;
                let inner_client = lock.clone();
                drop(lock);
                if let Some(inner_client) = inner_client {
                    let chat = if let Some(pack) = pack_map.get(&message.bridge_config.tgGroup) {
                        pack
                    } else {
                        let now = chrono::Local::now().timestamp();
                        if pack_time + 5 * 60 > now {
                            warn!("[TG] pack flood wait : {}", message.bridge_config.tgGroup);
                            continue;
                        }
                        pack_time = now;
                        let mut ds = inner_client.iter_dialogs();
                        loop {
                            match ds.next().await {
                                Ok(Some(dialog)) => {
                                    pack_map.insert(dialog.chat.id(), dialog.chat.pack());
                                }
                                Ok(None) => break,
                                Err(err) => {
                                    error!("[TG] pack err : {:?}", err);
                                    continue 'outer;
                                }
                            }
                        }
                        if let Some(pack) = pack_map.get(&message.bridge_config.tgGroup) {
                            pack
                        } else {
                            error!("[TG] group not found : {}", message.bridge_config.tgGroup);
                            continue 'outer;
                        }
                    };
                    let _ = inner_client.send_message(chat.clone(), send).await;
                }
            }
        }
    }
}
