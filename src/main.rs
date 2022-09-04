mod bridge;
mod bridge_dc;
mod bridge_qq;

use std::fs;
use std::path;
use std::sync::{Arc, Mutex};

use serde::Deserialize;
use serde::Serialize;
use serenity::prelude::*;

use mirai_rs::Mirai;

pub type HttpResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bridge_service = Arc::new(bridge::BridgeService::new());
    let config = Arc::new(Config::new());
    let mirai = Mirai::builder(
        &config.miraiConfig.host,
        config.miraiConfig.port,
        &config.miraiConfig.verifyKey,
    )
    .bind_qq(3245538509)
    .event_handler(bridge_qq::MiraiBridgeHandler {
        config: config.clone(),
        bridge: bridge_service.clone(),
    })
    .await;

    let token = &config.discordConfig.botToken;
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(bridge_dc::Handler {
            config: config.clone(),
            bridge: bridge_service.clone(),
        })
        .await
        .expect("Err creating client");
    tokio::select! {
        val = mirai.start() => {},
        val = client.start() => {},
        val = bridge_dc::dc(bridge_service.clone()) => {}
    }

    Ok(())
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use super::*;
    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn test() -> Result<(), Box<dyn std::error::Error>> {
        // let config = Config::new();
        // let mut mirai = Mirai::new(
        //     &config.miraiConfig.host,
        //     config.miraiConfig.port,
        //     &config.miraiConfig.verifyKey,
        // )
        // .bind_qq(3245538509);
        // let resp = tokio_test::block_on(mirai.verify());
        // println!("{:?}", resp);
        // let resp = tokio_test::block_on(mirai.bind());

        // println!("{:?}", resp);

        Ok(())
    }

    #[test]
    fn getConfig() {
        let config = Config::new();
        println!("config:");
        println!("{:?}", config);
    }
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct Config {
    miraiConfig: MiraiConfig,
    discordConfig: DiscordConfig,
    bridges: Vec<BridgeConfig>,
}

impl Config {
    pub fn new() -> Self {
        let file = fs::read_to_string("./config.json").unwrap();
        // println!("{file}");
        let config: Config = serde_json::from_str(file.as_str()).unwrap();

        config
    }
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
struct MiraiConfig {
    verifyKey: String,
    host: String,
    port: u32,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
struct DiscordConfig {
    botId: u64,
    botToken: String,
}

#[derive(Clone, Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct BridgeConfig {
    discord: DiscordBridgeConfig,
    qqGroup: u64,
    enable: bool,
}

#[derive(Clone, Deserialize, Serialize, Debug, Eq, PartialEq)]
struct DiscordBridgeConfig {
    id: u64,
    token: String,
    channelId: u64,
}

mod test_dc2;
