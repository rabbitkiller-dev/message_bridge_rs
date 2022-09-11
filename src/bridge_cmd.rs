///! 集中定义用户指令

use regex::Regex;

use crate::bridge::{MessageChain, MessageContent, User};
use crate::bridge_cmd::bind_meta::BindMeta;
use crate::bridge_cmd::Cmd::*;

/// 指令类别
pub enum Cmd {
    /// dc,qq互相绑定
    Bind,
    /// 确认绑定
    ConfirmBind,
}

/// 识别指令类别
/// - `token_chain` 消息链
pub fn kind(token_chain: &MessageChain) -> Option<Cmd> {
    let mut is_cmd = false;
    for ctx in token_chain {
        if let MessageContent::Plain { text } = ctx {
            if !is_cmd && !text.starts_with('!') {
                break;
            }
            is_cmd = true;
            if Bind.get_regex().is_match(text) {
                return Some(Bind);
            }
            if ConfirmBind.get_regex().is_match(text) {
                return Some(ConfirmBind);
            }
        };
    }
    None
}

impl Cmd {
    /// 获取指令正则表达式
    /// TODO 改为静态 Regex
    pub fn get_regex(&self) -> Regex {
        let r = match self {
            // !绑定 平台 用户id
            Bind => r"\A!绑定 (\S+?) (\d{4,20})\z",
            ConfirmBind => r"\A!确认绑定\z",
        };
        Regex::new(r).unwrap()
    }

    /// 解析指令，提取参数
    /// - `input` 用户输入（文本）
    pub fn get_args(&self, input: &str) -> Vec<String> {
        let mut args = Vec::with_capacity(8);
        for cap in self.get_regex().captures_iter(input.trim()) {
            for (x, mat) in cap.iter().enumerate() {
                // cap[0] 是整句
                if x < 1 { continue; }
                if let Some(_) = mat {
                    args.push(cap[x].to_string())
                }
            }
        }
        args
    }
}// impl Cmd

#[cfg(test)]
mod ts_cmd_regex {
    use crate::bridge_cmd::Cmd::Bind;

    #[test]
    fn ts_get_args() {
        let inp = "!绑定 dc 123456789";
        println!("inp: '{}'", inp);
        let args = Bind.get_args(inp);
        for (x, a) in args.iter().enumerate() {
            println!("{}: '{}'", x, a);
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
            (self.from == other.from && self.to == other.to) ||
                (self.from == other.to && self.to == other.from)
        }
    }

    impl Eq for BindMeta {}
}
