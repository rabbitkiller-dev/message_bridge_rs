use std::collections::HashMap;

use lazy_static::lazy_static;
use tokio::sync::Mutex;
use tracing::instrument;

use crate::{
    bridge::{manager::BRIDGE_USER_MANAGER, user::BridgeUser},
    elo,
};

/// 关联组
#[derive(Debug)]
struct Mapping {
    /// 申请者id
    req: String,
    /// 响应者id
    resp: Option<String>,
}

lazy_static! {
    /// # 缓存绑定申请者
    /// - `key` 口令
    /// - `value` 关联组
    static ref CACHE_REQ: Mutex<HashMap<String, Mapping>> = Mutex::new(HashMap::with_capacity(32));
}

/// 根据输入的一对信息元查询桥用户信息
async fn get_pair(a: &str, b: &str) -> (Option<BridgeUser>, Option<BridgeUser>) {
    let tab_user = BRIDGE_USER_MANAGER.lock().await;
    (tab_user.get(a).await, tab_user.get(b).await)
}

/// 检查是否已经绑定
async fn is_bound(a: &str, b: &str) -> bool {
    let (a, b) = get_pair(a, b).await;
    if a.is_none() || b.is_none() {
        return false;
    }
    let (a, b) = (a.unwrap(), b.unwrap());
    if a.ref_id.is_some() && b.ref_id.is_some() && a.ref_id == b.ref_id {
        return true;
    }
    false
}

/// # 添加申请
/// ### Argument
/// `req_user_id` 申请者id
/// ### Return
/// `Ok(..)` 回应口令
#[instrument(skip_all)]
pub async fn add_req(req_user_id: &str) -> Result<String, ()> {
    let cache = &mut CACHE_REQ.lock().await;
    let token = loop {
        let tmp = &uuid::Uuid::new_v4().to_string()[..6];
        if !cache.contains_key(tmp) {
            break tmp.to_string();
        }
    };
    // 移除旧数据
    cache.retain(|_, m| &req_user_id != &m.req);
    cache.insert(
        token.clone(),
        Mapping {
            req: req_user_id.to_string(),
            resp: None,
        },
    );
    tracing::debug!("缓存中的绑定申请数量: {}", cache.len());
    Ok(token)
}

/// # 缓存回应
/// ### Arguments
/// - `token` 口令
/// - `resp_user_id` 回应者id
/// ### Return
/// `Err(..)` 失败描述
#[instrument(skip_all)]
pub async fn update_resp(token: String, resp_user_id: &str) -> Result<(), &'static str> {
    let mut cache = CACHE_REQ.lock().await;
    let mapping = elo!(cache.get_mut(&token) ;; return Err("无效的口令！"));
    if &mapping.req == resp_user_id {
        return Err("不要自引用");
    } else if let Some(old_resp) = &mapping.resp {
        // 查重
        if old_resp == resp_user_id {
            return Ok(());
        } else if is_bound(&mapping.req, resp_user_id).await {
            return Err("您与该账户已经存在关联。如有疑问请联系管理员。");
        }
    }
    mapping.resp = Some(resp_user_id.to_string());
    {
        // trace mapping update
        let upd_mapping = cache.get(&token).unwrap();
        tracing::trace!(?upd_mapping);
    }
    tracing::debug!("缓存中的绑定申请数量: {}", cache.len());
    Ok(())
}

/// # 确认建立关联
/// ### Argument
/// `req_user_id` 申请者信息
/// ### Returnt
/// `Err(..)` 失败描述
#[instrument(skip_all)]
pub async fn confirm_bind(req_user_id: &str) -> Result<(), &'static str> {
    let resp_user_id = {
        let cache = &mut CACHE_REQ.lock().await;
        let mapping = cache.iter().find(|(_, m)| &m.req == req_user_id);
        let Some((token, m)) = mapping else {
            return Err("您未申请绑定，或申请已被重置。");
        };
        if m.resp.is_none() {
            return Err("您的关联申请暂未收获回应！");
        }
        // don't use immut-borrow
        let key = token.clone();
        // take data
        let mapping = cache.remove(&key).unwrap();
        tracing::debug!("缓存中的绑定申请数量: {}", cache.len());
        mapping.resp.unwrap()
    };
    tracing::debug!(a = req_user_id, b = resp_user_id);

    // get bridge user
    let (mut user_a, mut user_b) = {
        let (a, b) = get_pair(req_user_id, &resp_user_id).await;
        if a.is_none() || b.is_none() {
            tracing::warn!("桥用户信息缺失！");
            tracing::warn!("【{req_user_id}】{a:?}");
            tracing::warn!("【{resp_user_id}】{b:?}");
            return Err("关联用户不存在！");
        }
        (a.unwrap(), b.unwrap())
    };
    let tab_user = &mut BRIDGE_USER_MANAGER.lock().await;
    // copy or create ref_id
    let ref_id = if user_a.ref_id.is_some() {
        user_a.ref_id
    } else if user_b.ref_id.is_some() {
        user_b.ref_id
    } else {
        Some(uuid::Uuid::new_v4().to_string())
    };
    user_a.ref_id = ref_id.clone();
    user_b.ref_id = ref_id.clone();

    tracing::trace!(?user_a, ?user_b);
    match tab_user.batch_update(&[user_a, user_b]).await {
        Ok(c) => {
            tracing::info!("{c}行修改成功");
            Ok(())
        }
        Err(e) => {
            tracing::error!("用户关联保存失败！{e}");
            Err("保存失败！")
        }
    }
}

/// # 解除关联
/// ### Arguments
/// - `user_id` 申请者信息
/// - `platform` 指定平台
/// ### Returnt
/// `Err(..)` 失败描述
#[instrument(skip_all)]
pub async fn unbind(user_id: &str, platform: &str) -> Result<(), &'static str> {
    let tab_user = &mut BRIDGE_USER_MANAGER.lock().await;
    let ref_id = {
        let Some(user) = &tab_user.get(user_id).await else {
            tracing::warn!("找不到 id 为【{user_id}】的桥用户！");
            return Err("获取用户信息失败");
        };
        elo!(user.ref_id.clone() ;; return Ok(()))
    };

    let Some(mut target) = tab_user.findByRefAndPlatform(&ref_id, platform).await else {
        return Ok(());
    };
    target.ref_id = None;
    match tab_user.batch_update(&[target]).await {
        Ok(c) => {
            tracing::info!("{c}行修改成功");
            Ok(())
        }
        Err(e) => {
            tracing::error!("保存解除关联失败！{e}");
            Err("操作失败")
        }
    } // match
}
