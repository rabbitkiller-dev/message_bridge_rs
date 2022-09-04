use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::Target;
/**
 * 基础响应格式
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct BaseResponse<T> {
    pub code: u16,
    pub msg: String,
    pub data: T,
}
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

// #[serde(flatten)]
// extra: std::collections::HashMap<String, Value>,
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageEvent {
    GroupMessage(GroupMessage),
    TempMessage {
        #[serde(rename = "messageChain")]
        message_chain: MessageChain,
        sender: sender::GroupSender,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMessage {
    #[serde(rename = "messageChain")]
    pub message_chain: MessageChain,
    pub sender: sender::GroupSender,
}

mod sender {
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
    pub enum Permission {
        #[serde(rename = "ADMINISTRATOR")]
        Administrator,

        #[serde(rename = "OWNER")]
        Owner,

        #[serde(rename = "MEMBER")]
        Member,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GroupSender {
        pub id: Target,

        #[serde(rename = "memberName")]
        pub member_name: String,

        #[serde(rename = "specialTitle")]
        pub special_title: String,

        pub permission: Permission,

        #[serde(rename = "joinTimestamp")]
        pub join_timestamp: u64,

        #[serde(rename = "lastSpeakTimestamp")]
        pub last_speak_timestamp: u64,

        #[serde(rename = "muteTimeRemaining")]
        pub mute_time_remaining: u64,

        pub group: Group,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Group {
        pub id: Target,
        pub name: String,
        pub permission: Permission,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OtherClientSender {
        pub id: Target,
        pub platform: String,
    }
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
