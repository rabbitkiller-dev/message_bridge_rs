use crate::message::MessageChain;
use crate::Target;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMessage {
    #[serde(rename = "messageChain")]
    pub message_chain: MessageChain,
    pub sender: GroupSender,
}

/**
 *
 */
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
/**
 * 群成员信息类型
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    /**
     * 群名片
     */
    #[serde(rename = "memberName")]
    pub member_name: String,

    /**
     * 群权限 OWNER、ADMINISTRATOR 或 MEMBER
     */
    permission: Permission,
    /**
     * 群头衔
     */
    #[serde(rename = "specialTitle")]
    special_title: String,
    /**
     * 入群时间戳
     */
    #[serde(rename = "joinTimestamp")]
    join_timestamp: i64,
    /**
     * 上一次发言时间戳
     */
    #[serde(rename = "lastSpeakTimestamp")]
    last_speak_timestamp: i64,
    /**
     * 剩余禁言时间
     */
    #[serde(rename = "muteTimeRemaining")]
    mute_time_remaining: u64,
    /**
     * 所在的群
     */
    group: Group,
}
