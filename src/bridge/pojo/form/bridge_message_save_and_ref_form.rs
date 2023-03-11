use crate::bridge;
pub struct BridgeMessageSaveAndRefForm {
    /**
     * 平台: Discord = DC, QQ = QQ
     */
    pub platform: String,
    /**
     * 来源id
     */
    pub origin_id: String,
    /**
     * 消息体
     */
    pub message_chain: bridge::MessageChain,
}
