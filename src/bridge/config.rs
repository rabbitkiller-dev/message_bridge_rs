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
