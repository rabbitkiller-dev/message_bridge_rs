use std::sync::Arc;

use anyhow::Ok;
use proc_qq::re_exports::async_trait::async_trait;
use proc_qq::re_exports::ricq::msg::MessageChain;
use proc_qq::re_exports::ricq::version::ANDROID_WATCH;
use proc_qq::re_exports::ricq_core::msg::elem;
use proc_qq::{
    Authentication, ClientBuilder, DeviceSource, LoginEventProcess, MessageChainPointTrait,
    MessageEvent, MessageEventProcess, ModuleEventHandler, ModuleEventProcess, ShowQR,
};

use crate::bridge::BridgeClientPlatform;
use crate::{bridge, utils, Config};

pub async fn upload_group_image(
    group_id: u64,
    url: &str,
    rq_client: Arc<proc_qq::re_exports::ricq::Client>,
) -> anyhow::Result<elem::GroupImage> {
    let client = reqwest::Client::new();
    let stream = client.get(url).send().await?;
    let img_bytes = stream.bytes().await.unwrap();
    let group_image = rq_client
        .upload_group_image(group_id as i64, img_bytes.to_vec())
        .await
        .unwrap();
    Ok(group_image)
}

/**
 * 同步消息方法
 */
pub async fn sync_message(
    bridge: Arc<bridge::BridgeClient>,
    rq_client: Arc<proc_qq::re_exports::ricq::Client>,
) {
    let mut subs = bridge.sender.subscribe();

    loop {
        let message = &subs.recv().await.unwrap();

        let mut send_content = MessageChain::default();
        // 配置发送者头像
        if let Some(avatar_url) = &message.user.avatar_url {
            tracing::debug!("用户头像: {:?}", message.user.avatar_url);
            let image =
                upload_group_image(message.bridge_config.qqGroup, avatar_url, rq_client.clone())
                    .await;
            if let Result::Ok(image) = image {
                send_content.push(image);
            }
        }
        // 配置发送者用户名
        send_content.push(elem::Text::new(format!("{}\n", message.user.name)));

        for chain in &message.message_chain {
            match chain {
                // 桥文本 转 qq文本
                bridge::MessageContent::Plain { text } => {
                    send_content.push(elem::Text::new(text.to_string()))
                }
                // @桥用户 转 @qq用户 或 @文本
                bridge::MessageContent::At { id } => {
                    let bridge_user = bridge::user_manager::bridge_user_manager
                        .lock()
                        .await
                        .get(id)
                        .await;
                    if let None = bridge_user {
                        send_content.push(elem::Text::new(format!("@[UN] {}", id)));
                        continue;
                    }
                    let bridge_user = bridge_user.unwrap();
                    // 查看桥关联的本平台用户id
                    if let Some(ref_user) = bridge_user.findRefByPlatform("QQ").await {
                        let origin_id = ref_user.origin_id.parse::<i64>().unwrap();
                        send_content.push(elem::At::new(origin_id));
                        continue;
                    }
                    // 没有关联账号用标准格式发送消息
                    send_content.push(elem::Text::new(format!(
                        "@[{}] {}",
                        bridge_user.platform, bridge_user.display_text
                    )))
                }
                // 桥图片 转 qq图片
                bridge::MessageContent::Image { url, path } => {
                    tracing::debug!("桥消息-图片: {:?}", message.user.avatar_url);
                    if let Some(url) = url {
                        let image = upload_group_image(
                            message.bridge_config.qqGroup,
                            url,
                            rq_client.clone(),
                        )
                        .await;
                        if let Result::Ok(image) = image {
                            send_content.push(image);
                        }
                    };
                    if let Some(_) = path {};
                }
                _ => send_content.push(elem::Text::new("{未处理的桥信息}".to_string())),
            }
        }
        tracing::debug!("[QQ] 同步消息");
        tracing::debug!("{:?}", send_content);
        tracing::debug!("{:?}", message.bridge_config.qqGroup as i64);
        let result = rq_client
            .send_group_message(message.bridge_config.qqGroup as i64, send_content)
            .await
            .ok();
        tracing::debug!("{:?}", result);
    }
}

/**
 * 消息桥构建入口
 */
pub async fn start(config: Arc<Config>, bridge: Arc<bridge::BridgeClient>) {
    tracing::info!("[QQ] 初始化QQ桥");
    let handler = Handler {
        config: config.clone(),
        bridge: bridge.clone(),
        origin_client: None,
    };
    let handler = Box::new(handler);
    let on_message = ModuleEventHandler {
        name: "OnMessage".to_owned(),
        process: ModuleEventProcess::Message(handler),
    };

    // let modules = module!("qq_bridge", "qq桥模块", handler);
    let module = proc_qq::Module {
        id: "qq_bridge".to_string(),
        name: "qq桥模块".to_string(),
        handles: vec![on_message],
    };

    let client = ClientBuilder::new()
        .priority_session("session.token")
        .authentication(Authentication::QRCode)
        .show_rq(ShowQR::OpenBySystem)
        .device(DeviceSource::JsonFile("device.json".to_owned()))
        .version(&ANDROID_WATCH)
        .modules(vec![module])
        .build()
        .await
        .unwrap();
    // let arc = Arc::new(client);
    let rq_client = client.rq_client.clone();
    tokio::select! {
        _ = client.start() => {
            tracing::warn!("[QQ] QQ客户端退出");
        },
        _ = sync_message(bridge.clone(), rq_client) => {
            tracing::warn!("[QQ] QQ桥关闭");
        },
    }
}

pub struct Handler {
    pub config: Arc<Config>,
    pub bridge: Arc<bridge::BridgeClient>,
    pub origin_client: Option<Arc<proc_qq::re_exports::ricq::Client>>,
}

#[async_trait]
impl MessageEventProcess for Handler {
    async fn handle(&self, event: &MessageEvent) -> anyhow::Result<bool> {
        if let MessageEvent::GroupMessage(group_message) = event {
            // 查询这个频道是否需要通知到群
            let group_id = group_message.inner.group_code as u64;
            let sender_id = group_message.inner.from_uin as u64;
            let sender_nickname = group_message.inner.group_card.clone();
            let bridge_config = match self
                .config
                .bridges
                .iter()
                .find(|bridge| group_id == bridge.qqGroup && bridge.enable)
            {
                Some(bridge_config) => bridge_config,
                // 该消息的频道没有配置桥, 忽略这个消息
                None => return Ok(true),
            };
            let _bridge_user = apply_bridge_user(sender_id, sender_nickname.as_str()).await;
            let user = bridge::User {
                name: format!(
                    "[QQ] {}({})",
                    "".to_string(),
                    sender_id // group_message.sender.member_name.to_string(),
                              // group_message.sender.id
                ),
                avatar_url: Some(format!("https://q1.qlogo.cn/g?b=qq&nk={}&s=100", sender_id)),
                unique_id: sender_id,
                platform: BridgeClientPlatform::QQ,
                display_id: sender_id,
                platform_id: group_id,
            };
            let mut bridge_message = bridge::BridgeMessage {
                id: uuid::Uuid::new_v4().to_string(),
                bridge_config: bridge_config.clone(),
                message_chain: Vec::new(),
                user,
            };

            for chain1 in &group_message.message_chain().0 {
                let chain = elem::RQElem::from(chain1.clone());
                match chain {
                    elem::RQElem::At(at) => {
                        tracing::debug!("RQElem::At: {:?}", at);
                        let target = at.target as u64;
                        let name = at.display.strip_prefix("@").unwrap();
                        let bridge_user = apply_bridge_user(target, name).await;
                        bridge_message
                            .message_chain
                            .push(bridge::MessageContent::At { id: bridge_user.id });
                    }
                    elem::RQElem::Text(text) => {
                        tracing::debug!("RQElem::Text: {:?}", text);
                        bridge_message
                            .message_chain
                            .push(bridge::MessageContent::Plain { text: text.content });
                    }
                    elem::RQElem::Face(face) => {
                        tracing::debug!("RQElem::Face: {:?}", face);
                    }
                    // elem::RQElem::MarketFace(_) => todo!(),
                    // elem::RQElem::Dice(_) => todo!(),
                    // elem::RQElem::FingerGuessing(_) => todo!(),
                    // elem::RQElem::LightApp(_) => todo!(),
                    // elem::RQElem::RichMsg(_) => todo!(),
                    // elem::RQElem::FriendImage(_) => todo!(),
                    elem::RQElem::GroupImage(group_image) => {
                        tracing::debug!("group_image: {:?}", group_image);
                        tracing::debug!("group_image2: {:?}", group_image.url());
                        let file_path =
                            match utils::download_and_cache(group_image.url().as_str()).await {
                                std::result::Result::Ok(path) => Some(path),
                                Err(_) => {
                                    tracing::error!("下载图片失败: {:?}", group_image.url());
                                    None
                                }
                            };
                        // let base64 = image_base64::to_base64(path.as_str());
                        bridge_message
                            .message_chain
                            .push(bridge::MessageContent::Image {
                                url: Some(group_image.url()),
                                path: file_path,
                            })
                        // bridge_message
                        //     .message_chain
                        //     .push(bridge::MessageContent::Image {
                        //         // url: Some(format!(
                        //         //     "https://gchat.qpic.cn/{}",
                        //         //     custom_face.thumb_url()
                        //         // )),
                        //         url: Some(format!("{}", group_image.url())),
                        //         path: None,
                        //     });
                    }
                    // elem::RQElem::FlashImage(_) => todo!(),
                    // elem::RQElem::VideoFile(_) => todo!(),
                    elem::RQElem::Other(o) => {
                        tracing::debug!("未处理1 MessageChain: {:?}", o);
                    }
                    o => {
                        tracing::debug!("未处理2 MessageChain: {:?}", o);
                        bridge_message
                            .message_chain
                            .push(bridge::MessageContent::Plain {
                                text: "[未处理]".to_string(),
                            });
                    }
                }
                // match chain {
                //     elem::At() => {

                //     }
                //     proc_qq::re_exports::ricq_core::pb::msg::elem::Elem::Text(text) => {
                //         if !text.attr6_buf().is_empty() {
                //             let at = elem::RQElem::At(elem::At::from(text));
                //         } else {
                //             bridge_message
                //                 .message_chain
                //                 .push(bridge::MessageContent::Plain {
                //                     text: text.str().to_string(),
                //                 });
                //         }
                //     }
                //     // proc_qq::re_exports::ricq_core::pb::msg::elem::Elem::Face(_) => todo!(),
                //     // proc_qq::re_exports::ricq_core::pb::msg::elem::Elem::OnlineImage(_) => todo!(),
                //     // proc_qq::re_exports::ricq_core::pb::msg::elem::Elem::NotOnlineImage(_) => todo!(),
                //     // proc_qq::re_exports::ricq_core::pb::msg::elem::Elem::TransElemInfo(_) => todo!(),
                //     // proc_qq::re_exports::ricq_core::pb::msg::elem::Elem::MarketFace(_) => todo!(),
                //     proc_qq::re_exports::ricq_core::pb::msg::elem::Elem::CustomFace(
                //         custom_face,
                //     ) => {
                //         bridge_message
                //             .message_chain
                //             .push(bridge::MessageContent::Image {
                //                 url: Some(format!(
                //                     "https://gchat.qpic.cn/{}",
                //                     custom_face.thumb_url()
                //                 )),
                //                 path: None,
                //             });
                //     }
                //     proc_qq::re_exports::ricq_core::pb::msg::elem::Elem::ElemFlags2(_) => {}
                //     // proc_qq::re_exports::ricq_core::pb::msg::elem::Elem::RichMsg(_) => todo!(),
                //     // proc_qq::re_exports::ricq_core::pb::msg::elem::Elem::GroupFile(_) => todo!(),
                //     // proc_qq::re_exports::ricq_core::pb::msg::elem::Elem::ExtraInfo(_) => todo!(),
                //     // proc_qq::re_exports::ricq_core::pb::msg::elem::Elem::VideoFile(_) => todo!(),
                //     // proc_qq::re_exports::ricq_core::pb::msg::elem::Elem::AnonGroupMsg(_) => todo!(),
                //     // proc_qq::re_exports::ricq_core::pb::msg::elem::Elem::QqWalletMsg(_) => todo!(),
                //     // proc_qq::re_exports::ricq_core::pb::msg::elem::Elem::CustomElem(_) => todo!(),
                //     proc_qq::re_exports::ricq_core::pb::msg::elem::Elem::GeneralFlags(_) => {}
                //     // proc_qq::re_exports::ricq_core::pb::msg::elem::Elem::SrcMsg(_) => todo!(),
                //     // proc_qq::re_exports::ricq_core::pb::msg::elem::Elem::LightApp(_) => todo!(),
                //     // proc_qq::re_exports::ricq_core::pb::msg::elem::Elem::CommonElem(_) => todo!(),
                //     o => {
                //         tracing::debug!("未处理MessageChain: {:?}", o);
                //         bridge_message
                //             .message_chain
                //             .push(bridge::MessageContent::Plain {
                //                 text: "[未处理]".to_string(),
                //             });
                //     }
                // }
            }

            self.bridge.send(bridge_message);
        }

        Ok(true)
    }
}

/**
 * 申请桥用户
 */
async fn apply_bridge_user(id: u64, name: &str) -> bridge::user::BridgeUser {
    let bridge_user = bridge::user_manager::bridge_user_manager
        .lock()
        .await
        .likeAndSave(bridge::pojo::BridgeUserSaveForm {
            origin_id: id.to_string(),
            platform: "QQ".to_string(),
            display_text: format!("{}({})", name, id),
        })
        .await;
    bridge_user.unwrap()
}

#[async_trait]
impl LoginEventProcess for Handler {
    async fn handle(&self, _: &proc_qq::LoginEvent) -> anyhow::Result<bool> {
        tracing::info!("[QQ] 登录到qq客户端");
        Ok(true)
    }
}
