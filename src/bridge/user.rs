use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
pub struct BridgeUser {
    /**
     * id
     */
    pub id: String,
    /**
     * 平台: Discord, QQ
     */
    pub platform: String,
    /**
     * 来源id
     */
    pub origin_id: String,
    /**
     * 平台: Discord, QQ
     */
    pub display_text: String,
}
