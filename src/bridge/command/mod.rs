use clap::{FromArgMatches, Parser, Subcommand};

use crate::elr;

use super::{pojo::BridgeSendMessageForm, MessageContent};

pub mod handler;

pub const CMD_TIP: &str = "!help";
pub const CMD_BIND: &str = "!绑定";
pub const CMD_UNBIND: &str = "!解除绑定";
pub const CMD_CONFIRM_BIND: &str = "!确认绑定";

#[derive(Parser, Debug)]
pub enum BridgeCommand {
    /// 指令提示
    #[command(name = CMD_TIP)]
    Tips {
        /// 获取[command]的提示
        command: Option<String>,
    },
    /// 申请绑定
    #[command(name = CMD_BIND)]
    Bind {
        /// 指定目标所属平台
        platform: String,
        /// 指定用户+id。例：you(1234)
        user: String,
    },
    /// 解除绑定
    #[command(name = CMD_UNBIND)]
    Unbind {
        /// 指定解绑哪个平台
        platform: String,
    },
    /// 确认绑定
    #[command(name = CMD_CONFIRM_BIND)]
    ConfirmBind,
}

/// 指令内容
pub struct CommandCentext {
    /// 基础内容
    pub token: BridgeCommand,
    /// 详细内容
    pub ctx: clap::Command,
}

/// 指令消息解析
pub trait CommandMessageParser {
    /// # 检查是否指令
    /// ### Argument
    /// - `&self` 待解析消息的载体
    /// ### Return
    /// - 指令内容
    fn try_parse(&self) -> Option<CommandCentext>;
}

/// 识别解析以 BridgeSendMessageForm 为载体的指令
impl CommandMessageParser for BridgeSendMessageForm {
    fn try_parse(&self) -> Option<CommandCentext> {
        let ctx = &self.message_chain;
        let Some(MessageContent::Plain { text }) = ctx.get(0) else {
            return None;
        };
        let text = text.trim();
        if text.is_empty() || !text.starts_with('!') {
            return None;
        }
        let args = text.split_whitespace();
        let patter = BridgeCommand::augment_subcommands(clap::Command::new("cc").no_binary_name(true));
        let mat = elr!(patter.clone().try_get_matches_from(args) ;; return None);
        let cmd = elr!(BridgeCommand::from_arg_matches(&mat) ;; return None);
        Some(CommandCentext { token: cmd, ctx: patter })
    }
}
