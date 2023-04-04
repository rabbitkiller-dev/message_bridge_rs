use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::BitOr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;
use serde::Serialize;
use tokio::sync::broadcast;

use crate::{bridge, elo, elr};
use crate::bridge::BridgeClientPlatform::*;

pub mod bridge_message;
pub mod command;
pub mod manager;
pub mod pojo;
pub mod user;

pub use bridge_message::{BridgeMessage, Image, MessageChain, MessageContent};

use self::command::CommandMessageParser;

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
    Discord = 1,
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
        Ok(match &*s.to_lowercase() {
            "dc" => Discord,
            "qq" => QQ,
            "cmd" => Cmd,
            "tg" => Telegram,
            _ => return Err(ParseEnumErr(format!("平台'{}'未定义", s))),
        })
    }
}

impl BridgeClientPlatform {
    /// 数值转枚举
    pub fn by(val: u64) -> Option<Self> {
        Some(match val {
            1 => Discord,
            2 => QQ,
            4 => Cmd,
            8 => Telegram,
            _ => return None,
        })
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

pub struct BridgeService {
    pub clients: Vec<Arc<BridgeClient>>,
}

impl BridgeService {
    pub fn new() -> Self {
        BridgeService { clients: vec![] }
    }

    pub async fn create_client(name: &str, service: Arc<Mutex<BridgeService>>) -> Arc<BridgeClient> {
        let clients = &mut service.lock().await.clients;
        if clients.iter().any(|client| client.name == name) {
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
            bridge,
            name: name.to_string(),
            sender,
            receiver,
        }
    }

    /**
     * 向其它桥发送消息
     */
    pub async fn send_message(&self, message: bridge::pojo::BridgeSendMessageForm) {
        let msg_cp_cmd = message.clone();
        let bridge = self.bridge.lock().await;
        let id = bridge::manager::BRIDGE_MESSAGE_MANAGER.lock().await.save(message.clone()).await;

        // let client = bridge
        //     .clients
        //     .iter()
        //     .filter(|client| &client.name != &self.name);

        let bridge_message = bridge::BridgeMessage {
            id: id.clone(),
            sender_id: message.sender_id.clone(),
            avatar_url: message.avatar_url.clone(),
            bridge_config: message.bridge_config.clone(),
            message_chain: message.message_chain,
        };

        for client in bridge.clients.iter() {
            if &client.name != &self.name {
                if let Err(e) = client.sender.send(bridge_message.clone()) {
                    tracing::error!("消息中转异常：{:#?}", e);
                }
            }
        }

        // 尝试解析指令
        let cmd = elo!(msg_cp_cmd.try_parse() ;; return);
        tracing::info!("[指令] {:?}", cmd.token);
        let message_chain = elr!(cmd.process(&msg_cp_cmd) ;; return);
        let feedback = bridge::BridgeMessage {
            bridge_config: message.bridge_config.clone(),
            sender_id: message.sender_id.clone(),
            avatar_url: None,
            message_chain,
            id,
        };
        // 指令反馈
        for client in bridge.clients.iter() {
            if &client.name != &self.name {
                continue;
            }
            if let Err(e) = client.sender.send(feedback.clone()) {
                tracing::error!("消息中转异常：{e:#?}");
            }
        }// for
    }
}
