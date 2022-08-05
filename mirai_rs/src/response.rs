use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct AboutResponse {
    pub code: u32,
    pub data: AboutData,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct AboutData {
    pub version: String,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct VerifyResponse {
    pub code: u32,
    pub session: String,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct BindResponse {
    pub code: u32,
    pub msg: String,
}
