use serde::Deserialize;
use serde::Serialize;

use crate::bridge;

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
    /**
     * 关联表id
     */
    pub ref_id: Option<String>,
}

impl BridgeUser {
    pub async fn findRefByPlatform(&self, platform: &str) -> Option<BridgeUser> {
        return if let Some(ref_id) = &self.ref_id {
            bridge::user_manager::bridge_user_manager.lock().await.findByRefAndPlatform(ref_id, platform).await
        } else {
            None
        }

    }
}
