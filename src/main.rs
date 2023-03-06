#![feature(fs_try_exists)]

use std::sync::{Arc, Mutex};

use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::*;

mod bridge;
mod bridge_cmd;
mod bridge_data;
mod bridge_dc;
mod bridge_log;
mod bridge_qq_for_ricq;
mod cmd_adapter;
mod config;
mod logger;
mod utils;

pub type HttpResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _log_guard = logger::init_logger();
    let config = Arc::new(Config::new());
    tracing::info!("config: {:#?}", config);
    tracing::info!("config loaded");
    let bridge_service = bridge::BridgeService::new();
    let bridge_service = Arc::new(Mutex::new(bridge_service));
    let bridge_dc_client =
        bridge::BridgeService::create_client("bridge_dc_client", bridge_service.clone());
    let bridge_qq_client =
        bridge::BridgeService::create_client("bridge_qq_client", bridge_service.clone());
    let bridge_cmd_adapter =
        bridge::BridgeService::create_client("bridge_cmd_adapter", bridge_service.clone());
    // let a = Some(bridge_service.clone());
    tracing::info!("bridge ready");

    tokio::select! {
        _ = bridge_dc::start(config.clone(), bridge_dc_client) => {},
        _ = bridge_qq_for_ricq::start(config.clone(), bridge_qq_client) => {},
        _ = cmd_adapter::start(config.clone(), bridge_cmd_adapter) => {},
    }

    Ok(())
}

fn _init_tracing_subscriber() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .without_time(),
        )
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("ricq", Level::DEBUG)
                .with_target("proc_qq", Level::DEBUG)
                // 这里改成自己的crate名称
                .with_target("message_bridge_rs", Level::DEBUG),
        )
        .init();
}

/// # 2元表达式宏 - Result
/// ## Example
/// ```
/// assert_eq!(elr!(Ok::<_, ()>(1) ;; 2), 1);
/// assert_eq!(elr!(Err(0) ;; 42), 42);
/// ```
#[macro_export]
macro_rules! elr {
    ($opt:expr ;; $ret:expr) => {
        if let Ok(v) = $opt {
            v
        } else {
            $ret
        }
    };
}
/// # 2元表达式宏 - Option
/// ## Example
/// ```
/// assert_eq!(elo!(Some(1) ;; 2), 1);
/// assert_eq!(elo!(None ;; 42), 42);
/// ```
#[macro_export]
macro_rules! elo {
    ($opt:expr ;; $ret:expr) => {
        if let Some(v) = $opt {
            v
        } else {
            $ret
        }
    };
}

#[cfg(test)]
#[allow(unused)]
mod test {
    use super::*;

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn ts_el() {
        assert_eq!(elr!(Ok::<_, ()>(1) ;; 2), 1);
        assert_eq!(elr!(Err(0) ;; 42), 42);
        assert_eq!(elo!(Some(1) ;; 2), 1);
        assert_eq!(elo!(None ;; 42), 42);
    }

    #[test]
    fn test() -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    #[test]
    fn get_config() {
        let config = Config::new();
        println!("config:");
        println!("{:?}", config);
    }
}
