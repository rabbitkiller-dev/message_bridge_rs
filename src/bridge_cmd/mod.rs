use crate::{bridge, Config};
use clap::Parser;
use std::sync::Arc;

pub mod bridge_client;
pub mod process;

pub const CMD_TIP: &str = "!help";
pub const CMD_BIND: &str = "!关联";
pub const CMD_UNBIND: &str = "!解除关联";
pub const CMD_CONFIRM_BIND: &str = "!确认关联";

#[derive(Parser, Debug)]
pub enum BridgeCommand {
    #[command(name = CMD_BIND)]
    Bind {
        token: Option<String>,
    },
    #[command(name = CMD_UNBIND)]
    Unbind {
        platform: String,
    },
    #[command(name = CMD_CONFIRM_BIND)]
    ConfirmBind,
    #[command(name = CMD_TIP)]
    Tips {
        command: Option<String>,
    },
}

/// 指令内容
pub struct CommandCentext<M> {
    /// 基础内容
    pub token: BridgeCommand,
    /// 详细内容
    pub ctx: clap::Command,
    /// 客户端
    pub client: String,
    /// 源消息
    pub src_msg: M,
}

/// 指令消息解析
pub trait CommandMessageParser<M> {
    /// # 检查是否指令
    /// ### Argument
    /// - `&self` 待解析消息的载体
    /// - `client` 消息源客户端
    /// ### Return
    /// - 指令内容
    fn try_parse(&self, client: &str) -> Result<CommandCentext<M>, &'static str>;
}

pub async fn start(_config: Arc<Config>, bridge: Arc<bridge::BridgeClient>) {
    tracing::info!("[CMD] 初始化指令处理器");
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
