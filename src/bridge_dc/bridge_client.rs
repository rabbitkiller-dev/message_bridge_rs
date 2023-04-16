use std::path::Path;
use std::sync::Arc;

use proc_qq::re_exports::image;
use serenity::builder::CreateButton;
use serenity::http::Http;
use serenity::model::application::component::ButtonStyle;
use serenity::model::channel::AttachmentType;
use serenity::model::webhook::Webhook;

use crate::bridge;

use super::{find_member_by_name, parse_text_mention_rule, to_reply_content, MentionText};

#[tracing::instrument(name = "bridge_dc_sync", skip_all)]
pub async fn listen(bridge: Arc<bridge::BridgeClient>, http: Arc<Http>) {
    let mut subs = bridge.sender.subscribe();
    loop {
        let message = &subs.recv().await.unwrap();
        tracing::info!("收到桥的消息, 同步到discord上");
        let webhook = Webhook::from_id_with_token(
            &http,
            message.bridge_config.discord.id,
            message.bridge_config.discord.token.as_str(),
        )
        .await
        .unwrap();
        tracing::debug!("discord info: {:#?}", webhook);
        let guild_id = webhook.guild_id.unwrap();

        // 组装dc消息
        let mut content: Vec<String> = Vec::new();
        let mut reply_content: Vec<String> = Vec::new();
        let mut reply_message_id = "".to_string();
        let mut fils: Vec<AttachmentType> = Vec::new();
        for chain in &message.message_chain {
            match chain {
                bridge::MessageContent::Plain { text } => {
                    let mention_text_list = parse_text_mention_rule(text.to_string());
                    for mention_text in mention_text_list {
                        match mention_text {
                            MentionText::Text(text) => content.push(text),
                            MentionText::MentionText { name, discriminator } => {
                                let member = find_member_by_name(&http, guild_id.0, &name, &discriminator).await;
                                if let Some(member) = member {
                                    content.push(format!("<@{}>", member.user.id.0));
                                } else {
                                    content.push(format!("@[DC] {}#{}", name, discriminator));
                                }
                            }
                        }
                    }
                }
                bridge::MessageContent::Image(image) => match image {
                    bridge::Image::Url(url) => fils.push(AttachmentType::Image(url::Url::parse(url).unwrap())),
                    bridge::Image::Path(path) => fils.push(AttachmentType::Path(Path::new(path))),
                    bridge::Image::Buff(data) => {
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
                        let reply_message = bridge::manager::BRIDGE_MESSAGE_MANAGER.lock().await.get(id).await;
                        if let Some(reply_message) = reply_message {
                            let refs = reply_message.refs.iter().find(|refs| refs.platform.eq("DC"));
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
                    let bridge_user = bridge::manager::BRIDGE_USER_MANAGER.lock().await.get(id).await;
                    if let None = bridge_user {
                        content.push(format!("@[UN] {}", id));
                        continue;
                    }
                    let bridge_user = bridge_user.unwrap();
                    // 查看桥关联的本平台用户id
                    if let Some(ref_user) = bridge_user.find_by_platform("DC").await {
                        content.push(format!("<@{}>", ref_user.origin_id));
                        continue;
                    }
                    // 没有关联账号用标准格式发送消息
                    content.push(format!("@{}", bridge_user.to_string()));
                }
                _ => tracing::warn!(unit = ?chain, "无法识别的MessageChain"),
            };
        }
        tracing::debug!(?content, ?fils, "桥内消息链组装完成");
        let bridge_user = bridge::manager::BRIDGE_USER_MANAGER
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
                tracing::debug!("消息头像url：{:?}", message.avatar_url);
                // 配置发送者用户名
                w.username(bridge_user.to_string());
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
                    bridge::manager::BRIDGE_MESSAGE_MANAGER
                        .lock()
                        .await
                        .ref_bridge_message(bridge::pojo::BridgeMessageRefMessageForm {
                            bridge_message_id: message.id.clone(),
                            platform: "DC".to_string(),
                            origin_id: msg.id.0.to_string(),
                        })
                        .await;
                } else {
                    tracing::error!("同步的消息没有返回消息id")
                }
                tracing::info!("已同步消息")
            }
            Err(err) => {
                tracing::error!(?err, "消息同步失败！")
            }
        }
    }
}
