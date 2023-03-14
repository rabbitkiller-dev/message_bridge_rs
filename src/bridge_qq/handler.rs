//! 负责处理 qq 消息

use std::sync::Arc;

use proc_qq::re_exports::async_trait::async_trait;
use proc_qq::re_exports::ricq_core;
use proc_qq::re_exports::ricq_core::msg::elem;
use proc_qq::{
    FriendMessageEvent, GroupMessageEvent, GroupTempMessageEvent, LoginEventProcess,
    MessageChainPointTrait, MessageEvent, MessageEventProcess,
};
use tracing::{debug, error, info};

use crate::bridge::{BridgeClient, Image, MessageContent};
use crate::config::BridgeConfig;
use crate::{bridge, elo, utils, Config};

use super::group_message_id::GroupMessageId;
use super::{apply_bridge_user, RqClient};

const OKK: anyhow::Result<bool> = Ok(true);

async fn recv_group_msg(
    event: &GroupMessageEvent,
    config: &BridgeConfig,
    bridge: Arc<BridgeClient>,
) -> anyhow::Result<bool> {
    let mut _pass = true;
    let msg = &event.inner;
    let group_id = msg.group_code as u64;
    let sender_id = msg.from_uin as u64;
    let sender_nickname = msg.group_card.clone();
    info!(
        "[{}]{group_id}-[{sender_nickname}]{sender_id} '{}'",
        msg.group_name, msg.elements
    );
    // 为发送者申请桥用户
    let bridge_user = apply_bridge_user(sender_id, sender_nickname.as_str()).await;
    // 并接该群消息的id
    let qq_message_id = GroupMessageId::new(group_id, msg.seqs.get(0).unwrap().clone());
    // 组装向桥发送的消息体表单
    let mut bridge_message = bridge::pojo::BridgeSendMessageForm {
        sender_id: bridge_user.id,
        avatar_url: Some(format!("https://q1.qlogo.cn/g?b=qq&nk={sender_id}&s=100")),
        bridge_config: config.clone(),
        message_chain: Vec::new(),
        origin_message: bridge::pojo::BridgeMessageRefPO {
            origin_id: qq_message_id.to_string(),
            platform: "QQ".to_string(),
        },
    };

    for chain1 in &event.message_chain().0 {
        let chain = elem::RQElem::from(chain1.clone());
        match chain {
            elem::RQElem::At(at) => {
                debug!("RQElem::At: {:?}", at);
                let bridge_user = apply_bridge_user(
                    at.target as u64,
                    elo!(at.display.strip_prefix("@") ;; continue),
                )
                .await;
                bridge_message
                    .message_chain
                    .push(MessageContent::At { id: bridge_user.id });
            }
            elem::RQElem::Text(text) => {
                debug!("RQElem::Text: {:?}", text);
                bridge_message
                    .message_chain
                    .push(MessageContent::Plain { text: text.content });
            }
            elem::RQElem::GroupImage(group_image) => {
                debug!("group_image: {:?}", group_image);
                debug!("group_image2: {:?}", group_image.url());
                let file_path = match utils::download_and_cache(group_image.url().as_str()).await {
                    Ok(path) => Some(path),
                    Err(_) => {
                        tracing::error!("下载图片失败: {:?}", group_image.url());
                        None
                    }
                };
                bridge_message.message_chain.push(MessageContent::Image(
                    if let Some(path) = file_path {
                        Image::Path(path)
                    } else {
                        Image::Url(group_image.url())
                    },
                ));
            }
            elem::RQElem::Other(o) => {
                if let ricq_core::pb::msg::elem::Elem::SrcMsg(source_msg) = *o {
                    debug!("疑似回复消息 id: {:?}", source_msg);
                    let seqs = source_msg.orig_seqs.first().unwrap().clone();
                    let group_message_id = GroupMessageId::new(source_msg.to_uin() as u64, seqs);
                    let result = bridge::manager::BRIDGE_MESSAGE_MANAGER
                        .lock()
                        .await
                        .find_by_ref_and_platform(group_message_id.to_string().as_str(), "QQ")
                        .await;
                    if let Err(err) = result {
                        bridge_message
                            .message_chain
                            .push(MessageContent::Err { message: err });
                        continue;
                    }
                    let result = result.unwrap();
                    if let Some(reply) = result {
                        // 这条是一个笨逻辑, qq的回复会自动at, 这里把他去掉
                        bridge_message.message_chain.pop();
                        bridge_message.message_chain.pop();
                        // 填入回复的消息
                        bridge_message.message_chain.push(MessageContent::Reply {
                            id: Some(reply.id.clone()),
                        });
                        continue;
                    }
                    bridge_message.message_chain.push(MessageContent::Err {
                        message: "回复一条QQ消息, 但是同步回复消息失败".to_string(),
                    });
                } else {
                    debug!("未解读 elem: {:?}", o);
                }
            }
            o => {
                debug!("未处理 elem: {:?}", o);
                bridge_message.message_chain.push(MessageContent::Plain {
                    text: "[未处理]".to_string(),
                });
            }
        }
    }
    bridge.send_message(bridge_message).await;
    OKK
}

/// 陌生人、群成员临时会话
async fn recv_tmp_msg(event: &GroupTempMessageEvent) -> anyhow::Result<bool> {
    debug!("tmp session msg: {:?}", event.inner);
    // TODO proc tmp session msg
    OKK
}

/// 好友消息
async fn recv_friend_msg(event: &FriendMessageEvent) -> anyhow::Result<bool> {
    debug!("friend msg: {:?}", event.inner);
    // TODO proc friend msg
    OKK
}

pub struct DefaultHandler {
    pub config: Arc<Config>,
    pub bridge: Arc<BridgeClient>,
    pub origin_client: Option<Arc<RqClient>>,
}
impl DefaultHandler {
    fn find_cfg_by_group(&self, group_id: u64) -> Option<&BridgeConfig> {
        let bridge_config = self
            .config
            .bridges
            .iter()
            .find(|b| group_id == b.qqGroup && b.enable);
        Some(bridge_config?)
    }
}
#[async_trait]
impl MessageEventProcess for DefaultHandler {
    async fn handle(&self, event: &MessageEvent) -> anyhow::Result<bool> {
        let res = match event {
            MessageEvent::FriendMessage(e) => recv_friend_msg(e).await,
            MessageEvent::GroupTempMessage(e) => recv_tmp_msg(e).await,
            MessageEvent::GroupMessage(group_msg_event) => {
                let gid = group_msg_event.inner.group_code as u64;
                debug!("收到群消息({gid})");
                // 如果频道没有配置桥, 则忽略消息
                let Some(bridge_cfg) = self.find_cfg_by_group(gid) else {
                    info!("群({gid})未启用消息同步");
                    return OKK;
                };
                recv_group_msg(group_msg_event, bridge_cfg, self.bridge.clone()).await
            }
        };
        match res {
            Ok(flag) => Ok(flag),
            Err(err) => {
                let bot_id = event.client().uin().await;
                error!(?err, "[{bot_id}] 消息处理时异常");
                Ok(false)
            }
        }
        // return
    }
}
#[async_trait]
impl LoginEventProcess for DefaultHandler {
    async fn handle(&self, _: &proc_qq::LoginEvent) -> anyhow::Result<bool> {
        tracing::info!("[QQ] 登录到qq客户端");
        OKK
    }
}
