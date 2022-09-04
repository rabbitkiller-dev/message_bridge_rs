use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct SendGroupMessageResponse {
    code: u32,
    msg: String,
    messageId: u32,
}
