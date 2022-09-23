//! 接收桥收到的用户指令，加以识别和响应

use std::sync::Arc;

use chrono::Local;
use tracing::{debug, info, trace, warn};

use crate::bridge::{BridgeClient, BridgeClientPlatform, BridgeMessage, MessageContent::Plain, User};
use crate::bridge_cmd;
use crate::bridge_cmd::{bind_meta::BindMeta, Cmd::*};
use crate::bridge_data::bind_map::{add_bind, get_bind, rm_bind};
use crate::Config;

type CacheBind = Vec<(i64, BindMeta)>;

/// 缓存超时（毫秒）
const CACHE_TIMEOUT: i64 = 30_000;

/// 持续接收指令消息
pub async fn listen(conf: Arc<Config>, bridge: Arc<BridgeClient>) {
    // cache token - bind cmd
    let mut cache_bind: CacheBind = Vec::with_capacity(1024);
    let mut rx = bridge.sender.subscribe();

    loop {
        let msg = rx.recv().await.unwrap();
        // match cmd
        if let Some((cmd, args)) = bridge_cmd::kind(&msg.message_chain) {
            let result = match cmd {
                Help => get_help(&msg.user),
                Bind => try_cache_bind(&msg.user, &mut cache_bind, args).to_string(),
                ConfirmBind => try_bind(&msg.user, &mut cache_bind).to_string(),
                Unbind => try_unbind(&msg.user, args),
            };
            if result.is_empty() {
                continue;
            }
            let bc = conf.bridges.iter().find(|b| b.enable);
            if let Some(bc) = bc {
                let mut feedback = BridgeMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    bridge_config: bc.clone(),
                    message_chain: Vec::new(),
                    user: msg.user.clone(),
                };
                feedback.user.platform = BridgeClientPlatform::Cmd;
                feedback.message_chain.push(Plain { text: result });
                debug!("bot feedback: {:#?}", feedback);
                bridge.send(feedback);
            }
        }
    } // loop
}

/// 开启频道
pub async fn start(config: Arc<Config>, bridge: Arc<BridgeClient>) {
    tokio::select! {
        _ = listen(config.clone(), bridge.clone()) => {},
    }
}

/// 获取指令帮助
/// - `user` 调用者
fn get_help(user: &User) -> String {
    // !来点[搜图]
    // !废话生成器
    // !猜数字游戏

    // 管理员:
    // !服务器状态
    // !重启
    // !查看所有成员绑定关系
    // !绑定成员关联 [用户名] [用户名]
    // !解除成员关联 [用户名]
    let bind_str = match user.platform {
        BridgeClientPlatform::Discord => "!绑定 qq [qq号]".to_string(),
        BridgeClientPlatform::QQ => "!绑定 dc [#9617(仅填写数字)]".to_string(),
        _ => "".to_string(),
    };
    format!(
        "!帮助\n\
        !ping\n\
        {}\n\
        !确认绑定\n\
        !解除绑定\n\
        !查看绑定状态",
        bind_str
    )
}

/// 解析绑定指令的参数
/// - `args` 指令参数
fn parse_bind_args(args: &Vec<String>) -> Option<(BridgeClientPlatform, u64)> {
    let p: BridgeClientPlatform = match args[0].parse() {
        Err(e) => {
            warn!(?e, "无法绑定未定义平台的账户。");
            return None;
        }
        Ok(p) => p,
    };
    let u: u64 = match args[1].parse() {
        Err(_) => {
            warn!("目前只支持纯数字id。");
            return None;
        }
        Ok(p) => p,
    };
    Some((p, u))
}

/// 查询映射
/// - `from` 请求者信息
/// - `to` 指令参数：绑定的平台和用户id
fn is_mapping(from: (BridgeClientPlatform, u64), to: (BridgeClientPlatform, u64)) -> bool {
    if let Some(mapping) = get_bind(from, to.0) {
        if mapping.unique_id == to.1 || mapping.display_id == to.1 {
            info!(
                "'{} {}' 已映射至 '{} {}'",
                from.0, from.1, mapping.platform, mapping.unique_id
            );
            return true;
        }
    }
    false
}

/// 检查绑定指令，尝试缓存
/// - `sign` 指令内容
/// - `cache` 缓存集合
fn try_cache_bind(user: &User, caches: &mut CacheBind, args: Option<Vec<String>>) -> String {
    // TODO 防过量请求，避免缓存爆炸
    // TODO 检查权限
    if let Some(bind_to) = parse_bind_args(&args.unwrap()) {
        // 查询映射
        if is_mapping((user.platform, user.unique_id), bind_to) {
            return "此用户已绑定".to_string();
        }

        let new_meta = BindMeta::new((user.platform, user.unique_id), bind_to);
        let now = Local::now().timestamp_millis();
        caches.retain(|(t, old_meta)| {
            if now - *t > CACHE_TIMEOUT {
                trace!("缓存已过期 [dl:{} > ch:{}] {:?}", now - CACHE_TIMEOUT, *t, old_meta);
                return false;
            }
            if new_meta == *old_meta {
                trace!("刷新缓存 [{} -> {}]", *t, now);
                return false;
            }
            true
        });
        debug!("缓存绑定请求: {:?}", new_meta);
        caches.push((now, new_meta));
        info!("缓存请求数: {}", caches.len());
        return format!("已记录，{}秒后失效。", CACHE_TIMEOUT / 1000);
    }
    warn!("绑定指令参数解析失败！");
    get_help(user)
}

/// 尝试建立映射
/// - `user` 接受绑定的用户
/// - `cache` 缓存集合
fn try_bind(user: &User, caches: &mut CacheBind) -> &'static str {
    let deadline = Local::now().timestamp_millis() - CACHE_TIMEOUT;
    let mut opt: Option<BindMeta> = None;
    caches.retain(|(t, m)| {
        if *t < deadline {
            trace!("缓存已过期 [dl:{} > ch:{}] {:?}", deadline, *t, m);
            return false;
        }
        if m.to.platform == user.platform
            && (m.to.user == user.unique_id || m.to.user == user.display_id)
        {
            if opt == None {
                opt = Some(*m);
            }
            return false;
        }
        true
    });
    if let Some(m) = opt {
        let from = m.from.to_user();
        add_bind(&from, user);
        info!("{}({}) bind to {}", from.platform, from.unique_id, user.name);
        return "绑定完成"
    }
    ""
}

/// 尝试解绑
/// - `user` 发送解绑指令的用户
/// - `args` 指令参数
fn try_unbind(user: &User, args: Option<Vec<String>>) -> String {
    match (args.unwrap())[0].parse::<BridgeClientPlatform>() {
        Ok(p) => {
            let msg = if p == user.platform {
                "原地TP？"
            } else {
                if rm_bind((user.platform, user.unique_id), p) {
                    "已解除绑定"
                } else {
                    "未向此平台绑定用户"
                }
            };
            return msg.to_string();
        }
        e => warn!(?e, "无法绑定未定义平台的账户。"),
    }
    get_help(user)
}
