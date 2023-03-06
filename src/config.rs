#![allow(non_snake_case)]

use serde::Deserialize;
use serde::Serialize;
use std::fs;

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct Config {
    #[serde(rename = "qqConfig")]
    pub qq_config: QQConfig,
    #[serde(rename = "discordConfig")]
    pub discord_config: DiscordConfig,
    pub bridges: Vec<BridgeConfig>,
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
pub struct QQConfig {
    pub verifyKey: String,
    pub host: String,
    pub port: u32,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct DiscordConfig {
    pub botId: u64,
    pub botToken: String,
}

#[derive(Clone, Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct BridgeConfig {
    pub discord: DiscordBridgeConfig,
    pub qqGroup: u64,
    pub enable: bool,
}

#[derive(Clone, Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct DiscordBridgeConfig {
    pub id: u64,
    pub token: String,
    pub channelId: u64,
}

#[derive(Clone, Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct BridgeUser {
    id: String,
    qq: u64,
    discordId: u64,
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use super::*;

    #[test]
    fn getConfig() {
        let config = Config::new();
        println!("config:");
        println!("{:?}", config);
    }
}
