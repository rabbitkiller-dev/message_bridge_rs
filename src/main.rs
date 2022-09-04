mod bridge;
mod bridge_dc;
mod bridge_qq;

use std::fs;
use std::sync::{Arc, Mutex};

use serde::Deserialize;
use serde::Serialize;

pub type HttpResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Arc::new(Config::new());
    let mut bridge_service = bridge::BridgeService::new();
    let bridge_service = Arc::new(Mutex::new(bridge_service));
    let bridge_dc_client =
        bridge::BridgeService::create_client("bridge_dc_client", bridge_service.clone());
    let bridge_qq_client =
        bridge::BridgeService::create_client("bridge_qq_client", bridge_service.clone());
    // let a = Some(bridge_service.clone());

    tokio::select! {
        val = bridge_dc::start(config.clone(), bridge_dc_client) => {},
        val = bridge_qq::start(config.clone(), bridge_qq_client) => {},
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
mod test_mirai;
