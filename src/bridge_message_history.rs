use serde::Deserialize;
use serde::Serialize;
use tokio::fs;

pub struct BridgeMessageHistory;

impl BridgeMessageHistory {
    pub async fn insert(id: &str, platform: Platform, message_id: &str) -> Result<(), String> {
        let mut list = BridgeMessageHistory::find_all().await;

        let history = match list.iter_mut().find(|history| history.id == id) {
            Some(history) => history,
            None => {
                let history = History {
                    id: id.to_string(),
                    message: vec![],
                };
                list.push(history);
                list.last_mut().unwrap()
            }
        };
        let exsit = history
            .message
            .iter()
            .find(|msg| msg.message_id == message_id && msg.platform == platform);

        if let Some(_) = exsit {
            println!("{:?}", exsit);
            return Result::Err("插入相同的消息id".to_string());
        }

        let message = HisotryMessage {
            platform: platform,
            message_id: message_id.to_string(),
        };
        history.message.push(message);

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

    pub async fn find_by_message_id(platform: Platform, message_id: &str) -> Option<History> {
        let list = BridgeMessageHistory::find_all().await;

        for history in list {
            let find = history
                .message
                .iter()
                .find(|msg| msg.message_id == message_id && msg.platform == platform);

            if let Some(_) = find {
                return Some(history);
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_inser() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                BridgeMessageHistory::insert("bridge_1", Platform::QQ, "qq_1")
                    .await
                    .unwrap();
            })
    }
}
