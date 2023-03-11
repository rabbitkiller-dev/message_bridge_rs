use crate::bridge::MessageChain;
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BridgeMessagePO {
    /**
     * id
     */
    pub id: String,
    /**
     * 桥用户
     */
    pub sender_id: String,
    /**
     * 头像链接
     */
    pub avatar_url: Option<String>,
    /**
     * 关联桥消息的列表
     */
    pub refs: Vec<BridgeMessageRefPO>,
    /**
     * 消息内容
     */
    pub message_chain: MessageChain,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
pub struct BridgeMessageRefPO {
    /**
     * 平台: Discord = DC, QQ = QQ
     */
    pub platform: String,
    /**
     * 来源id
     */
    pub origin_id: String,
}
