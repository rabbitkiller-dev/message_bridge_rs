use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
pub struct BridgeUserRefPO {
    /**
     * 关联id
     */
    pub id: String,
}
