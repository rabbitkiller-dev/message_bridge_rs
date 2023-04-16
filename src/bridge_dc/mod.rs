use std::sync::Arc;

use serenity::http::Http;
use serenity::model::guild::Member;
use serenity::prelude::*;
use tracing::{instrument, warn};

use crate::bridge::pojo::BridgeMessagePO;
use crate::bridge::user::BridgeUser;

// use crate::bridge_message_history::{BridgeMessageHistory, Platform};
use crate::{bridge, bridge_dc, Config};

pub mod bridge_client;
pub mod handler;

pub use handler::*;

pub async fn start(config: Arc<Config>, bridge: Arc<bridge::BridgeClient>) {
    tracing::info!("[DC] 初始化DC桥");
    let token = &config.discord_config.botToken;
    let intents =
        GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_MEMBERS | GatewayIntents::DIRECT_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

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
    // webhook
    //     .execute(&client.cache_and_http.http, false, |w| {
    //         w.username("Webhook test").components(|c| {
    //             c.create_action_row(|row| {
    //                 row.create_button(|b| b.style(ButtonStyle::Link).url("https://discord.com/channels/724829522230378536/781347109676384297/1082716108953497681").label("跳转回复"))
    //             })
    //         })
    //     })
    //     .await
    //     .expect("Could not execute webhook.");
    let cache = client.cache_and_http.clone();

    tokio::select! {
        _ = client.start() => {
            tracing::warn!("[DC] Discord客户端退出");
        },
        _ = bridge_client::listen(bridge.clone(), cache.http.clone()) => {
            tracing::warn!("[DC] Discord桥关闭");
        },
    }
}

/**
 * 申请桥用户
 */
pub async fn apply_bridge_user(id: u64, name: &str, discriminator: u16) -> BridgeUser {
    let bridge_user = bridge::manager::BRIDGE_USER_MANAGER
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
pub async fn find_member_by_name(http: &Http, guild_id: u64, nickname: &str, discriminator: &str) -> Option<Member> {
    let members = http.get_guild_members(guild_id, None, None).await.unwrap();
    let member = members
        .into_iter()
        .find(|member| member.user.name == nickname && member.user.discriminator.to_string() == discriminator);
    member
}

/**
 * 将桥消息转化成回复dc的消息
 */
pub async fn to_reply_content(reply_message: BridgeMessagePO) -> Vec<String> {
    let user = match bridge::manager::BRIDGE_USER_MANAGER
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
            bridge::MessageContent::Image(..) => content.push_str("[图片]"),
            bridge::MessageContent::Reply { .. } => content.push_str("[回复消息]"),
            bridge::MessageContent::At { id } => {
                let bridge_user = bridge::manager::BRIDGE_USER_MANAGER.lock().await.get(&id).await;
                if let None = bridge_user {
                    content.push_str(format!("@[UN] {}", id).as_str());
                    continue;
                }
                let bridge_user = bridge_user.unwrap();
                // 查看桥关联的本平台用户id
                if let Some(ref_user) = bridge_user.find_by_platform("DC").await {
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

/**
 * 解析文本规则取出提及@[DC]用户的文本
 */
#[derive(Debug)]
pub enum MentionText {
    Text(String),
    MentionText { name: String, discriminator: String },
}
pub fn parse_text_mention_rule(text: String) -> Vec<MentionText> {
    let mut text = text;
    let mut chain: Vec<MentionText> = vec![];
    let split_const = "#|x-x|#".to_string();
    let reg_at_user = regex::Regex::new(r"@\[DC\] ([^\n^#^@]+)?#(\d\d\d\d)").unwrap();
    // let caps = reg_at_user.captures(text);
    while let Some(caps) = reg_at_user.captures(text.as_str()) {
        println!("{:?}", caps);
        let from = caps.get(0).unwrap().as_str();
        let name = caps.get(1).unwrap().as_str().to_string();
        let discriminator = caps.get(2).unwrap().as_str().to_string();

        let result = text.replace(from, &split_const);
        let splits: Vec<&str> = result.split(&split_const).collect();
        let prefix = splits.get(0).unwrap();
        chain.push(MentionText::Text(prefix.to_string()));
        chain.push(MentionText::MentionText { name, discriminator });
        if let Some(fix) = splits.get(1) {
            text = fix.to_string();
        }
    }
    if text.len() > 0 {
        chain.push(MentionText::Text(text.to_string()));
    }
    println!("parse_text_mention_rule: {:?}", chain);
    chain
}
