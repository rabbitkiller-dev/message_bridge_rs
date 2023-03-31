use clap::{FromArgMatches, Parser, Subcommand};

use super::{pojo::BridgeSendMessageForm, MessageContent};

const CMD_ALL: &str = "全部指令";
const CMD_BIND: &str = "绑定";
const CMD_UNBIND: &str = "解除绑定";
const CMD_CONFIRM_BIND: &str = "确认绑定";

/// 定义指令关键字、结构
#[derive(Parser, Debug)]
pub enum BridgeCommand {
    /// 列出所有指令
    #[command(name = CMD_ALL)]
    AllCmd,
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
    fn try_parse(&self) -> Option<CommandCentext>;
}

/// 实现以 BridgeSendMessageForm 为载体的指令识别解析过程
impl CommandMessageParser for BridgeSendMessageForm {
    fn try_parse(&self) -> Option<CommandCentext> {
        let ctx = &self.message_chain;
        let text = if let Some(MessageContent::Plain { text }) = ctx.get(0) {
            text.trim()
        } else {
            tracing::debug!("no plain");
            return None;
        };
        if text.is_empty() || !text.starts_with("!") {
            tracing::debug!("plain empty or does not start with '!'");
            return None;
        }
        let text = &text[1..];
        let args = text.split_whitespace();
        let patter = BridgeCommand::augment_subcommands(clap::Command::new("cc").no_binary_name(true));
        let Ok(mat) = patter.clone().try_get_matches_from(args) else {
            tracing::debug!("no command matche");
            return None;
        };
        let Ok(cmd) = BridgeCommand::from_arg_matches(&mat) else {
            tracing::debug!("no command matche");
            return None;
        };
        Some(CommandCentext { token: cmd, ctx: patter })
    }
}
