use serde::Deserialize;
use serde::Serialize;
use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::bridge;

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
pub struct BridgeUser {
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
    /**
     * 查询该用户指定平台关联的用户
     */
    pub async fn find_by_platform(&self, platform: &str) -> Option<BridgeUser> {
        return if let Some(ref_id) = &self.ref_id {
            bridge::manager::BRIDGE_USER_MANAGER
                .lock()
                .await
                .findByRefAndPlatform(ref_id, platform)
                .await
        } else {
            None
        };
    }
}

impl Display for BridgeUser {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "[{}] {}", self.platform, self.display_text)
    }
}
