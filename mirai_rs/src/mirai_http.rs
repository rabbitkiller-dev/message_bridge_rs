use crate::message::MessageChain;
use crate::message::{BaseResponse, EventPacket, MessageEvent};
use crate::model::SendGroupMessageResponse;
use crate::{HttpResult, Mirai};

use serde_json::json;
use std::collections::HashMap;

pub struct MiraiHttp {
    host: String,
    port: u32,
    verify_key: String,
    qq: u32,
    session_key: String,
    req: reqwest::Client,
}

impl MiraiHttp {
    pub fn new(mirai: &Mirai) -> Self {
        MiraiHttp {
            host: mirai.host.clone(),
            port: mirai.port,
            verify_key: mirai.verify_key.clone(),
            qq: mirai.qq,
            session_key: mirai.session_key.clone(),
            req: reqwest::Client::new(),
        }
    }

    pub async fn fetch_message(&self, count: u32) -> HttpResult<BaseResponse<Vec<EventPacket>>> {
        let path = format!(
            "/fetchMessage?sessionKey={}&count={}",
            &self.session_key, count
        );
        let client = reqwest::Client::new();
        let resp: BaseResponse<Vec<EventPacket>> = client
            .get(&self.get_url(path.as_str()))
            .send()
            .await?
            .json()
            .await?;

        Ok(resp)
    }
    pub async fn send_group_message(
        &self,
        message_chain: MessageChain,
        group: u64,
    ) -> HttpResult<SendGroupMessageResponse> {
        // {
        //     "sessionKey":"YourSession",
        //     "target":987654321,
        //     "messageChain":[
        //       { "type":"Plain", "text":"hello\n" },
        //       { "type":"Plain", "text":"world" },
        //       { "type":"Image", "url":"https://i0.hdslb.com/bfs/album/67fc4e6b417d9c68ef98ba71d5e79505bbad97a1.png" }
        //     ]
        //   }
        let js = json!({
            "sessionKey": self.session_key,
            "group": group,
            "messageChain": message_chain
        });
        let client = reqwest::Client::new();
        // let mut data = HashMap::new();
        // data.insert("verifyKey", self.verify_key.as_str());

        let resp: SendGroupMessageResponse = self
            .req
            .post(self.get_url("/sendGroupMessage"))
            .json(&js)
            .send()
            .await?
            .json()
            .await?;

        Ok(resp)
    }

    pub fn get_url(&self, uri: &str) -> String {
        return format!("http://{}:{}{}", self.host, self.port, uri);
    }
}
