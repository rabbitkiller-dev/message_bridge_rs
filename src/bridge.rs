use crate::BridgeConfig;
use serde::Deserialize;
use serde::Serialize;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeMessage {
    pub bridge_config: BridgeConfig,
    pub message_chain: MessageChain,
}
impl BridgeMessage {}

pub type MessageChain = Vec<MessageContent>;

#[serde(tag = "type")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Plain { text: String },
}

pub struct BridgeService {
    pub sender: broadcast::Sender<BridgeMessage>,
    pub receiver: broadcast::Receiver<BridgeMessage>,
}

impl BridgeService {
    pub fn new() -> Self {
        let (sender, mut receiver) = broadcast::channel(32);

        BridgeService { sender, receiver }
    }
}
