use lazy_static::lazy_static;
use std::fs;
use tokio::sync::Mutex;

use crate::bridge;

pub struct BridgeMessageManager {
    messages: Vec<bridge::pojo::BridgeMessagePO>,
}

impl BridgeMessageManager {
    pub fn new() -> BridgeMessageManager {
        let path = "./data/bridge_message.json";
        if let Ok(true) = fs::try_exists(path) {
            let file = fs::read_to_string(path).unwrap();
            return BridgeMessageManager {
                messages: serde_json::from_str(file.as_str()).unwrap(),
            };
        }
        BridgeMessageManager { messages: vec![] }
    }
    /**
     * 保存消息
     */
    pub async fn save(&mut self, form: bridge::pojo::BridgeSendMessageForm) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let mut bridge_message = bridge::pojo::BridgeMessagePO {
            id: id.clone(),
            refs: vec![],
            message_chain: form.message_chain,
        };
        bridge_message.refs.push(form.origin_message);
        self.messages.push(bridge_message);
        self.serialize();
        id
    }

    /**
     * 保存并且关联消息
     */
    pub async fn save_and_ref(
        &mut self,
        form: bridge::pojo::BridgeMessageSaveAndRefForm,
    ) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let mut bridge_message = bridge::pojo::BridgeMessagePO {
            id: id.clone(),
            refs: vec![],
            message_chain: vec![],
        };
        bridge_message.refs.push(bridge::pojo::BridgeMessageRefPO {
            platform: form.platform,
            origin_id: form.origin_id,
        });
        self.messages.push(bridge_message);
        self.serialize();
        id
    }
    /**
     * 关联消息桥消息
     */
    pub async fn ref_bridge_message(
        &mut self,
        form: bridge::pojo::BridgeMessageRefMessageForm,
    ) -> bool {
        let message = self
            .messages
            .iter_mut()
            .find(|message| form.bridge_message_id.eq(&message.id));
        match message {
            Some(message) => {
                message.refs.push(bridge::pojo::BridgeMessageRefPO {
                    origin_id: form.origin_id,
                    platform: form.platform,
                });
                self.serialize();
                true
            }
            None => false,
        }
        // for user in &self.bridge_users {
        //     if origin_id.eq(&user.origin_id) && platform.eq(&user.platform) {
        //         return Some(user.clone());
        //     }
        // }
        // let mut bridge_message = bridge::pojo::BridgeMessagePO {
        //     id: id.clone(),
        //     refs: vec![],
        // };
        // bridge_message.refs.push(bridge::pojo::BridgeMessageRefPO {
        //     platform: form.platform,
        //     origin_id: form.origin_id,
        // });
        // self.messages.push(bridge_message);
        // self.serialize();
        // id
    }

    fn serialize(&self) {
        let content = serde_json::to_string(&self.messages).unwrap();
        fs::write("./data/bridge_message.json", content).unwrap();
    }
}

lazy_static! {
    pub static ref BRIDGE_MESSAGE_MANAGER: Mutex<BridgeMessageManager> =
        Mutex::new(BridgeMessageManager::new());
}
