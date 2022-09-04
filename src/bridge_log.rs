use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc, Local};
pub struct BridgeLog {

}
impl BridgeLog {
    pub fn write_log(content: &str) {
        let mut log_content = fs::read_to_string("./bridge_log.log").unwrap();
        let utc = Local::now().format("%Y-%m-%d %H:%M:%S");
        let content = format!(r#"{log_content}
===Start {utc}===
{content}
===End {utc}==="#);
        fs::write("./bridge_log.log", content).unwrap();
    }
}
#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use super::*;

    #[test]
    fn writeLog() {
        BridgeLog::write_log("test");
    }
}
