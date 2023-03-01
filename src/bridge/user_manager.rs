use crate::bridge::pojo::BridgeUserSaveForm;
use crate::bridge::user::BridgeUser;
use std::fs;
use tracing_subscriber::fmt::format;

pub struct BridgeUserManager {
    bridge_users: Vec<BridgeUser>,
}

impl BridgeUserManager {
    pub fn new() -> BridgeUserManager {
        let path = "./bridge_user.json";
        if let Ok(true) = fs::try_exists(path) {
            let file = fs::read_to_string(path).unwrap();
            let bridge_users: Vec<BridgeUser> = serde_json::from_str(file.as_str()).unwrap();
            return BridgeUserManager {
                bridge_users
            };
        }
        BridgeUserManager {
            bridge_users: vec![]
        }
    }

    /// 根据id查询指定用户
    pub async fn get(&self, id: impl Into<String>) -> Option<BridgeUser> {
        self.bridge_users.into_iter().find(|user| {
            id.eq(&user.id)
        })
    }

    /// 模糊查询用户 (源id和平台)
    pub async fn like(&self, origin_id: impl Into<String>, platform: impl Into<String>) -> Option<BridgeUser> {
        self.bridge_users.into_iter().find(|user| {
            origin_id.eq(&user.id) && platform.eq(&user.platform)
        })
    }

    /// 保存一条新的用户
    pub async fn save(&mut self, form: BridgeUserSaveForm) -> Result<bool, String> {
        if let Some(_) = self.like(&form.origin_id, &form.platform) {
            let help = format!("该平台{:}已存在用户id为{:}的用户", &form.platform, &form.origin_id);
            Err(help)
        }
        let user = BridgeUser {
            id: uuid::Uuid::new_v4().to_string(),
            origin_id: form.origin_id,
            platform: form.platform,
            display_text: form.display_text,
        };
        self.bridge_users.push(user);
        Ok(true)
    }
}

pub const bridge_user: BridgeUserManager = BridgeUserManager::new();
