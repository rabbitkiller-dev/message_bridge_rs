use std::path::Path;
use std::sync::Arc;

use crate::{bridge, bridge_dc, Config};

pub mod bridge_client;

pub async fn start(config: Arc<Config>, bridge: Arc<bridge::BridgeClient>) {
    apply_bridge_user().await;
    bridge_client::listen(bridge.clone()).await;
}

/**
 * 申请桥用户
 */
pub async fn apply_bridge_user() -> bridge::user::BridgeUser {
    let bridge_user = bridge::manager::BRIDGE_USER_MANAGER
        .lock()
        .await
        .likeAndSave(bridge::pojo::BridgeUserSaveForm {
            origin_id: "00000001".to_string(),
            platform: "CMD".to_string(),
            display_text: "桥命令Bot".to_string(),
        })
        .await;
    bridge_user.unwrap()
}
