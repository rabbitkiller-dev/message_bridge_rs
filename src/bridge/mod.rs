use serenity::async_trait;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::BitOr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;
use serde::Serialize;
use tokio::sync::broadcast;

use crate::bridge::BridgeClientPlatform::*;
use crate::{bridge, BridgeConfig};

pub mod bridge_message_manager;
pub mod pojo;
pub mod user;
pub mod user_manager;
pub mod user_ref_manager;

pub use bridge_message_manager::*;

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
    // 桥用户
    pub bridge_user_id: String,
    // 头像链接
    pub avatar_url: Option<String>,
    pub bridge_config: BridgeConfig,
    pub message_chain: MessageChain,
}

impl BridgeMessage {}

pub type MessageChain = Vec<MessageContent>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageContent {
    /**
     * 回复
     */
    Reply {
        /**
         * 想要回复的桥消息id
         */
        id: Option<String>,
    },
    /**
     * 普通文本
     */
    Plain {
        text: String,
    },
    /**
     * 提及某人
     */
    At {
        /**
         * 目标用户的桥用户id
         */
        id: String,
    },
    /**
     * 提及所有人
     */
    AtAll,
    /**
     * 图片
     */
    Image(Image),
    /**
     * 发生了一些错误
     */
    Err {
        // 错误信息
        message: String,
    },
    Othen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Image {
    Url(String),
    Path(String),
    Buff(Vec<u8>),
}

impl Image {
    pub(crate) async fn load_data(self) -> anyhow::Result<Vec<u8>> {
        match self {
            Image::Url(url) => Ok(reqwest::get(url).await?.bytes().await?.to_vec()),
            Image::Path(path) => Ok(tokio::fs::read(path).await?),
            Image::Buff(data) => Ok(data),
        }
    }
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

    pub async fn create_client(
        name: &str,
        service: Arc<Mutex<BridgeService>>,
    ) -> Arc<BridgeClient> {
        let clients = &mut service.lock().await.clients;
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
        let bridge = self.bridge.lock().await;
        let id = bridge::bridge_message_manager::BRIDGE_MESSAGE_MANAGER
            .lock()
            .await
            .save(message.clone())
            .await;

        // let client = bridge
        //     .clients
        //     .iter()
        //     .filter(|client| &client.name != &self.name);

        let bridge_user_id = message.bridge_user_id.clone();
        let avatar_url = message.avatar_url.clone();
        let bridge_message = bridge::BridgeMessage {
            id,
            bridge_user_id,
            avatar_url,
            bridge_config: message.bridge_config,
            message_chain: message.message_chain,
        };

        for client in bridge.clients.iter() {
            if &client.name != &self.name {
                if let Err(e) = client.sender.send(bridge_message.clone()) {
                    println!("消息中转异常：{:#?}", e);
                }
            }
        }
    }
}
