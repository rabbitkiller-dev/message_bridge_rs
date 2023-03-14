use crate::config::BridgeConfig;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeMessage {
    pub id: String,
    // 桥用户
    pub sender_id: String,
    // 头像链接
    pub avatar_url: Option<String>,
    pub bridge_config: BridgeConfig,
    pub message_chain: MessageChain,
}

pub type MessageChain = Vec<MessageContent>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageContent {
    /**
     * 回复
     */
    Reply {
        /**
         * 想要回复的桥消息id
         */
        id: Option<String>,
    },
    /**
     * 普通文本
     */
    Plain {
        text: String,
    },
    /**
     * 提及某人
     */
    At {
        /**
         * 目标用户的桥用户id
         */
        id: String,
    },
    /**
     * 提及所有人
     */
    AtAll,
    /**
     * 图片
     */
    Image(Image),
    /**
     * 发生了一些错误
     */
    Err {
        // 错误信息
        message: String,
    },
    Othen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Image {
    Url(String),
    Path(String),
    Buff(Vec<u8>),
}

impl Image {
    pub(crate) async fn load_data(self) -> anyhow::Result<Vec<u8>> {
        match self {
            Image::Url(url) => Ok(reqwest::get(url).await?.bytes().await?.to_vec()),
            Image::Path(path) => Ok(tokio::fs::read(path).await?),
            Image::Buff(data) => Ok(data),
        }
    }
}
