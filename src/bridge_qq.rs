use crate::{bridge, Config};
use mirai_rs::api::MessageEvent;
use mirai_rs::message::{GroupMessage, MessageChain, MessageContent};
use mirai_rs::EventHandler;
use mirai_rs::Mirai;
use std::sync::{Arc, Mutex};
pub struct MiraiBridgeHandler {
    pub config: Arc<Config>,
    pub bridge: Arc<bridge::BridgeClient>,
}

pub async fn bridge_qq(bridge: Arc<bridge::BridgeClient>, mirai: mirai_rs::mirai_http::MiraiHttp) {
    loop {
        let message = bridge.sender.subscribe().recv().await.unwrap();
        println!("[bridge_qq] 收到桥的消息, 同步到qq上");
        println!("{:?}", message);
        let mut message_chain: MessageChain = vec![];

        // 配置用户名
        message_chain.push(MessageContent::Plain {
            text: format!("{}\n", message.user.name),
        });

        for chain in message.message_chain.iter() {
            match chain {
                bridge::MessageContent::Plain { text } => {
                    message_chain.push(MessageContent::Plain { text: text.clone() })
                }
            }
        }
        mirai
            .send_group_message(message_chain, message.bridge_config.qqGroup)
            .await;
    }
}

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
            };
            let mut bridge_message = bridge::BridgeMessage {
                bridge_config: bridge_config.clone(),
                message_chain: Vec::new(),
                user,
            };
            for chain in &group_message.message_chain {
                match chain {
                        MessageContent::Plain { text } => {
                            bridge_message.message_chain.push(bridge::MessageContent::Plain { text: text.to_string() })
                        }
                        _ => {
                            println!("消息的内容没有处理");
                        }
                        // MessageContent::Source { id, time } => todo!(),
                        // MessageContent::Quote { id, group_id, sender_id, target_id, origin } => todo!(),
                        // MessageContent::At { target, display } => todo!(),
                        // MessageContent::AtAll {  } => todo!(),
                        // MessageContent::Face { face_id, name } => todo!(),
                        // MessageContent::Plain { text } => todo!(),
                        // MessageContent::Image { image_id, url, path, base64 } => todo!(),
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
