///! 接收桥收到的用户指令，加以识别和响应

use {
    chrono::Local,
    crate::{
        bridge::{
            BridgeClient,
            BridgeClientPlatform,
            BridgeMessage,
            MessageChain,
            MessageContent::Plain,
            User,
        },
        bridge_cmd::{
            bind_meta::BindMeta,
            Cmd::*,
        },
        bridge_cmd,
        bridge_data::bind_map::{add_bind, get_bind},
        Config,
    },
    std::sync::Arc,
};

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
        if let Some(cmd) = bridge_cmd::kind(&msg.message_chain) {
            let result = match cmd {
                Help => {
                    // !来点[搜图]
                    // !废话生成器
                    // !猜数字游戏

                    // 管理员:
                    // !服务器状态
                    // !重启
                    // !查看所有成员绑定关系
                    // !绑定成员关联 [用户名] [用户名]
                    // !解除成员关联 [用户名]
                    let bind_str = match msg.user.platform {
                        BridgeClientPlatform::Discord => "!绑定 qq [qq号]".to_string(),
                        BridgeClientPlatform::QQ => "!绑定 dc [#9617(仅填写数字)]".to_string(),
                        _ => "".to_string(),
                    };
                    format!(
                        r#"!帮助
!ping
{}
!确认绑定
!解除绑定
!查看绑定状态"#,
                        bind_str
                    )
                }
                Bind => {
                    let mut cache_msg = try_cache_bind(&msg, &mut cache_bind).to_string();
                    if cache_msg.is_empty() {
                        cache_msg = format!("已记录。{}秒后失效。", CACHE_TIMEOUT / 1000);
                    }
                    cache_msg
                }
                ConfirmBind => {
                    match try_bind(&msg.user, &mut cache_bind) {
                        Ok(o) => {
                            if o == None {
                                continue;
                            } else {
                                "绑定完成".to_string()
                            }
                        },
                        Err(_) => "绑定失败，请联系管理员".to_string(),
                    }
                }
            };
            if result.is_empty() {
                continue;
            }
            let bc = conf.bridges.iter().find(|b| b.enable);
            if let Some(bc) = bc {
                let mut feedback = BridgeMessage {
                    bridge_config: bc.clone(),
                    message_chain: Vec::new(),
                    user: msg.user.clone(),
                };
                feedback.user.platform = BridgeClientPlatform::Cmd;
                feedback.message_chain.push(Plain { text: result });
                println!("bot feedback: {:#?}", feedback);
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

/// 取指令文本
/// - `token_chain` 指令内容
fn plain_token(token_chain: &MessageChain) -> String {
    let mut plain = String::new();
    for token in token_chain {
        if let Plain { text } = token {
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
/// - `from` 请求者信息
/// - `to` 指令参数：绑定的平台和用户id
fn is_mapping(from: (BridgeClientPlatform, u64), to: (BridgeClientPlatform, u64)) -> bool {
    if let Some(mapping) = get_bind(from, to.0) {
        if mapping.unique_id == to.1 || mapping.display_id == to.1 {
            println!(
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
fn try_cache_bind<'r>(input: &'r BridgeMessage, caches: &mut CacheBind) -> &'r str {
    // TODO 防过量请求，避免缓存爆炸
    // TODO 检查权限
    // 解析参数
    let in_plain = plain_token(&input.message_chain);
    let plain_args = Bind.get_args(&in_plain);
    let bind_to: (BridgeClientPlatform, u64) = match parse_bind_args(&plain_args) {
        None => return "指令错误",
        Some(a) => a,
    };
    // 查询映射
    if is_mapping((input.user.platform, input.user.unique_id), bind_to) {
        return "此用户已绑定";
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
        println!("缓存绑定请求: {:?}", new_meta);
        caches.push((now, new_meta));
        println!("缓存请求数: {}", caches.len());
    }
    ""
}

/// 尝试建立映射
/// - `user` 接受绑定的用户
/// - `cache` 缓存集合
fn try_bind(user: &User, caches: &mut CacheBind) -> Result<Option<()>, ()> {
    let deadline = Local::now().timestamp_millis() - CACHE_TIMEOUT;
    let mut opt: Option<BindMeta> = None;
    caches.retain(|(t, m)| {
        if *t < deadline {
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
        println!(
            "{}({}) bind to {}",
            from.platform, from.unique_id, user.name
        );
        return if add_bind(&from, user) {
            Ok(Some(()))
        } else {
            Err(())
        }
    }
    Ok(None)
}
