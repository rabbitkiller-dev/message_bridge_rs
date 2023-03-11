use std::path::Path;
use std::sync::Arc;

use serenity::http::Http;
use serenity::model::channel::AttachmentType;
use serenity::model::guild::Member;
use serenity::model::webhook::Webhook;
use serenity::prelude::*;
use tracing::{debug, error, info, instrument, warn};

use crate::bridge::Image;
// use crate::bridge_message_history::{BridgeMessageHistory, Platform};
use crate::{bridge, bridge_dc, Config};

pub mod handler;

pub use handler::*;
use proc_qq::re_exports::image;

/**
 *
 */
#[instrument(name = "bridge_dc_sync", skip_all)]
pub async fn dc(bridge: Arc<bridge::BridgeClient>, http: Arc<Http>) {
    let mut subs = bridge.sender.subscribe();
    loop {
        let message = &subs.recv().await.unwrap();
        info!("收到桥的消息, 同步到discord上");
        let webhook = Webhook::from_id_with_token(
            &http,
            message.bridge_config.discord.id,
            message.bridge_config.discord.token.as_str(),
        )
        .await
        .unwrap();
        debug!("discord info: {:#?}", webhook);
        let _guild_id = webhook.guild_id.unwrap();

        // 组装dc消息
        let mut content: Vec<String> = Vec::new();
        let mut fils: Vec<AttachmentType> = Vec::new();
        for chain in &message.message_chain {
            match chain {
                bridge::MessageContent::Plain { text } => content.push(text.clone()),
                bridge::MessageContent::Image(image) => match image {
                    Image::Url(url) => {
                        fils.push(AttachmentType::Image(url::Url::parse(url).unwrap()))
                    }
                    Image::Path(path) => fils.push(AttachmentType::Path(Path::new(path))),
                    Image::Buff(data) => {
                        match image::guess_format(data) {
                            Ok(format) => fils.push(AttachmentType::Bytes {
                                data: data.into(),
                                filename: format!("file.{}", format.extensions_str()[0]),
                            }),
                            Err(_) => {}
                        };
                    }
                },
                bridge::MessageContent::At { id } => {
                    let bridge_user = bridge::user_manager::bridge_user_manager
                        .lock()
                        .await
                        .get(id)
                        .await;
                    if let None = bridge_user {
                        content.push(format!("@[UN] {}", id));
                        continue;
                    }
                    let bridge_user = bridge_user.unwrap();
                    // 查看桥关联的本平台用户id
                    if let Some(ref_user) = bridge_user.findRefByPlatform("DC").await {
                        content.push(format!("<@{}>", ref_user.origin_id));
                        continue;
                    }
                    // 没有关联账号用标准格式发送消息
                    content.push(format!("@{}", bridge_user.to_string()));
                    // trace!("用户'{}'收到@", username);
                    // let re = Regex::new(r"@\[DC\] ([^\n]+)?#(\d\d\d\d)").unwrap();
                    // let caps = re.captures(username);
                    // match caps {
                    //     Some(caps) => {
                    //         let name = caps.get(1).unwrap();
                    //         let dis = caps.get(2).unwrap();
                    //         let member =
                    //             find_member_by_name(&http, guild_id.0, name.as_str(), dis.as_str())
                    //                 .await;
                    //         if let Some(member) = member {
                    //             content.push(format!("<@{}>", member.user.id.0));
                    //         } else {
                    //             content.push(username.clone());
                    //             warn!("@用户无法解析: {}", username);
                    //         }
                    //     }
                    //     None => {
                    //         content.push(username.clone());
                    //         warn!("@用户无法解析: {}", username);
                    //     }
                    // }
                }
                _ => warn!(unit = ?chain, "无法识别的MessageChain"),
            };
        }
        debug!(?content, ?fils, "桥内消息链组装完成");
        let bridge_user = bridge::user_manager::bridge_user_manager
            .lock()
            .await
            .get(&message.bridge_user_id)
            .await
            .unwrap();
        let resp = webhook
            .execute(&http, true, |w| {
                // 配置发送者头像
                if let Some(url) = &message.avatar_url {
                    w.avatar_url(url.as_str());
                }
                debug!("消息头像url：{:?}", message.avatar_url);
                // 配置发送者用户名
                w.username(bridge_user.display_text);
                if content.len() == 0 && fils.len() == 0 {
                    content.push("{本次发送的消息没有内容}".to_string());
                }
                w.add_files(fils).content(content.join(""))
            })
            .await;

        match resp {
            Ok(result) => {
                if let Some(_msg) = result {
                    // BridgeMessageHistory::insert(
                    //     &message.id,
                    //     Platform::Discord,
                    //     msg.id.0.to_string().as_str(),
                    // )
                    // .await
                    // .unwrap();
                } else {
                    error!("同步的消息没有返回消息id")
                }
                info!("已同步消息")
            }
            Err(err) => {
                error!(?err, "消息同步失败！")
            }
        }
    }
}

pub async fn start(config: Arc<Config>, bridge: Arc<bridge::BridgeClient>) {
    tracing::info!("[DC] 初始化DC桥");
    let token = &config.discord_config.botToken;
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(bridge_dc::Handler {
            config: config.clone(),
            bridge: bridge.clone(),
        })
        .await
        .expect("Err creating client");

    let cache = client.cache_and_http.clone();

    tokio::select! {
        _ = client.start() => {
            tracing::warn!("[DC] Discord客户端退出");
        },
        _ = dc(bridge.clone(), cache.http.clone()) => {
            tracing::warn!("[DC] Discord桥关闭");
        },
    }
}

/**
 * 申请桥用户
 */
pub async fn apply_bridge_user(
    id: u64,
    name: &str,
    discriminator: u16,
) -> bridge::user::BridgeUser {
    let bridge_user = bridge::user_manager::bridge_user_manager
        .lock()
        .await
        .likeAndSave(bridge::pojo::BridgeUserSaveForm {
            origin_id: id.to_string(),
            platform: "DC".to_string(),
            display_text: format!("{}#{}", name, discriminator),
        })
        .await;
    bridge_user.unwrap()
}

/**
 * 通过名称和discriminator查询成员
 */
#[instrument(level = "debug", skip(http), ret)]
pub async fn find_member_by_name(
    http: &Http,
    guild_id: u64,
    nickname: &str,
    discriminator: &str,
) -> Option<Member> {
    let members = http.get_guild_members(guild_id, None, None).await.unwrap();
    let member = members.into_iter().find(|member| {
        member.user.name == nickname && member.user.discriminator.to_string() == discriminator
    });
    member
}
