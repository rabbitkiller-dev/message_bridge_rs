use crate::{bridge, Config};
use mirai_rs::api::MessageEvent;
use mirai_rs::message::{GroupMessage, MessageContent};
use mirai_rs::EventHandler;
use std::sync::{Arc, Mutex};
pub struct MiraiBridgeHandler {
    pub config: Arc<Config>,
    pub bridge: Arc<bridge::BridgeService>,
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

            let sender = self.bridge.sender.clone();
            let mut bridge_message = bridge::BridgeMessage {
                bridge_config: bridge_config.clone(),
                message_chain: Vec::new(),
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
            sender.send(bridge_message);
            println!("接收到群消息:");
            println!("{:?}", group_message);
            // println!("接收到群消息:");
            // println!("{:?}", group_message);
        }
    }
}