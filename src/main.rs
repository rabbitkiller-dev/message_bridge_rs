mod bridge;
mod bridge_dc;
mod bridge_qq;
mod bridge_log;
mod bridge_cmd;
mod cmd_adapter;
mod bridge_save;
mod config;

use config::*;
use std::sync::{Arc, Mutex};

pub type HttpResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Arc::new(Config::new());
    let mut bridge_service = bridge::BridgeService::new();
    let bridge_service = Arc::new(Mutex::new(bridge_service));
    let bridge_dc_client =
        bridge::BridgeService::create_client("bridge_dc_client", bridge_service.clone());
    let bridge_qq_client =
        bridge::BridgeService::create_client("bridge_qq_client", bridge_service.clone());
    let bridge_cmd_adapter =
        bridge::BridgeService::create_client("bridge_cmd_adapter", bridge_service.clone());
    // let a = Some(bridge_service.clone());

    tokio::select! {
        _ = bridge_dc::start(config.clone(), bridge_dc_client) => {},
        _ = bridge_qq::start(config.clone(), bridge_qq_client) => {},
        _ = cmd_adapter::start(config.clone(), bridge_cmd_adapter) => {},
    }

    Ok(())
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use super::*;
    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn test() -> Result<(), Box<dyn std::error::Error>> {
        // let config = Config::new();
        // let mut mirai = Mirai::new(
        //     &config.miraiConfig.host,
        //     config.miraiConfig.port,
        //     &config.miraiConfig.verifyKey,
        // )
        // .bind_qq(3245538509);
        // let resp = tokio_test::block_on(mirai.verify());
        // println!("{:?}", resp);
        // let resp = tokio_test::block_on(mirai.bind());

        // println!("{:?}", resp);

        Ok(())
    }

    #[test]
    fn getConfig() {
        let config = Config::new();
        println!("config:");
        println!("{:?}", config);
    }
}


// mod test_dc2;
mod test_mirai;
