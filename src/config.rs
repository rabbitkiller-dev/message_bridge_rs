#![allow(non_snake_case)]

use proc_qq::Authentication;
use proc_qq::re_exports::ricq::version;
use serde::Deserialize;
use serde::Serialize;
use std::fs;

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct Config {
    /// 是否将二维码打印到终端
    #[serde(rename = "printQR")]
    pub print_qr: Option<bool>,
    #[serde(rename = "qqConfig")]
    pub qq_config: QQConfig,
    #[serde(rename = "discordConfig")]
    pub discord_config: DiscordConfig,
    #[serde(rename = "telegramConfig")]
    pub telegram_config: TelegramConfig,
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
    /// bot账号
    pub botId: Option<i64>,
    /// bot登录密码（可选）
    pub password: Option<String>,
    version: String,
    /// 登录认证方式（无token时）
    auth: String,
}
impl QQConfig {
    /// 获取认证方式
    pub fn get_auth(&self) -> anyhow::Result<Authentication> {
        use Authentication::*;
        match &*self.auth.to_lowercase() {
            "pwd" => {
                if self.botId.is_none() || self.password.is_none() {
                    return Err(anyhow::anyhow!("[QQ] 需配置账号(botId)密码(password)！"));
                }
                let pwd = self.password.as_ref().unwrap();
                if pwd.len() != 16 {
                    return Err(anyhow::anyhow!("[QQ] 密码请使用16位MD5加密"));
                }
                let mut buf = [0; 16];
                let mut x = 0;
                for b in pwd.bytes() {
                    if x > 15 {break}
                    buf[x] = b;
                    x += 1;
                }
                Ok(UinPasswordMd5(self.botId.unwrap(), buf))
            }
            "qr" => Ok(QRCode),
            _ => Err(anyhow::anyhow!("[QQ] 登录方式目前仅支持：二维码(qr)、账号密码(pwd)")),
        } // match
    }
    /// 获取客户端协议
    pub fn get_version(&self) -> anyhow::Result<&'static version::Version> {
        use proc_qq::re_exports::ricq::version::*;
        match &*self.version.to_lowercase() {
            "ipad" => Ok(&IPAD),
            "macos" => Ok(&MACOS),
            "qidian" => Ok(&QIDIAN),
            "androidphone" => Ok(&ANDROID_PHONE),
            "androidwatch" => Ok(&ANDROID_WATCH),
            v => Err(anyhow::anyhow!("[QQ] 暂不支持[{v}]协议，请更换！")),
        } // match
    }
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct DiscordConfig {
    pub botId: u64,
    pub botToken: String,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct TelegramConfig {
    pub apiId: i32,
    pub apiHash: String,
    pub botToken: String,
}

#[derive(Clone, Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct BridgeConfig {
    pub discord: DiscordBridgeConfig,
    pub qqGroup: u64,
    pub tgGroup: i64,
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
