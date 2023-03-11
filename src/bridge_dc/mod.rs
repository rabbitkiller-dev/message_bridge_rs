use std::path::Path;
use std::sync::Arc;

use serenity::builder::CreateButton;
use serenity::http::Http;
use serenity::model::application::component::ButtonStyle;
use serenity::model::channel::AttachmentType;
use serenity::model::guild::Member;
use serenity::model::webhook::Webhook;
use serenity::prelude::*;
use tracing::{debug, error, info, instrument, warn};

use crate::bridge::pojo::BridgeMessagePO;
use crate::bridge::user::BridgeUser;
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
        let guild_id = webhook.guild_id.unwrap();

        // 组装dc消息
        let mut content: Vec<String> = Vec::new();
        let mut reply_content: Vec<String> = Vec::new();
        let mut reply_message_id = "".to_string();
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
                bridge::MessageContent::Reply { id } => {
                    if let Some(id) = id {
                        let reply_message =
                            bridge::BRIDGE_MESSAGE_MANAGER.lock().await.get(id).await;
                        if let Some(reply_message) = reply_message {
                            let refs = reply_message
                                .refs
                                .iter()
                                .find(|refs| refs.platform.eq("DC"));
                            if let Some(refs) = refs {
                                reply_message_id = refs.origin_id.clone();
                            }
                            reply_content = to_reply_content(reply_message).await;
                        } else {
                            content.push("> {回复消息}\n".to_string());
                        }
                    }
                }
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
            .get(&message.sender_id)
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
                // w.components(|c| c.add_action_row());
                w.add_files(fils);
                reply_content.append(&mut content);
                w.content(reply_content.join(""));
                if reply_content.len() > 0 {
                    w.components(|c| {
                        c.create_action_row(|row| {
                            let mut button = CreateButton::default();
                            button.style(ButtonStyle::Link);
                            button.url(format!(
                                "https://discord.com/channels/{}/{}/{}",
                                guild_id, message.bridge_config.discord.channelId, reply_message_id
                            ));
                            button.label("跳转回复");
                            row.add_button(button)
                        })
                    });
                }
                println!("add_button: {:?}", w);
                // w.content(content.join(""));
                // .content(content.join("")).components(f).content(content.join(""))
                w
            })
            .await;

        match resp {
            Ok(result) => {
                if let Some(msg) = result {
                    // 发送成功后, 将平台消息和桥消息进行关联, 为以后进行回复功能
                    bridge::BRIDGE_MESSAGE_MANAGER
                        .lock()
                        .await
                        .ref_bridge_message(bridge::pojo::BridgeMessageRefMessageForm {
                            bridge_message_id: message.id.clone(),
                            platform: "DC".to_string(),
                            origin_id: msg.id.0.to_string(),
                        })
                        .await;
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
    // 创建机器人的webhook, 能够发送交互组件消息
    // let webhook = http.create_webhook(channel_id, &map, None).await?;
    // let map = serde_json::json!({"name": "test"});
    // let webhook = client
    //     .cache_and_http
    //     .http
    //     .create_webhook(781347109676384297, &map, None)
    //     .await
    //     .unwrap();
    // let webhook = client
    //     .cache_and_http
    //     .http
    //     .get_webhook(1084186702567981077)
    //     .await
    //     .unwrap();
    // println!("webhook_id: {}", webhook.id);
    webhook
        .execute(&client.cache_and_http.http, false, |w| {
            w.username("Webhook test").components(|c| {
                c.create_action_row(|row| {
                    row.create_button(|b| b.style(ButtonStyle::Link).url("https://discord.com/channels/724829522230378536/781347109676384297/1082716108953497681").label("跳转回复"))
                })
            })
        })
        .await
        .expect("Could not execute webhook.");
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
pub async fn apply_bridge_user(id: u64, name: &str, discriminator: u16) -> BridgeUser {
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

pub async fn to_reply_content(reply_message: BridgeMessagePO) -> Vec<String> {
    let mut content: Vec<String> = vec![];
    let user = match bridge::user_manager::bridge_user_manager
        .lock()
        .await
        .get(&reply_message.sender_id)
        .await
    {
        Some(user) => user.display_text,
        None => {
            format!("[UN] {}", reply_message.sender_id)
        }
    };

    let mut content: String = String::new();
    content.push_str(format!("回复 @{} 的消息\n", user).as_str());
    for chain in reply_message.message_chain {
        match chain {
            bridge::MessageContent::Plain { text } => content.push_str(&text),
            bridge::MessageContent::Image(image) => content.push_str("[图片]"),
            bridge::MessageContent::Reply { id } => content.push_str("[回复消息]"),
            bridge::MessageContent::At { id } => {
                let bridge_user = bridge::user_manager::bridge_user_manager
                    .lock()
                    .await
                    .get(&id)
                    .await;
                if let None = bridge_user {
                    content.push_str(format!("@[UN] {}", id).as_str());
                    continue;
                }
                let bridge_user = bridge_user.unwrap();
                // 查看桥关联的本平台用户id
                if let Some(ref_user) = bridge_user.findRefByPlatform("DC").await {
                    content.push_str(format!("@{}", ref_user.to_string()).as_str());
                    continue;
                }
                // 没有关联账号用标准格式发送消息
                content.push_str(format!("@{}", bridge_user.to_string()).as_str());
            }
            _ => warn!(unit = ?chain, "无法识别的MessageChain"),
        };
    }
    let mut result: Vec<String> = vec![];
    let splis: Vec<&str> = content.split("\n").collect();
    for sp in splis {
        result.push(format!("> {}\n", sp));
    }
    // result.push(value)
    result
}
