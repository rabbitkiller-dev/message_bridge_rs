#[macro_use]
use lazy_static::lazy_static;
use std::fs;
use tokio::sync::Mutex;

use crate::bridge::pojo::BridgeUserRefPO;
use crate::bridge::user::BridgeUser;

pub struct BridgeUserRefManager {
    bridge_user_refs: Vec<BridgeUserRefPO>,
}

impl BridgeUserRefManager {
    pub fn new() -> BridgeUserRefManager {
        let path = "./bridge_user_ref.json";
        if let Ok(true) = fs::try_exists(path) {
            let file = fs::read_to_string(path).unwrap();
            let bridge_user_refs: Vec<BridgeUserRefPO> = serde_json::from_str(file.as_str()).unwrap();
            return BridgeUserRefManager { bridge_user_refs };
        }
        BridgeUserRefManager {
            bridge_user_refs: vec![],
        }
    }

    pub async fn save(&mut self) -> Result<String, String> {
        let user = BridgeUserRefPO {
            id: uuid::Uuid::new_v4().to_string(),
        };
        self.bridge_user_refs.push(user.clone());
        self.serialize();
        Ok(user.id)
    }

    fn serialize(&self) {
        let content = serde_json::to_string(&self.bridge_user_refs).unwrap();
        fs::write("./bridge_user_ref.json", content).unwrap();
    }
}
lazy_static! {
    pub static ref bridge_user_ref_manager: Mutex<BridgeUserRefManager> = Mutex::new(BridgeUserRefManager::new());
}
