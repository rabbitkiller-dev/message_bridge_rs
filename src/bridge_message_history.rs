use serde::Deserialize;
use serde::Serialize;
use tokio::fs;

pub struct BridgeMessageHistory;

impl BridgeMessageHistory {
    pub async fn insert(id: &str, platform: Platform, message_id: &str) -> Result<(), String> {
        let mut history = History {
            id: id.to_string(),
            message: vec![],
        };
        let message = HisotryMessage {
            platform: platform,
            message_id: message_id.to_string(),
        };
        history.message.push(message);

        let mut list = BridgeMessageHistory::find_all().await;

        let exsit = list.iter().find(|history| history.id == id);

        if let Some(_) = exsit {
            return Result::Err("插入相同的消息id".to_string());
        }

        list.push(history);

        BridgeMessageHistory::save(list).await;

        Result::Ok(())
    }

    pub async fn find_all() -> Vec<History> {
        let file = fs::read_to_string("./data/bridge_message_history.json")
            .await
            .unwrap();
        let config: Vec<History> = serde_json::from_str(file.as_str()).unwrap();
        config
    }

    pub async fn save(list: Vec<History>) {
        let content = serde_json::to_string(&list).unwrap();
        fs::write("./data/bridge_message_history.json", content)
            .await
            .unwrap();
    }

    pub fn find_by_message_id() {}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Platform {
    Discord,
    QQ,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    id: String,
    message: Vec<HisotryMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HisotryMessage {
    platform: Platform,
    message_id: String,
}
