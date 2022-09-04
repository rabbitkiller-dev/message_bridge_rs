use crate::{bridge, Config};
use std::sync::{Arc, Mutex};

use serenity::async_trait;
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::webhook::Webhook;
use serenity::model::Timestamp;
use serenity::prelude::*;

pub async fn dc(bridge: Arc<bridge::BridgeService>) {
    loop {
        let message = bridge.sender.subscribe().recv().await.unwrap();
        println!("在dc监听到内容");
        let http = Http::new("");
        let webhook = Webhook::from_url(&http, "https://discord.com/api/webhooks/1005861193623285810/QWgRpQtk8HHO1HxL6i7XVxM3GD8C21u9YCizKzdilpqdIhnswoWB_x6zsCuiOHk899gt").await.unwrap();
        webhook
            .execute(&http, false, |w| {
                for chain in &message.message_chain {
                    match chain {
                        bridge::MessageContent::Plain { text } => {
                            w.content(text);
                        }
                        _ => {
                            println!("消息的内容没有处理");
                        }
                    }
                }
                w.username("bot");
                w
            })
            .await
            .expect("Could not execute webhook.");
    }
}

pub struct Handler {
    pub config: Arc<Config>,
    pub bridge: Arc<bridge::BridgeService>,
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

        let bridgeConfig = match self
            .config
            .bridges
            .iter()
            .find(|bridge| msg.channel_id == bridge.discord.channelId && bridge.enable)
        {
            Some(bridgeConfig) => bridgeConfig,
            None => {
                // 该消息的频道没有配置桥, 忽略这个消息
                return;
            }
        };

        let sender = self.bridge.sender.clone();
        let bridge_message = bridge::BridgeMessage {
            bridge_config: bridgeConfig.clone(),
            message_chain: Vec::new(),
        };
        // sender.send(bridge_message);
        println!("收到来自dc的消息: {}", msg.content);
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
