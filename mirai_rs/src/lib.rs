mod response;

use response::{AboutResponse, BindResponse, VerifyResponse};
use serde_json::{json, Value};
use std::collections::HashMap;

pub type HttpResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Debug)]
pub struct Mirai {
    host: String,
    port: u32,
    verify_key: String,
    qq: u32,
    session_key: String,
}

impl Mirai {
    pub fn new(host: &str, port: u32, verify_key: &str) -> Self {
        Mirai {
            host: host.to_string(),
            port: port,
            verify_key: verify_key.to_string(),
            qq: 0,
            session_key: "".to_string(),
        }
    }

    pub fn bind_qq(&self, qq: u32) -> Mirai {
        // let mut mirai = Clone::clone(self);
        let mirai = Mirai {
            host: self.host.clone(),
            port: self.port,
            verify_key: self.verify_key.clone(),
            qq: qq,
            session_key: "".to_string(),
        };
        // mirai.qq = qq;
        // return mirai;
        mirai
    }

    /**
     * 认证
     * 发送verify_key获取session_key
     * https://github.com/project-mirai/mirai-api-http/blob/master/docs/adapter/HttpAdapter.md#%E8%AE%A4%E8%AF%81
     */
    pub async fn verify(&mut self) -> HttpResult<VerifyResponse> {
        let js = json!({"verifyKey": self.verify_key});
        let client = reqwest::Client::new();
        let mut data = HashMap::new();
        data.insert("verifyKey", self.verify_key.as_str());

        let resp: VerifyResponse = client
            .post(self.get_url("/verify"))
            .json(&data)
            .send()
            .await?
            .json()
            .await?;

        self.session_key = resp.session.clone();

        Ok(resp)
    }

    pub async fn bind(&self) -> HttpResult<BindResponse> {
        println!("self{:?}", &self);
        let js = json!({"verifyKey": self.verify_key});
        let client = reqwest::Client::new();
        let mut data: HashMap<&str, Value> = HashMap::new();
        data.insert("sessionKey", json!(self.session_key));
        data.insert("qq", json!(self.qq));

        println!("{:?}", data);

        let resp: BindResponse = client
            .post(self.get_url("/bind"))
            .json(&data)
            .send()
            .await?
            .json()
            .await?;

        Ok(resp)
    }

    pub async fn about() -> HttpResult<AboutResponse> {
        let client = reqwest::Client::new();
        let resp: AboutResponse = client
            .get("http://52.193.15.252:8080/about")
            .send()
            .await?
            .json()
            .await?;

        Ok(resp)
    }

    pub fn get_url(&self, uri: &str) -> String {
        return format!("{}:{}{}", self.host, self.port, uri);
    }
}
