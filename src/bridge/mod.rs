use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::BitOr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use serde::Deserialize;
use serde::Serialize;
use tokio::sync::broadcast;

use crate::bridge::BridgeClientPlatform::*;
use crate::BridgeConfig;

pub mod pojo;
pub mod user;
pub mod user_manager;
pub mod user_ref_manager;

/// 解析枚举文本错误
#[derive(Debug)]
pub struct ParseEnumErr(String);

impl Display for ParseEnumErr {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

/// 客户端所属平台
/// # implement
/// ## [`FromStr`]
/// 凭借该特征可以将 str 解析为枚举
/// ```
/// println!("{:?}", "qq".parse::<BridgeClientPlatform>());
/// ```
/// ## [`BitOr`]
/// 借此特征简化枚举的“位标识”操作
#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
#[repr(u64)]
pub enum BridgeClientPlatform {
    Discord = 1 << 0,
    QQ = 1 << 1,
    Cmd = 1 << 2,
    Telegram = 1 << 3,
}

impl BitOr for BridgeClientPlatform {
    type Output = u64;
    fn bitor(self, rhs: Self) -> Self::Output {
        self as u64 | rhs as u64
    }
}

impl Display for BridgeClientPlatform {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let name = match self {
            Discord => "DC",
            QQ => "QQ",
            Cmd => "CMD",
            Telegram => "TG",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for BridgeClientPlatform {
    type Err = ParseEnumErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("dc") {
            Ok(Discord)
        } else if s.eq_ignore_ascii_case("qq") {
            Ok(QQ)
        } else {
            Err(ParseEnumErr(format!("平台'{}'未定义", s)))
        }
    }
}

impl BridgeClientPlatform {
    /// 数值转枚举
    pub fn by(val: u64) -> Option<Self> {
        match val {
            1 => Some(Discord),
            2 => Some(QQ),
            _ => None,
        }
    }
}

#[cfg(test)]
mod ts_bridge_client_platform {
    use BCP::*;

    use crate::bridge::BridgeClientPlatform as BCP;

    #[test]
    fn ts_display() {
        println!("dc:{}, qq:{}", Discord, QQ)
    }

    #[test]
    fn ts_parse() {
        println!("parse 'qQ' to enum: {}", "qQ".parse::<BCP>().unwrap());
        println!("parse 'Dc' to enum: {}", "Dc".parse::<BCP>().unwrap());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeMessage {
    pub id: String,
    pub bridge_config: BridgeConfig,
    pub message_chain: MessageChain,
    pub user: User,
}

impl BridgeMessage {}

pub type MessageChain = Vec<MessageContent>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageContent {
    Plain {
        text: String,
    },
    At {
        id: String,
    },
    AtAll,
    Image {
        /// 图片地址, 通常是cdn或者远程
        url: Option<String>,
        /// 本机图片地址
        path: Option<String>,
    },
    Othen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// 全称
    pub name: String,
    /// 头像链接
    pub avatar_url: Option<String>,
    /// 系统id
    pub unique_id: u64,
    /// 显示id
    pub display_id: u64,
    /// 群组/伺服器id
    pub platform_id: u64,
    /// 客户端平台枚举
    pub platform: BridgeClientPlatform,
}

pub struct BridgeService {
    pub clients: Vec<Arc<BridgeClient>>,
}

impl BridgeService {
    pub fn new() -> Self {
        BridgeService { clients: vec![] }
    }

    pub fn create_client(name: &str, service: Arc<Mutex<BridgeService>>) -> Arc<BridgeClient> {
        let clients = &mut service.lock().unwrap().clients;
        let exist = clients.iter().find(|client| &client.name == name);
        if let Some(_) = exist {
            panic!("存在同一个桥名: {}", name);
        }
        let client = Arc::new(BridgeClient::new(name, service.clone()));
        clients.push(client.clone());
        client
    }
}

pub struct BridgeClient {
    pub name: String,
    pub bridge: Arc<Mutex<BridgeService>>,
    pub sender: broadcast::Sender<BridgeMessage>,
    pub receiver: broadcast::Receiver<BridgeMessage>,
}

impl BridgeClient {
    pub fn new(name: &str, bridge: Arc<Mutex<BridgeService>>) -> Self {
        let (sender, receiver) = broadcast::channel(32);
        BridgeClient {
            bridge: bridge,
            name: name.to_string(),
            sender,
            receiver,
        }
    }

    pub fn send(&self, message: BridgeMessage) {
        let bridge = self.bridge.lock().unwrap();
        for client in bridge.clients.iter() {
            if &client.name != &self.name {
                if let Err(e) = client.sender.send(message.clone()) {
                    println!("消息中转异常：{:#?}", e);
                }
            }
        }
    }
}
