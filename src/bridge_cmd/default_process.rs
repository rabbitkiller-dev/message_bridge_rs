//! 指令处理

use crate::bridge::{MessageContent, BridgeMessage};

use super::{CMD_UNBIND, CMD_BIND, CMD_CONFIRM_BIND, CommandCentext, BridgeCommand};

impl CommandCentext<BridgeMessage> {
    /// # 获取指令帮助
    /// ### Return
    /// - `Ok(Some(BridgeMessage))` 反馈消息
    /// - `Err(String)` 失败描述
    fn get_help(&self) -> Result<Vec<MessageContent>, String> {
        let mut sub = "".to_string();
        if let BridgeCommand::Tips { command } = &self.token {
            if let Some(tmp) = command {
                if tmp.starts_with('!') {
                    sub = tmp.to_owned();
                } else {
                    sub = format!("!{tmp}");
                }
            }
        }
        let text = match &*sub {
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
        };
        Ok(vec![MessageContent::Plain { text }])
    }
    
    /// # 指令处理
    /// ### Return
    /// `Some(feedback)` 反馈指令处理结果
    pub fn process_command(&self) -> Result<Vec<MessageContent>, String> {
        use super::BridgeCommand::*;
        match self.token {
            Tips { .. } => self.get_help(),
            _ => Err("TODO".to_string()),
        }
    }
}
