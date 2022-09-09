use crate::bridge::{MessageChain, MessageContent, User};
use crate::bridge_cmd::Cmd::Bind;

/// 指令
#[derive(Debug)]
pub struct CmdMeta {
    /// 操作者
    pub operator: User,
    /// 指令文本链
    pub token_chain: MessageChain,
}

/// 指令类别
pub enum Cmd {
    /// dc,qq互相绑定
    Bind,
}

/// 识别指令类别
pub fn kind(token_chain: &MessageChain) -> Option<Cmd> {
    if let Some(first) = token_chain.get(0) {
        return match first {
            MessageContent::Plain { text } => {
                // TODO 正则，检验指令格式
                if text.starts_with("!绑定") {
                    return Some(Bind);
                }

                None// return
            }
            _ => None,// return
        };
    }
    None
}

