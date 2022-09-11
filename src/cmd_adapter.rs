///! 接收桥收到的用户指令，加以识别和响应

use std::sync::Arc;

use chrono::Local;

use bridge_cmd::bind_meta::BindMeta;
use bridge_cmd::Cmd;
use Cmd::*;

use crate::{bridge_cmd, Config};
use crate::bridge::{BridgeClient, BridgeClientPlatform, BridgeMessage, MessageChain, MessageContent, User};
use crate::bridge_data::bind_map::{add_bind, get_bind};

type CacheBind = Vec<(i64, BindMeta)>;

/// 缓存超时（毫秒）
const CACHE_TIMEOUT: i64 = 30_000;

/// 持续接收指令消息
pub async fn listen(bridge: Arc<BridgeClient>) {
    // cache token - bind cmd
    let mut cache_bind: CacheBind = Vec::with_capacity(1024);
    let mut rx = bridge.sender.subscribe();

    loop {
        let msg = rx.recv().await.unwrap();
        // match cmd
        if let Some(cmd) = bridge_cmd::kind(&msg.message_chain) {
            match cmd {
                Bind => try_cache_bind(&msg, &mut cache_bind),
                ConfirmBind => try_bind(&msg.user, &mut cache_bind),
            }
        }
    } // loop
}

/// 开启频道
pub async fn start(_config: Arc<Config>, bridge: Arc<BridgeClient>) {
    tokio::select! {
        _ = listen(bridge.clone()) => {},
    }
}

/// 取指令文本
/// - `token_chain` 指令内容
fn plain_token(token_chain: &MessageChain) -> String {
    let mut plain = String::new();
    for token in token_chain {
        if let MessageContent::Plain { text } = token {
            plain += text
        }
    }
    plain
}

/// 解析绑定指令的参数
/// - `args` 指令参数
fn parse_bind_args(args: &Vec<String>) -> Option<(BridgeClientPlatform, u64)> {
    let p: BridgeClientPlatform = match args[0].parse() {
        Err(e) => {
            println!("无法绑定未定义平台的账户。{}", e);
            return None;
        }
        Ok(p) => p,
    };
    let u: u64 = match args[1].parse() {
        Err(_) => {
            println!("目前只支持纯数字id。");
            return None;
        }
        Ok(p) => p,
    };
    Some((p, u))
}

/// 查询映射
/// - `user` 请求者信息
/// - `to` 指令参数：绑定的平台和用户id
fn is_mapping(user: &User, to: (BridgeClientPlatform, u64)) -> bool {
    if let Some(u) = get_bind(user, to.0) {
        if u == to.1 {
            println!("'{}' 已映射至 '{} {}'", user.name, to.0, to.1);
            return true;
        }
    }
    false
}

/// 检查绑定指令，尝试缓存
/// - `sign` 指令内容
/// - `cache` 缓存集合
fn try_cache_bind(input: &BridgeMessage, caches: &mut CacheBind) {
    // TODO 防过量请求，避免缓存爆炸
    // TODO 检查权限
    // 解析参数
    let in_plain = plain_token(&input.message_chain);
    let plain_args = Bind.get_args(&in_plain);
    let bind_to: (BridgeClientPlatform, u64) = match parse_bind_args(&plain_args) {
        None => { return; }
        Some(a) => a,
    };
    // 查询映射
    if is_mapping(&input.user, bind_to) {
        return;
    }

    let new_meta = BindMeta::new((input.user.platform, input.user.unique_id), bind_to);
    let now = Local::now().timestamp_millis();
    let mut add_cache = true;
    caches.retain(|(t, old_meta)| {
        // 剔除超时缓存
        if now - *t > CACHE_TIMEOUT {
            return false;
        }
        // 检查重复
        if new_meta == *old_meta {
            add_cache = false;
            return true;
        }
        true
    });
    if add_cache {
        // TODO 提醒用户注意超时
        println!("缓存绑定请求: {:?}", new_meta);
        caches.push((now, new_meta));
        println!("缓存请求数: {}", caches.len());
    }
}

/// 尝试建立映射
/// - `user` 接受绑定的用户
/// - `cache` 缓存集合
fn try_bind(user: &User, caches: &mut CacheBind) {
    let deadline = Local::now().timestamp_millis() - CACHE_TIMEOUT;
    let mut opt: Option<BindMeta> = None;
    caches.retain(|(t, m)| {
        if *t < deadline {
            return false;
        }
        if m.to.platform == user.platform && m.to.user == user.unique_id {
            if opt == None {
                opt = Some(*m);
            }
            return false;
        }
        true
    });
    if let Some(m) = opt {
        let from = m.from.to_user();
        // TODO 验证映射用户信息有效
        println!("{}({}) bind to {}", from.platform, from.unique_id, user.name);
        add_bind(&from, user);
    }
    // TODO 反馈：绑定成功or失败
}
