use crate::BridgeConfig;

use std::sync::{Arc, Mutex};

use serde::Deserialize;
use serde::Serialize;
use tokio::sync::broadcast;

/// 客户端所属平台
#[repr(u64)]
#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum BridgeClientPlatform {
    Discord = 1 << 0,
    QQ = 1 << 1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeMessage {
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
    Image {
        url: Option<String>,  // 图片地址, 通常是cdn或者远程
        path: Option<String>, // 本机图片地址
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub avatar_url: Option<String>,
    pub unique_id: u64,
    pub display_id: u16,
    pub platform_id: u64,
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
        let (sender, mut receiver) = broadcast::channel(32);
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
                client.sender.send(message.clone());
            }
        }
    }

    /// 发送到指定频道
    /// - cli 消息频道名
    pub fn send_to(&self, cli: &str, msg: &BridgeMessage) {
        let bridge = self.bridge.lock();
        match bridge {
            Ok(b) => {
                let client = b.clients.iter().find(|c| c.name == cli.to_string());
                if let Some(cli) = client {
                    if let Err(_) = cli.sender.send(msg.clone()) {
                        println!("All Share-Receiver handles have already been dropped");
                    }
                } else {
                    println!(r#"Can not found "{cli}""#);
                }
            }
            Err(_) => {
                println!("Err when get bridge lock");
                return;
            }
        } // match
    } // fn share
}
