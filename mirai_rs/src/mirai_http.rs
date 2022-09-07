use crate::message::MessageChain;
use crate::message::{BaseResponse, EventPacket};
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
        let js = json!({
            "sessionKey": self.session_key,
            "group": group,
            "messageChain": message_chain
        });
        // let mut data = HashMap::new();
        // data.insert("verifyKey", self.verify_key.as_str());

        let response = match self
            .req
            .post(self.get_url("/sendGroupMessage"))
            .json(&js)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(err) => {
                println!("[mirai_http] send_group_message请求失败");
                println!("[mirai_http] {:?}", err);
                Result::Err(err)?
            }
        };
        println!("[mirai_http] send_group_message {}", response.status());
        let resp = response.text().await.unwrap();
        let resp: SendGroupMessageResponse = match serde_json::from_str(resp.as_str()) {
            Ok(resp) => resp,
            Err(err) => {
                println!("[mirai_http] send_group_message转换json失败");
                println!("[mirai_http] {:?}", resp);
                println!("[mirai_http] {:?}", err);
                Result::Err(err)?
            }
        };

        Ok(resp)
    }

    pub fn get_url(&self, uri: &str) -> String {
        return format!("http://{}:{}{}", self.host, self.port, uri);
    }
}
