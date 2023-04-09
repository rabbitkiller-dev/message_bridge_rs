use std::{collections::HashMap, str::FromStr};

use chrono::Local;
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use tracing::instrument;

use crate::{
    bridge::{manager::BRIDGE_USER_MANAGER, user::BridgeUser},
    elo, elr,
};

use super::token::Token;

/// 缓存超时时限（毫秒）
const CACHE_TIMEOUT: i64 = 1000 * 3600 * 24;

/// 关联组
#[derive(Debug, Clone)]
struct Mapping {
    /// 申请者id
    appl: String,
    /// 响应者id
    resp: Option<String>,
    /// 创建时间
    create_time: i64,
}

/// 绑定指令相关的错误
pub enum BindErr {
    /// 已存在关联
    AlreadyMapping,
    /// 不存在于缓存中的口令
    InvalidToken,
    /// 缓存中没有申请记录
    NoApply,
    /// 找不到桥用户
    NotFoundBridgeUser,
    /// 口令不在缓存中
    NotFoundToken,
    /// 申请未收获回应
    NoResponed,
    /// 尝试关联自身
    SelfReference,
    /// 更新桥用户信息失败
    UpdateBridgeUserFailure,
}

lazy_static! {
    /// # 缓存关联申请
    /// - `key` 口令
    /// - `val` 关联组
    static ref MAPPING_SESSION: Mutex<HashMap<u32, Mapping>> = Mutex::new(HashMap::with_capacity(32));
    /// # 速查表：通过申请者 id 定位关联组
    /// - `key` 申请者 id
    /// - `val` 口令
    static ref APPLICANT_DICT: Mutex<HashMap<String, u32>> = Mutex::new(HashMap::with_capacity(32));
}

/// 根据输入的一对 id 查询桥用户信息
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
    let a = a.unwrap();
    // return
    a.ref_id.is_some() && a.ref_id == b.unwrap().ref_id
}

/// 清理超时缓存
async fn clean_overtime_session() {
    let now = Local::now().timestamp();
    MAPPING_SESSION
        .lock()
        .await
        .retain(|_, Mapping { create_time, .. }| *create_time - now < CACHE_TIMEOUT);
}

/// # 创建申请绑定的会话
/// ### Argument
/// `applicant_id` 申请者id
/// ### Return
/// `Ok(..)` 回应口令
#[instrument]
pub async fn create_session(applicant_id: &str) -> Result<String, ()> {
    clean_overtime_session().await;
    let mut cache = MAPPING_SESSION.lock().await;
    let token = loop {
        let tmp = Token::new();
        if !cache.contains_key(&tmp.val()) {
            break tmp;
        }
    };
    // 移除旧数据
    let mut dict = APPLICANT_DICT.lock().await;
    if let Some(old_token) = dict.get_mut(applicant_id) {
        cache.remove(old_token);
        *old_token = token.val();
    }
    cache.insert(
        token.val(),
        Mapping {
            appl: applicant_id.to_string(),
            resp: None,
            create_time: Local::now().timestamp(),
        },
    );
    tracing::debug!("缓存中的申请数量: {}", cache.len());
    Ok(token.to_string())
}

/// # 缓存回应
/// ### Arguments
/// - `token` 口令
/// - `resp_user_id` 回应者id
/// ### Return
/// `Err(..)` 失败描述
#[instrument]
pub async fn update_resp(token: &str, resp_user_id: &str) -> Result<(), BindErr> {
    use BindErr::*;
    clean_overtime_session().await;
    let token = elr!(Token::from_str(token) ;; e -> {
        tracing::info!("用户输入的口令不合法！{e:?}");
        return Err(BindErr::InvalidToken);
    });
    let mut cache = MAPPING_SESSION.lock().await;
    let Mapping { appl, resp, .. } = elo!(cache.get_mut(&token.val()) ;; return Err(NotFoundToken));
    if appl == resp_user_id {
        return Err(SelfReference);
    } else if let Some(old_resp) = resp {
        // 查重
        if old_resp == resp_user_id {
            return Ok(());
        } else if is_bound(appl, resp_user_id).await {
            return Err(AlreadyMapping);
        }
    }
    *resp = Some(resp_user_id.to_string());
    Ok(())
}

/// # 确认建立关联
/// ### Argument
/// `applicant_id` 申请者信息
/// ### Returnt
/// `Err(..)` 失败描述
#[instrument(skip_all)]
pub async fn confirm_bind(applicant_id: &str) -> Result<(), BindErr> {
    use BindErr::*;
    let mut dict = APPLICANT_DICT.lock().await;
    let mut cache = MAPPING_SESSION.lock().await;

    let token = *elo!(dict.get(applicant_id) ;; return Err(BindErr::NoApply));
    let Mapping { resp, .. } = elo!(cache.get(&token) ;; return Err(BindErr::NoApply));
    let resp_user_id = elo!(resp ;; return Err(NoResponed));
    tracing::debug!(applicant_id, resp_user_id);
    // get bridge user
    let (mut user_a, mut user_b) = {
        let (a, b) = get_pair(applicant_id, resp_user_id).await;
        if a.is_none() || b.is_none() {
            tracing::warn!("桥用户信息缺失！");
            tracing::warn!("【{applicant_id}】{a:?}");
            tracing::warn!("【{resp_user_id}】{b:?}");
            return Err(NotFoundBridgeUser);
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
        Some(applicant_id.to_string())
    };
    user_a.ref_id = ref_id.clone();
    user_b.ref_id = ref_id.clone();

    match tab_user.batch_update(&[user_a, user_b]).await {
        Ok(c) => {
            tracing::info!("{c}行修改成功");
            dict.remove(applicant_id);
            cache.remove(&token);
            Ok(())
        }
        Err(e) => {
            tracing::error!("用户关联保存失败！{e}");
            Err(UpdateBridgeUserFailure)
        }
    }
}

/// # 解除关联
/// ### Arguments
/// - `user_id` 指令使用者信息
/// - `platform` 指定平台
/// ### Returnt
/// `Err(..)` 失败描述
#[instrument(skip_all)]
pub async fn unbind(user_id: &str, platform: &str) -> Result<(), BindErr> {
    let mut tab_user = BRIDGE_USER_MANAGER.lock().await;
    let Some(user) = tab_user.get(user_id).await else {
        tracing::warn!("找不到 id 为【{user_id}】的桥用户！");
        return Err(BindErr::NotFoundBridgeUser);
    };
    let ref_id = elo!(user.ref_id ;; return Ok(()));
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
            Err(BindErr::UpdateBridgeUserFailure)
        }
    } // match
}

impl std::fmt::Display for BindErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use BindErr::*;
        match self {
            AlreadyMapping => write!(f, "您与该账户已经存在关联。"),
            InvalidToken | NotFoundToken => write!(f, "无效的口令！"),
            NoApply => write!(f, "您未申请绑定，或申请已被重置。"),
            NotFoundBridgeUser => write!(f, "获取用户信息失败！"),
            NoResponed => write!(f, "您的关联申请暂未收获回应。"),
            SelfReference => write!(f, "自引用操作无效！"),
            UpdateBridgeUserFailure => write!(f, "更新关联失败！"),
        }
    }
}
