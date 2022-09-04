use crate::BridgeConfig;

use std::sync::{Arc, Mutex};

use serde::Deserialize;
use serde::Serialize;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeMessage {
    pub bridge_config: BridgeConfig,
    pub message_chain: MessageChain,
    pub user: User,
}
impl BridgeMessage {}

pub type MessageChain = Vec<MessageContent>;

#[serde(tag = "type")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Plain { text: String },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub name: String,
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
            println!(
                "1({}), 2({}), 3({})",
                client.name,
                self.name,
                &client.name != &self.name
            );
            if &client.name != &self.name {
                client.sender.send(message.clone());
            }
        }
    }
}
