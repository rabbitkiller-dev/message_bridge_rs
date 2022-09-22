use std::sync::Arc;

use regex::Regex;
use tracing::{debug, error, info, instrument, trace, warn};

use mirai_rs::api::MessageEvent;
use mirai_rs::message::{MessageChain, MessageContent};
use mirai_rs::EventHandler;
use mirai_rs::Mirai;

use crate::bridge::BridgeClientPlatform;
use crate::bridge_message_history::{BridgeMessageHistory, Platform};
use crate::{bridge, utils, Config};

pub struct MiraiBridgeHandler {
    pub config: Arc<Config>,
    pub bridge: Arc<bridge::BridgeClient>,
}

#[instrument(skip_all, name = "bridge_qq_sync")]
pub async fn bridge_qq(bridge: Arc<bridge::BridgeClient>, mirai: mirai_rs::mirai_http::MiraiHttp) {
    let mut subs = bridge.sender.subscribe();
    loop {
        let message = &subs.recv().await.unwrap();
        info!("收到桥的消息, 同步到qq上");
        let mut message_chain: MessageChain = vec![];

        if let BridgeClientPlatform::Cmd = message.user.platform {
            // 平台方是cmd时不配置头像和名称
        } else {
            // 配置发送者头像
            if let Some(_) = message.user.avatar_url {
                debug!(url = ?message.user.avatar_url, "用户头像");
                message_chain.push(MessageContent::Image {
                    image_id: None,
                    url: message.user.avatar_url.clone(),
                    path: None,
                    base64: None,
                });
            }
            // 配置发送者用户名
            message_chain.push(MessageContent::Plain {
                text: format!("{}\n", message.user.name),
            });
        }

        for chain in message.message_chain.iter() {
            match chain {
                bridge::MessageContent::Plain { text } => {
                    message_chain.push(MessageContent::Plain { text: text.clone() })
                }
                bridge::MessageContent::Image { url, .. } => {
                    trace!(?url, "图片");
                    message_chain.push(MessageContent::Image {
                        image_id: None,
                        url: url.clone(),
                        path: None,
                        base64: None,
                    });
                }
                bridge::MessageContent::At { username, .. } => {
                    trace!("用户'{}'收到@", username);
                    let re = Regex::new(r"@\[QQ\] [^\n]+?\(([0-9]+)\)").unwrap();
                    let caps = re.captures(username);
                    match caps {
                        Some(caps) => {
                            let target = caps.get(1);
                            if let Some(target) = target {
                                let target = target.as_str().parse::<u64>().unwrap();
                                message_chain.push(MessageContent::At {
                                    target,
                                    display: Some("".to_string()),
                                })
                            } else {
                                warn!("@用户无法解析: {}", username);
                                message_chain.push(MessageContent::Plain {
                                    text: format!("@{}", username),
                                });
                            }
                        }
                        None => {
                            warn!("@用户无法解析: {}", username);
                            message_chain.push(MessageContent::Plain {
                                text: format!("@{}", username),
                            });
                        }
                    }
                }
                _ => warn!(unit = ?chain, "无法识别的MessageChain"),
            }
        }
        let resp = mirai
            .send_group_message(message_chain, message.bridge_config.qqGroup)
            .await;

        match resp {
            Ok(result) => {
                BridgeMessageHistory::insert(
                    &message.id,
                    Platform::QQ,
                    result.messageId.to_string().as_str(),
                )
                .await
                .unwrap();
                info!("已同步消息")
            }
            Err(err) => {
                error!(?err, "消息同步失败！")
            }
        }
    }
}

/**
 * 消息桥构建入口
 */
pub async fn start(config: Arc<Config>, bridge: Arc<bridge::BridgeClient>) {
    let mut mirai = Mirai::builder(
        &config.miraiConfig.host,
        config.miraiConfig.port,
        &config.miraiConfig.verifyKey,
    )
    .bind_qq(3245538509)
    .event_handler(MiraiBridgeHandler {
        config: config.clone(),
        bridge: bridge.clone(),
    })
    .await;
    let http = mirai.get_http().await;
    info!("qq(mirai) ready");

    tokio::select! {
        _ = mirai.start() => {
            warn!("qq(mirai) client exited")
        },
        _ = bridge_qq(bridge.clone(), http) => {
            warn!("bridge_qq listening is closed")
        },
    }
}

/**
 * 用来监听Mirai(qq)发送而来的事件
 */
#[mirai_rs::async_trait]
impl EventHandler for MiraiBridgeHandler {
    #[instrument(skip_all, name = "bridge_qq_recv")]
    async fn message(&self, ctx: &Mirai, msg: MessageEvent) {
        if let MessageEvent::GroupMessage(group_message) = msg {
            // 查询这个频道是否需要通知到群
            let bridge_config = match self
                .config
                .bridges
                .iter()
                .find(|bridge| group_message.sender.group.id == bridge.qqGroup && bridge.enable)
            {
                Some(bridge_config) => bridge_config,
                // 该消息的频道没有配置桥, 忽略这个消息
                None => return,
            };

            let user = bridge::User {
                name: format!(
                    "[QQ] {}({})",
                    group_message.sender.member_name.to_string(),
                    group_message.sender.id
                ),
                avatar_url: Some(format!(
                    "https://q1.qlogo.cn/g?b=qq&nk={}&s=100",
                    group_message.sender.id
                )),
                unique_id: group_message.sender.id,
                platform: BridgeClientPlatform::QQ,
                display_id: group_message.sender.id,
                platform_id: group_message.sender.group.id,
            };
            debug!("qq user: {:#?}", user);

            let mut bridge_message = bridge::BridgeMessage {
                id: uuid::Uuid::new_v4().to_string(),
                bridge_config: bridge_config.clone(),
                message_chain: Vec::new(),
                user,
            };

            for chain in &group_message.message_chain {
                match chain {
                    MessageContent::Source { id, time: _ } => {
                        let id = format!("{}", id);
                        // 记录消息id
                        BridgeMessageHistory::insert(
                            &bridge_message.id,
                            Platform::QQ,
                            id.as_str(),
                        )
                        .await.unwrap();
                    }
                    MessageContent::Plain { text } => {
                        trace!("plain: {}", text);
                        let result = utils::parser_message(text);
                        for ast in result {
                            match ast {
                                utils::MarkdownAst::Plain { text } => {
                                    bridge_message
                                        .message_chain
                                        .push(bridge::MessageContent::Plain { text });
                                }
                                utils::MarkdownAst::At { username } => {
                                    trace!("用户'{}'收到@", username);
                                    bridge_message
                                        .message_chain
                                        .push(bridge::MessageContent::At {
                                            bridge_user_id: None,
                                            username,
                                        });
                                }
                                utils::MarkdownAst::AtInDiscordUser { id } => {
                                    trace!("discord 用户'{}'收到@", id);
                                    bridge_message
                                        .message_chain
                                        .push(bridge::MessageContent::Plain { text: format!("<@{}>", id) });
                                }
                            }
                        }
                    }
                    MessageContent::Image { image_id: _, url, .. } => {
                        if let Some(url) = url {
                            debug!(?url, "图片");
                            let file_path = match utils::download_and_cache(url).await {
                                Ok(path) => {
                                    debug!(?path, "图片");
                                    Some(path)
                                }
                                Err(err) => {
                                    warn!(?err, "下载图片失败");
                                    None
                                }
                            };
                            // let base64 = image_base64::to_base64(path.as_str());
                            bridge_message.message_chain.push(bridge::MessageContent::Image { url: Some(url.to_string()), path: file_path })
                        }
                    }
                    MessageContent::At { target, .. } => {
                        let member = ctx.get_http().await.get_member_info(group_message.sender.group.id, target.clone()).await.unwrap();
                        let name = format!(
                            "[QQ] {}({})",
                            member.member_name.to_string(),
                            target
                        );
                        trace!("用户'{}'收到@", name);
                        bridge_message.message_chain.push(bridge::MessageContent::At { bridge_user_id: None, username: name });
                    }
                    _ => {
                        bridge_message.message_chain.push(bridge::MessageContent::Plain { text: "{没有处理qq的MessageChain}".to_string() });
                    }
                    // MessageContent::Quote { id, group_id, sender_id, target_id, origin } => todo!(),
                    // MessageContent::At { target, display } => todo!(),
                    // MessageContent::AtAll {  } => todo!(),
                    // MessageContent::Face { face_id, name } => todo!(),
                    // MessageContent::Plain { text } => todo!(),
                    // MessageContent::FlashImage { image_id, url, path, base64 } => todo!(),
                    // MessageContent::Voice { voice_id, url, path, base64, length } => todo!(),
                    // MessageContent::Xml { xml } => todo!(),
                    // MessageContent::Json { json } => todo!(),
                    // MessageContent::App { content } => todo!(),
                    // MessageContent::Poke { name } => todo!(),
                    // MessageContent::Dice { value } => todo!(),
                    // MessageContent::MusicShare { kind, title, summary, jump_url, picture_url, music_url, brief } => todo!(),
                    // MessageContent::ForwardMessage { sender_id, time, sender_name, message_chain, message_id } => todo!(),
                    // MessageContent::File { id, name, size } => todo!(),
                    // MessageContent::MiraiCode { code } => todo!(),
                }
            }
            debug!("接收到的 qq 群消息: {:#?}", group_message.message_chain);
            debug!("qq 桥的消息链: {:#?}", bridge_message.message_chain);

            self.bridge.send(bridge_message);
        }
    }
}
