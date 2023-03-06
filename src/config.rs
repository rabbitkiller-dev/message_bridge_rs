#![allow(non_snake_case)]

use serde::Deserialize;
use serde::Serialize;
use std::fs;

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct Config {
    pub miraiConfig: MiraiConfig,
    pub discordConfig: DiscordConfig,
    pub bridges: Vec<BridgeConfig>,
    pub bridgesUsers: Vec<BridgeUser>,
}

impl Config {
    pub fn new() -> Self {
        let file = fs::read_to_string("./config.json").unwrap();
        // println!("{file}");
        let config: Config = serde_json::from_str(file.as_str()).unwrap();

        config
    }

    pub fn add_user(&mut self, qq: u64, discord_id: u64) {
        self.bridgesUsers.push(BridgeUser {
            id: uuid::Uuid::new_v4().to_string(),
            qq,
            discordId: discord_id,
        });
        let content = serde_json::to_string(&self).unwrap();
        fs::write("./config.json", content).unwrap();
    }
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct MiraiConfig {
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

    #[test]
    fn addUser() {
        let mut config = Config::new();

        config.add_user(000321, 111111)
    }
}
