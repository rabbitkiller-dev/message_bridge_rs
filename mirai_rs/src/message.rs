use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::model::group::{GroupMessage, GroupSender};
use crate::Target;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventPacket {
    MessageEvent(MessageEvent),
    // BotLoginEvent(),
    // BotMuteEvent(),
    // RecallEvent(),
    // GroupChangeEvent(),
    Unsupported(Value),
}

pub mod sender {
    use crate::Target;
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FriendSender {
        pub id: Target,
        pub nickname: String,
        pub remark: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OtherClientSender {
        pub id: Target,
        pub platform: String,
    }
}

// #[serde(flatten)]
// extra: std::collections::HashMap<String, Value>,
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageEvent {
    GroupMessage(GroupMessage),
    TempMessage {
        #[serde(rename = "messageChain")]
        message_chain: MessageChain,
        sender: GroupSender,
    },
    FriendMessage {
        #[serde(rename = "messageChain")]
        message_chain: MessageChain,
        sender: sender::FriendSender,
    },
    StrangerMessage {
        #[serde(rename = "messageChain")]
        message_chain: MessageChain,
        sender: sender::FriendSender,
    },
    OtherClientMessage {
        #[serde(rename = "messageChain")]
        message_chain: MessageChain,
        sender: sender::OtherClientSender,
    },
}

pub type MessageChain = Vec<MessageContent>;

#[serde(tag = "type")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Source {
        id: Target,
        time: u32,
    },
    #[serde(rename_all = "camelCase")]
    Quote {
        id: u32,
        group_id: Target,
        sender_id: Target,
        target_id: Target,
        origin: MessageChain,
    },
    At {
        target: Target,
        display: Option<String>,
    },
    AtAll {},
    #[serde(rename_all = "camelCase")]
    Face {
        face_id: Option<u16>,
        name: Option<String>,
    },
    Plain {
        text: String,
    },
    #[serde(rename_all = "camelCase")]
    Image {
        image_id: Option<String>,
        url: Option<String>,
        path: Option<String>,
        base64: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    FlashImage {
        image_id: Option<String>,
        url: Option<String>,
        path: Option<String>,
        base64: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Voice {
        voice_id: Option<String>,
        url: Option<String>,
        path: Option<String>,
        base64: Option<String>,
        length: Option<u32>,
    },
    Xml {
        xml: String,
    },
    Json {
        json: String,
    },
    App {
        content: String,
    },
    Poke {
        name: Poke,
    },
    Dice {
        value: u16,
    },
    #[serde(rename_all = "camelCase")]
    MusicShare {
        kind: String,
        title: String,
        summary: String,
        jump_url: String,
        picture_url: String,
        music_url: String,
        brief: String,
    },
    ForwardMessage {
        sender_id: Target,
        time: u32,
        sender_name: String,
        message_chain: MessageChain,
        message_id: u16,
    },
    File {
        id: String,
        name: String,
        size: u32,
    },
    MiraiCode {
        code: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Poke {
    Poke,
    ShowLove,
    Like,
    Heartbroken,
    SixSixSix,
    FangDaZhao,
}
