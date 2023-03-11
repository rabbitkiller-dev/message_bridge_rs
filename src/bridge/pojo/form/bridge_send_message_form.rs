use crate::bridge::{self, MessageContent};
use crate::BridgeConfig;
use serde::{Deserialize, Serialize};
/**
 * 向桥发送消息的表单
 */
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BridgeSendMessageForm {
    // 桥用户
    pub bridge_user_id: String,
    // 头像链接
    pub avatar_url: Option<String>,
    // 消息配置(TODO: 抽象配置)
    pub bridge_config: BridgeConfig,
    // 消息体
    pub message_chain: Vec<MessageContent>,
    // 消息来源
    pub origin_message: bridge::pojo::BridgeMessageRefPO,
}
