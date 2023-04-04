//! 指令处理

use crate::bridge::pojo::BridgeSendMessageForm;
use crate::bridge::MessageContent;

use super::{BridgeCommand, CommandCentext, CMD_BIND, CMD_CONFIRM_BIND, CMD_UNBIND};

const STR_EMP: &str = "";

/// # 获取指令提示
fn get_tip_ctx(cmd: &str) -> String {
    match cmd {
        CMD_UNBIND => format!("【接收绑定申请】无参，用例: {CMD_CONFIRM_BIND}"),
        CMD_BIND => format!(
            "【申请绑定桥用户】
{CMD_BIND} qq dong(1111)
{CMD_BIND} dc dong#2222"
        ),
        CMD_CONFIRM_BIND => format!(
            "【解除桥用户绑定】
{CMD_UNBIND} qq
{CMD_UNBIND} dc"
        ),
        _ => format!(
            "可用指令：
【申请绑定桥用户】{CMD_BIND} qq dong(1111)
【解除桥用户绑定】{CMD_UNBIND} qq
【接收绑定申请】{CMD_CONFIRM_BIND}"
        ),
    }
}

/// # 获取指令帮助
/// ### Arguments
/// - `cmd` 指令内容
/// - `msg` 指令消息
/// ### Return
/// - `Ok(Some(BridgeMessage))` 反馈消息
/// - `Err(String)` 失败描述
fn get_help(cmd: &CommandCentext, _msg: &BridgeSendMessageForm) -> Result<Vec<MessageContent>, String> {
    let mut sub = STR_EMP.to_string();
    if let BridgeCommand::Tips { command } = &cmd.token {
        if let Some(tmp) = command {
            if tmp.starts_with('!') {
                sub = tmp.to_owned();
            } else {
                sub = format!("!{tmp}");
            }
        }
    }
    let text = get_tip_ctx(&sub);
    Ok(vec![MessageContent::Plain { text }])
}

impl CommandCentext {
    /// # 指令处理
    /// ### Arguments
    /// - `msg` 指令消息
    /// ### Return
    /// `Some(feedback)` 反馈指令处理结果
    pub fn process(self, msg: &BridgeSendMessageForm) -> Result<Vec<MessageContent>, String> {
        use super::BridgeCommand::*;
        match self.token {
            Tips { .. } => get_help(&self, msg),
            _ => Err("TODO".to_string()),
        }
    }
}
