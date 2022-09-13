use std::sync::Arc;

use mirai_rs::api::MessageEvent;
use mirai_rs::message::{MessageChain, MessageContent};
use mirai_rs::EventHandler;
use mirai_rs::Mirai;
use regex::Regex;

use crate::bridge::BridgeClientPlatform;
use crate::{bridge, Config};

pub struct MiraiBridgeHandler {
    pub config: Arc<Config>,
    pub bridge: Arc<bridge::BridgeClient>,
}

pub async fn bridge_qq(bridge: Arc<bridge::BridgeClient>, mirai: mirai_rs::mirai_http::MiraiHttp) {
    let mut subs = bridge.sender.subscribe();
    loop {
        let message = &subs.recv().await.unwrap();
        println!("[bridge_qq] 收到桥的消息, 同步到qq上");
        println!("{:?}", message);
        let mut message_chain: MessageChain = vec![];

        if let BridgeClientPlatform::Cmd = message.user.platform {
            // 平台方是cmd时不配置头像和名称
        } else {
            // 配置发送者头像
            if let Some(_) = message.user.avatar_url {
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
                bridge::MessageContent::Image { url, path: _ } => {
                    message_chain.push(MessageContent::Image {
                        image_id: None,
                        url: url.clone(),
                        path: None,
                        base64: None,
                    });
                }
                bridge::MessageContent::At {
                    bridge_user_id,
                    username,
                } => {
                    let re = Regex::new(r"@\[QQ\] [^\n]+?\(([0-9]+)\)").unwrap();
                    let caps = re.captures(username);
                    match caps {
                        Some(caps) => {
                            let target = caps.get(1);
                            if let Some(target) = target {
                                let target = target.as_str().parse::<u64>().unwrap();
                                message_chain.push(MessageContent::At {
                                    target: target,
                                    display: Some("".to_string()),
                                })
                            } else {
                                message_chain.push(MessageContent::Plain {
                                    text: "{@用户出现错误}".to_string(),
                                });
                                println!("[bridge_qq] @用户出现错误: {}", username);
                            }
                        }
                        None => {
                            message_chain.push(MessageContent::Plain {
                                text: format!("@{}", username),
                            });
                        }
                    }
                }
                _ => message_chain.push(MessageContent::Plain {
                    text: "{无法识别的MessageChain}".to_string(),
                }),
            }
        }
        match mirai
            .send_group_message(message_chain, message.bridge_config.qqGroup)
            .await
        {
            Ok(_) => {
                println!("[bridge_qq] 同步桥信息成功");
            }
            Err(err) => {
                println!("[bridge_qq] 同步桥信息失败");
                println!("[bridge_qq] {:?}", err);
            }
        };
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
    tokio::select! {
        _ = mirai.start() => {},
        _ = bridge_qq(bridge.clone(), http) => {},
    }
}

/**
 * 用来监听Mirai(qq)发送而来的事件
 */
#[mirai_rs::async_trait]
impl EventHandler for MiraiBridgeHandler {
    async fn message(&self, msg: MessageEvent) {
        if let MessageEvent::GroupMessage(group_message) = msg {
            // 查询这个频道是否需要通知到群
            let bridge_config = match self
                .config
                .bridges
                .iter()
                .find(|bridge| group_message.sender.group.id == bridge.qqGroup && bridge.enable)
            {
                Some(bridge_config) => bridge_config,
                None => {
                    // 该消息的频道没有配置桥, 忽略这个消息
                    return;
                }
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

            let mut bridge_message = bridge::BridgeMessage {
                bridge_config: bridge_config.clone(),
                message_chain: Vec::new(),
                user,
            };
            for chain in &group_message.message_chain {
                match chain {
                        MessageContent::Source { id: _, time: _ } => {}
                        MessageContent::Plain { text } => {
                            let result = crate::utils::parser_message(text);
                            for ast in result {
                                match ast {
                                    crate::utils::MarkdownAst::Plain { text } => {
                                        bridge_message
                                            .message_chain
                                            .push(bridge::MessageContent::Plain { text: text });
                                    }
                                    crate::utils::MarkdownAst::At { username } => {
                                        bridge_message
                                            .message_chain
                                            .push(bridge::MessageContent::At {
                                                bridge_user_id: None,
                                                username: username,
                                            });
                                    }
                                    crate::utils::MarkdownAst::AtInDiscordUser { id } => {
                                        bridge_message
                                            .message_chain
                                            .push(bridge::MessageContent::Plain { text: format!("<@{}>", id) } );
                                    }
                                }
                            }
                        }
                        MessageContent::Image { image_id: _, url, path: _, base64: _ } => {
                            if let Some(url) = url {
                                let file_path = match crate::utils::download_and_cache(url).await {
                                    Ok(path) => {
                                        Some(path)
                                    },
                                    Err(err) => {
                                        println!("[bridge_qq] 下载图片失败");
                                        println!("[bridge_qq] {:?}", err);
                                        None
                                    }
                                };
                                // let base64 = image_base64::to_base64(path.as_str());
                                bridge_message.message_chain.push(bridge::MessageContent::Image { url: Some(url.to_string()), path: file_path })
                            }
                        }
                        MessageContent::At { target, display } => {
                            bridge_message.message_chain.push(bridge::MessageContent::Plain { text: "{没有处理qq的MessageChain}".to_string() })
                        }
                        _ => {
                            bridge_message.message_chain.push(bridge::MessageContent::Plain { text: "{没有处理qq的MessageChain}".to_string() })
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
            self.bridge.send(bridge_message);
            println!("接收到群消息:");
            println!("{:?}", group_message);
            // println!("接收到群消息:");
            // println!("{:?}", group_message);
        }
    }
}
