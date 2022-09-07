///! 接收，处理用户指令
use std::sync::Arc;

use chrono::Local;

use crate::{bridge, Config};
use crate::bridge::{BridgeMessage, MessageChain, MessageContent};
use crate::bridge_cmd::Cmd::*;
use crate::bridge_cmd::{kind, CmdMeta};

type CacheBind = Vec<(i64, CmdMeta)>;

/// 缓存超时（毫秒）
const CACHE_TIMEOUT: i64 = 30_000;

/// 持续接收指令消息
pub async fn cmd(bridge: Arc<bridge::BridgeClient>) {
    // cache token - bind cmd
    let mut cache_bind: CacheBind = Vec::with_capacity(1024);
    let mut rx = bridge.sender.subscribe();

    loop {
        let sign = rx.recv().await.unwrap();
        // match cmd
        if let Some(cmd) = kind(&sign.message_chain) {
            match cmd {
                Bind => check_bind(&sign, &mut cache_bind),
            } // match cmd kind
        }
    } // loop
}

/// 开启频道
pub async fn start(_config: Arc<Config>, bridge: Arc<bridge::BridgeClient>) {
    tokio::select! {
        _ = cmd(bridge.clone()) => {},
    }
}

/// 检查绑定指令，尝试缓存
/// - sign 指令内容
/// - cache 缓存集合
fn check_bind(input: &BridgeMessage, caches: &mut CacheBind) {
    // TODO 检查权限
    let in_platform = match input.from_platform() {
        Some(p) => p,
        _ => { return; }
    };

    let in_plain = plain_token(&input.message_chain);
    let now = Local::now().timestamp_millis();
    let mut add_cache = true;
    caches.retain(|(t, m)| {
        // 剔除超时缓存
        if now - *t > CACHE_TIMEOUT {
            return false;
        }

        let cache_plain = plain_token(&m.token_chain);
        // TODO 检查指令格式
        // 检查重复
        if in_plain == cache_plain {
            add_cache = false;
            return true;
        }

        // TODO 检查映射
        // 尝试匹配
        if cache_plain.contains(&input.user.name) && in_platform != m.platform {
            // TODO 建立映射
            println!("\t\t{}\n\t\t\t匹配\n\t\t{}", in_plain, cache_plain);
            add_cache = false;
            return false;
        }

        true
    });
    if add_cache {
        let meta = CmdMeta {
            token_chain: input.message_chain.clone(),
            operator: input.user.clone(),
            platform: in_platform,
        };
        println!("cache bind-cmd {:?}", meta);
        caches.push((now, meta));
        println!("cache count {}", caches.len());
    }
}

/// 取指令文本
/// - token_chain 指令内容
fn plain_token(token_chain: &MessageChain) -> String {
    let mut plain = String::new();
    for token in token_chain {
        match token {
            MessageContent::Plain { text } => {
                plain += text
            }
            _ => {},
        }
    }
    plain
}
