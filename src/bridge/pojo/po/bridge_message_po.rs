use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
pub struct BridgeMessagePO {
    /**
     * id
     */
    pub id: String,
    /**
     * 平台: Discord = DC, QQ = QQ
     */
    pub platform: String,
    /**
     * 来源id
     */
    pub origin_id: String,
}
