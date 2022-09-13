use crate::Config;
use mirai_rs::api::MessageEvent;
use mirai_rs::message::{MessageChain, MessageContent};
use mirai_rs::EventHandler;
use mirai_rs::Mirai;

use std::sync::Arc;

pub struct MiraiBridgeHandler;
#[mirai_rs::async_trait]
impl EventHandler for MiraiBridgeHandler {
    async fn message(&self, ctx: &Mirai, msg: MessageEvent) {}
}

#[cfg(test)]
#[allow(non_snake_case)]
fn test() {}

#[test]
fn test_mirai_send_group_message() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let config = Arc::new(Config::new());
            let mut mirai = Mirai::builder(
                &config.miraiConfig.host,
                config.miraiConfig.port,
                &config.miraiConfig.verifyKey,
            )
            .bind_qq(3245538509)
            .event_handler(MiraiBridgeHandler)
            .await;
            let http = mirai.get_http().await;
            let mut message_chian: MessageChain = vec![];
            message_chian.push(MessageContent::Plain {
                text: "测试发送消息".to_string(),
            });
            let result = http
                .send_group_message(message_chian, 518986671)
                .await
                .unwrap();
            println!("请求成功");
            println!("{:?}", result);
        })
}

#[test]
fn test_mirai_get_group_user() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let config = Arc::new(Config::new());
            let mut mirai = Mirai::builder(
                &config.miraiConfig.host,
                config.miraiConfig.port,
                &config.miraiConfig.verifyKey,
            )
            .bind_qq(3245538509)
            .event_handler(MiraiBridgeHandler)
            .await;
            let http = mirai.get_http().await;
            let result = http.get_member_info(518986671, 243249439).await.unwrap();
            println!("请求成功");
            println!("{:?}", result);
        })
}

#[test]
fn test_mirai_get_group_all_user() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let config = Arc::new(Config::new());
            let mut mirai = Mirai::builder(
                &config.miraiConfig.host,
                config.miraiConfig.port,
                &config.miraiConfig.verifyKey,
            )
            .bind_qq(3245538509)
            .event_handler(MiraiBridgeHandler)
            .await;
            let http = mirai.get_http().await;
            let result = http.member_list(518986671).await.unwrap();
            println!("请求成功");
            println!("{:?}", result);
        })
}
