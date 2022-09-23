//! 集中定义用户指令

use lazy_static::lazy_static;

use regex::Regex;

use crate::bridge::{MessageChain, MessageContent};
use crate::bridge_cmd::Cmd::*;

/// 指令类别
pub enum Cmd {
    /// help
    Help,
    /// dc,qq互相绑定
    Bind,
    /// 解除绑定
    Unbind,
    /// 确认绑定
    ConfirmBind,
}

/// 指令元素
type CmdMeta = (Cmd, Option<Vec<String>>);

/// 识别指令类别
/// - `token_chain` 消息链
pub fn kind(token_chain: &MessageChain) -> Option<CmdMeta> {
    let mut is_cmd = false;
    for ctx in token_chain {
        if let MessageContent::Plain { text } = ctx {
            if !is_cmd && !text.starts_with('!') && !text.starts_with('！') {
                break;
            }
            is_cmd = true;
            if ConfirmBind.get_regex().is_match(text) {
                return Some((ConfirmBind, None));
            }
            if Help.get_regex().is_match(text) {
                return Some((Help, None));
            }

            if let Some(param) = Bind.get_args(text) {
                return Some((Bind, Some(param)));
            }

            if let Some(param) = Unbind.get_args(text) {
                return Some((Unbind, Some(param)));
            }
        };
    }
    None
}

impl Cmd {
    /// 获取指令正则表达式
    pub fn get_regex(&self) -> &Regex {
        lazy_static! {
            static ref REGEX_HELP: Regex = Regex::new(r"^[!！](?:帮助|help)").unwrap();
            static ref REGEX_BIND: Regex = Regex::new(r"^[!！](?:绑定|bind) (\S+?) (\d{4,20})$").unwrap();
            static ref REGEX_CONFIRM_BIND: Regex = Regex::new(r"^[!！](?:确认绑定|confirm-bind)$").unwrap();
            static ref REGEX_UNBIND: Regex = Regex::new(r"^[!！](?:解除绑定|unbind) (\S+?)$").unwrap();
        }

        match self {
            Help => &REGEX_HELP,
            Bind => &REGEX_BIND,
            Unbind => &REGEX_UNBIND,
            ConfirmBind => &REGEX_CONFIRM_BIND,
        }
    }

    /// 解析指令，提取参数
    /// - `input` 用户输入（文本）
    pub fn get_args(&self, input: &str) -> Option<Vec<String>> {
        let mut param = Vec::with_capacity(8);
        for cap in self.get_regex().captures_iter(input.trim()) {
            let ls = cap.iter().enumerate();
            if ls.len() < 2 {
                return None;
            }
            for (x, mat) in ls {
                // cap[0] 是整句
                if x < 1 {
                    continue;
                }
                if let Some(_) = mat {
                    param.push(cap[x].to_string());
                }
            }
        }
        Some(param)
    }
} // impl Cmd

#[cfg(test)]
mod ts_cmd_regex {
    use crate::bridge_cmd::Cmd::Bind;

    #[test]
    fn ts_get_args() {
        let inp = "!绑定 dc 123456789";
        println!("inp: '{}'", inp);
        let args = Bind.get_args(inp);
        for (x, a) in args.iter().enumerate() {
            println!("{}: {:?}", x, a);
        }
    }
}

///! 集中定义绑定指令的信息元素
pub mod bind_meta {
    use crate::bridge::{BridgeClientPlatform, User};

    /// 绑定的必要信息单元
    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    pub struct MetaUnit {
        /// 用户所属平台
        pub platform: BridgeClientPlatform,
        /// 用户的系统id
        pub user: u64,
    }

    impl MetaUnit {
        /// 组装不完整的用户信息
        pub fn to_user(&self) -> User {
            User {
                platform: self.platform,
                unique_id: self.user,
                avatar_url: None,
                platform_id: 0,
                display_id: 0,
                name: "".to_string(),
            }
        }
    }

    /// 绑定指令的信息由2个信息单元组成
    #[derive(Debug, Copy, Clone)]
    pub struct BindMeta {
        /// 发起端
        pub from: MetaUnit,
        /// 指向端
        pub to: MetaUnit,
    }

    impl BindMeta {
        /// 构造绑定信息。通过元组轻松输入
        pub fn new(a: (BridgeClientPlatform, u64), b: (BridgeClientPlatform, u64)) -> Self {
            BindMeta {
                from: MetaUnit {
                    platform: a.0,
                    user: a.1,
                },
                to: MetaUnit {
                    platform: b.0,
                    user: b.1,
                },
            }
        }
    }

    impl PartialEq<Self> for BindMeta {
        fn eq(&self, other: &Self) -> bool {
            (self.from == other.from && self.to == other.to)
                || (self.from == other.to && self.to == other.from)
        }
    }

    impl Eq for BindMeta {}
}
