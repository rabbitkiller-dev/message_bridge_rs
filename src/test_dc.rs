use serenity::async_trait;
use serenity::http::Http;
use serenity::model::channel::AttachmentType;
use serenity::model::channel::Message;
use serenity::model::gateway::{GatewayIntents, Presence, Ready};
use serenity::model::prelude::Guild;
use serenity::model::webhook::Webhook;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

use std::env;

use crate::config::Config;

#[test]
fn use_webhook_send_dc_message() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let config = Config::new();
            let bridgeConfig = config.bridges.get(0).unwrap();
            let http = Http::new("");
            let webhook = Webhook::from_id_with_token(
                &http,
                bridgeConfig.discord.id,
                &bridgeConfig.discord.token,
            )
            .await
            .unwrap();
            // webhook.kind
            webhook
                .execute(&http, false, |w| {
                    w.username("Webhook test").components(|c| {
                        c.create_action_row(|row| {
                            row.create_button(|b| b.custom_id("btn").label("测试"))
                        })
                    })
                })
                .await
                .expect("Could not execute webhook.");
        })
}

#[test]
fn test_tokio_select() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            tokio::select! {
                _ = async {
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                        println!("第一个异步运行中...");
                    }
                } => {
                    println!("第一个异步结束");
                },
                val = async {
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                        println!("第一个异步运行中...");
                    }
                    "hello"
                } => {
                    println!("第二个异步结束");
                }
            }
            println!("结束");
        })
}

/**
 * 获取伺服所有用户
 */
#[test]
fn use_webhook_get_guild_user() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let config = Config::new();

            let token = &config.discord_config.botToken;

            let http = Http::new(&token);
            let member = http
                .get_member(724829522230378536, 724827488588660837)
                .await
                .unwrap();
            println!("member: {:?}", member);
        })
}

/**
 * 获取伺服所有用户
 */
#[test]
fn use_webhook_get_guild_all_user() {
    let config = Config::new();

    let token = &config.discord_config.botToken;

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let http = Http::new(&token);
            let member = http
                .get_guild_members(724829522230378536, None, None)
                .await
                .unwrap();
            println!("members: {:?}", member);
        })
}
