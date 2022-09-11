use std::path::Path;
use std::sync::Arc;

use serenity::async_trait;
use serenity::http::Http;
use serenity::model::channel::AttachmentType;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::Timestamp;
use serenity::model::webhook::Webhook;
use serenity::prelude::*;

use crate::{bridge, Config};
use crate::bridge::BridgeClientPlatform;
use crate::bridge_log;

/**
 *
 */
pub async fn dc(bridge: Arc<bridge::BridgeClient>) {
    let mut subs = bridge.sender.subscribe();
    loop {
        let message = &subs.recv().await.unwrap();
        println!("[bridge_dc] 收到桥的消息, 同步到discord上");
        let http = Http::new("");
        let webhook = Webhook::from_id_with_token(
            &http,
            message.bridge_config.discord.id,
            message.bridge_config.discord.token.as_str(),
        )
        .await
        .unwrap();

        webhook
            .execute(&http, false, |w| {
                // 配置发送者头像
                if let Some(url) = &message.user.avatar_url {
                    w.avatar_url(url.as_str());
                }
                // 配置发送者用户名
                w.username(message.user.name.clone());

                let mut content: Vec<&str> = Vec::new();
                for chain in &message.message_chain {
                    match chain {
                        bridge::MessageContent::Plain { text } => content.push(text),
                        bridge::MessageContent::Image { url, path } => {
                            if let Some(path) = path {
                                let path = Path::new(path);
                                w.add_file(AttachmentType::Path(path));
                                continue;
                            }
                            if let Some(url) = url {
                                let url = url::Url::parse(url).unwrap();
                                w.add_file(AttachmentType::Image(url));
                            }
                        }
                        _ => content.push("{无法识别的MessageChain}"),
                    };
                }
                if content.len() == 0 {
                    content.push("{本次发送的消息没有内容}");
                }
                w.content(content.join(""))
            })
            .await
            .expect("[bridge_dc] Could not execute webhook.");
    }
}

pub async fn start(config: Arc<Config>, bridge: Arc<bridge::BridgeClient>) {
    let token = &config.discordConfig.botToken;
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    println!("dc");

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            config: config.clone(),
            bridge: bridge.clone(),
        })
        .await
        .expect("Err creating client");

    println!("dc2");
    tokio::select! {
        _ = client.start() => {
            println!("xxxxxx");
        },
        _ = dc(bridge.clone()) => {},
    }
}

pub struct Handler {
    pub config: Arc<Config>,
    pub bridge: Arc<bridge::BridgeClient>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.id == self.config.discordConfig.botId {
            // 收到自己bot的消息, 不要继续以免消息循环
            return;
        }

        // 收到桥配置的webhook消息, 不要继续以免消息循环
        if let Some(_) = self
            .config
            .bridges
            .iter()
            .find(|bridge| msg.author.id == bridge.discord.id)
        {
            return;
        };
        let bridge_config = match self
            .config
            .bridges
            .iter()
            .find(|bridge| msg.channel_id == bridge.discord.channelId && bridge.enable)
        {
            Some(c) => c,
            None => {
                // 该消息的频道没有配置桥, 忽略这个消息
                return;
            }
        };
        let mut user = bridge::User {
            name: format!("[DC] {}#{}", msg.author.name, msg.author.discriminator),
            avatar_url: None,
            platform_id: 0,
            unique_id: msg.author.id.0,
            platform: BridgeClientPlatform::Discord,
            display_id: msg.author.discriminator as u64,
        };
        if let Some(url) = msg.author.avatar_url() {
            println!("[bridge_dc] avatar_url: {:?}", url);
            user.avatar_url = Some(url.replace(".webp?size=1024", ".png?size=40").to_string());
        }
        if let Some(gid) = msg.guild_id {
            user.platform_id = gid.0
        }
        // println!(
        //     "msg.author.default_avatar_url(){:?}",
        //     msg.author.static_avatar_url()
        // );

        bridge_log::BridgeLog::write_log(
            format!(
                r#"discord桥要发送的消息
{}
{}"#,
                user.name, msg.content
            )
            .as_str(),
        );

        // let sender = self.bridge.sender.clone();

        let mut bridge_message = bridge::BridgeMessage {
            bridge_config: bridge_config.clone(),
            message_chain: Vec::new(),
            user,
        };
        bridge_message
            .message_chain
            .push(bridge::MessageContent::Plain {
                text: msg.content.clone(),
            });
        // 将附件一股脑的放进图片里面 TODO: 以后在区分非图片的附件
        for attachment in msg.attachments {
            bridge_message
                .message_chain
                .push(bridge::MessageContent::Image {
                    url: Some(attachment.url),
                    path: None,
                });
        }

        self.bridge.send(bridge_message);
        if msg.content == "!hello" {
            // The create message builder allows you to easily create embeds and messages
            // using a builder syntax.
            // This example will create a message that says "Hello, World!", with an embed that has
            // a title, description, an image, three fields, and a footer.
            let msg = msg
                .channel_id
                .send_message(&ctx.http, |m| {
                    m.content("Hello, World!")
                        .embed(|e| {
                            e.title("This is a title")
                                .description("This is a description")
                                .image("attachment://ferris_eyes.png")
                                .fields(vec![
                                    ("This is the first field", "This is a field body", true),
                                    ("This is the second field", "Both fields are inline", true),
                                ])
                                .field(
                                    "This is the third field",
                                    "This is not an inline field",
                                    false,
                                )
                                .footer(|f| f.text("This is a footer"))
                                // Add a timestamp for the current time
                                // This also accepts a rfc3339 Timestamp
                                .timestamp(Timestamp::now())
                        })
                        .add_file("./ferris_eyes.png")
                })
                .await;

            if let Err(why) = msg {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} 已连接到discord!", ready.user.name);
    }
}
