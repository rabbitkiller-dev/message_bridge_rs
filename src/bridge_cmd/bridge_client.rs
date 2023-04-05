use std::path::Path;
use std::sync::Arc;
use teleser::re_exports::grammers_client::types::message;

use crate::{bridge, bridge_dc, Config};
use crate::bridge::{MessageContent};

/**
 *
 */
pub async fn listen(bridge: Arc<bridge::BridgeClient>) {
    let mut subs = bridge.sender.subscribe();
    loop {
        let message = &subs.recv().await.unwrap();
        // 匹配消息是否是命令

        // 反馈命令消息
        let chain = match message.message_chain.first() {
            Some(chain) => chain,
            None => continue
        };
        if let MessageContent::Plain { text } = chain {
            if !text.starts_with("!hello") {
                continue;
            }
        }

        // 组装向桥发送的消息体表单
        let bridge_user = bridge::manager::BRIDGE_USER_MANAGER
            .lock()
            .await
            .like("00000001", "CMD")
            .await
            .unwrap();
        let mut bridge_message = bridge::pojo::BridgeSendMessageForm {
            sender_id: bridge_user.id,
            avatar_url: Some(format!("https://q1.qlogo.cn/g?b=qq&nk=3245538509&s=100")),
            bridge_config: message.bridge_config.clone(),
            message_chain: Vec::new(),
            // 来源是cmd自己
            origin_message: bridge::pojo::BridgeMessageRefPO {
                origin_id: uuid::Uuid::new_v4().to_string(),
                platform: "CMD".to_string(),
            },
        };
        // 回复world
        bridge_message.message_chain.push(MessageContent::Plain { text: "world!".to_string() });
        bridge.send_message(bridge_message).await
    }
}
