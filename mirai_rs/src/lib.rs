mod adapter;
pub mod message;
pub mod mirai_http;
pub mod model;
pub mod response;

pub use async_trait::async_trait;
use core::panic;
use message::{EventPacket, MessageEvent};
use model::BaseResponse;
use response::{AboutResponse, BindResponse, VerifyResponse};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

pub type HttpResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub type Target = u64;

pub struct Mirai {
    host: String,
    port: u32,
    verify_key: String,
    qq: u32,
    session_key: String,
    event_handler: Option<Arc<dyn EventHandler>>,
}

impl Mirai {
    pub fn builder(host: &str, port: u32, verify_key: &str) -> MiraiBuilder {
        MiraiBuilder {
            host: host.to_string(),
            port,
            verify_key: verify_key.to_string(),
            qq: 0,
            event_handler: None,
        }
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

    pub async fn start(&mut self) {
        let event_handler = match &self.event_handler {
            Some(event_handler) => event_handler,
            None => {
                panic!("");
            }
        };
        loop {
            let result = self.fetch_message(1).await;
            let event_handler = event_handler.clone();
            match result {
                Ok(res) => {
                    for item in res.data {
                        if let EventPacket::MessageEvent(message) = item {
                            event_handler.message(&self, message).await;
                            continue;
                        }
                        println!("接收到其它消息");
                        println!("{:?}", serde_json::to_string(&item).unwrap());
                    }
                }
                Err(err) => {
                    println!("{:?}", err);
                    println!("获取信息失败");
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            //     let result = self.http_adapter.fetch_message(10).await;
            //     println!("接收到消息? {:?}", result);
            //     for event in result.data {
            //         match event {
            //             EventPacket::MessageEvent(message_event) => {
            //                 self.event_handler.message(message_event.clone()).await;
            //                 match message_event {
            //                     MessageEvent::GroupMessage(message) => {
            //                         self.event_handler.group_message(message).await
            //                     }
            //                     _ => println!(),
            //                 }
            //             }
            //             event => {
            //                 println!("{:?}", event);
            //                 eprintln!("没有处理的事件");
            //             }
            //         }
            //     }
        }
    }

    pub async fn get_http(&self) -> mirai_http::MiraiHttp {
        mirai_http::MiraiHttp::new(self)
    }

    pub fn get_url(&self, uri: &str) -> String {
        return format!("http://{}:{}{}", self.host, self.port, uri);
    }
}

pub struct MiraiBuilder {
    host: String,
    port: u32,
    verify_key: String,
    qq: u32,

    event_handler: Option<Arc<dyn EventHandler>>,
}

impl MiraiBuilder {
    pub fn bind_qq(mut self, qq: u32) -> Self {
        self.qq = qq;
        self
    }
    /// Sets an event handler with multiple methods for each possible event.
    pub async fn event_handler<H: EventHandler + 'static>(mut self, event_handler: H) -> Mirai {
        self.event_handler = Some(Arc::new(event_handler));

        let mut mirai = Mirai {
            host: self.host,
            port: self.port,
            verify_key: self.verify_key,
            qq: self.qq,
            event_handler: self.event_handler,
            session_key: "".to_string(),
        };

        println!("{},{}", &mirai.host, &mirai.port);

        match mirai.verify().await {
            Ok(res) => {
                mirai.session_key = res.session;
            }
            Err(err) => {
                eprintln!("{:?}", err);
                panic!("获取verify请求出错");
            }
        }

        match mirai.bind().await {
            Ok(res) => {}
            Err(err) => {
                println!("绑定qq请求出错")
            }
        }

        mirai
    }
}

pub mod api {
    pub use super::message::EventPacket;
    pub use super::message::MessageEvent;
}

/// The core trait for handling events by serenity.
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn message(&self, ctx: &Mirai, msg: MessageEvent);
}
