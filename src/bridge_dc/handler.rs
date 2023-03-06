use std::sync::Arc;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::Timestamp;
use serenity::prelude::*;
use tracing::{debug, error, info, instrument, trace};

use crate::bridge::BridgeClientPlatform;
use crate::bridge_dc::apply_bridge_user;
use crate::bridge_message_history::{BridgeMessageHistory, Platform};
use crate::{bridge, Config};

pub struct Handler {
    pub config: Arc<Config>,
    pub bridge: Arc<bridge::BridgeClient>,
}

#[async_trait]
impl EventHandler for Handler {
    #[instrument(skip_all, name = "bridge_dc_recv")]
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.id == self.config.discord_config.botId {
            // 收到自己bot的消息, 不要继续以免消息循环
            return;
        }

        // 收到桥配置的webhook消息, 不要继续以免消息循环
        if self
            .config
            .bridges
            .iter()
            .any(|bridge| msg.author.id == bridge.discord.id)
        {
            return;
        }
        let bridge_config = match self
            .config
            .bridges
            .iter()
            .find(|bridge| msg.channel_id == bridge.discord.channelId && bridge.enable)
        {
            Some(c) => c,
            // 该消息的频道没有配置桥, 忽略这个消息
            None => return,
        };
        let _bridge_user = apply_bridge_user(
            msg.author.id.0,
            msg.author.name.as_str(),
            msg.author.discriminator,
        )
        .await;
        let mut user = bridge::User {
            name: format!("[DC] {}#{}", msg.author.name, msg.author.discriminator),
            avatar_url: None,
            platform_id: 0,
            unique_id: msg.author.id.0,
            platform: BridgeClientPlatform::Discord,
            display_id: msg.author.discriminator as u64,
        };
        if let Some(url) = msg.author.avatar_url() {
            user.avatar_url = Some(url.replace(".webp?size=1024", ".png?size=40").to_string());
        }
        if let Some(gid) = msg.guild_id {
            user.platform_id = gid.0
        }
        let mut bridge_message = bridge::BridgeMessage {
            id: uuid::Uuid::new_v4().to_string(),
            bridge_config: bridge_config.clone(),
            message_chain: Vec::new(),
            user,
        };
        // 记录消息id
        BridgeMessageHistory::insert(
            &bridge_message.id,
            Platform::Discord,
            msg.id.0.to_string().as_str(),
        )
        .await
        .unwrap();

        let result = crate::utils::parser_message(&msg.content).await;
        for ast in result {
            match ast {
                crate::utils::MarkdownAst::Plain { text } => {
                    bridge_message
                        .message_chain
                        .push(bridge::MessageContent::Plain { text });
                }
                crate::utils::MarkdownAst::At { username } => {
                    trace!("用户'{}'收到@", username);
                    bridge_message
                        .message_chain
                        .push(bridge::MessageContent::Plain { text: username });
                    // bridge_message
                    //     .message_chain
                    //     .push(bridge::MessageContent::At {
                    //         bridge_user_id: None,
                    //         username,
                    //     });
                }
                crate::utils::MarkdownAst::DiscordAtUser { id } => {
                    let id: u64 = id.parse::<u64>().unwrap();
                    let member = ctx
                        .http
                        .get_member(msg.guild_id.unwrap().0, id)
                        .await
                        .unwrap();
                    let bridge_user =
                        apply_bridge_user(id, member.user.name.as_str(), member.user.discriminator)
                            .await;
                    // let member_name =
                    //     format!("[DC] {}#{}", member.user.name, member.user.discriminator);
                    // trace!("用户'{}'收到@", member_name);
                    bridge_message
                        .message_chain
                        .push(bridge::MessageContent::At { id: bridge_user.id });
                }
                crate::utils::MarkdownAst::DiscordAtEveryone {} => {
                    bridge_message
                        .message_chain
                        .push(bridge::MessageContent::AtAll);
                }
                crate::utils::MarkdownAst::DiscordAtHere {} => {
                    bridge_message
                        .message_chain
                        .push(bridge::MessageContent::AtAll);
                }
                crate::utils::MarkdownAst::DiscordEmoji { id, animated, .. } => {
                    let suffix = if animated { "gif" } else { "png" };
                    bridge_message
                        .message_chain
                        .push(bridge::MessageContent::Image {
                            url: Some(format!(
                                "https://cdn.discordapp.com/emojis/{}.{}",
                                id, suffix
                            )),
                            path: None,
                        });
                }
            }
        }
        // 将附件一股脑的放进图片里面 TODO: 以后在区分非图片的附件
        for attachment in msg.attachments {
            trace!(attachment.url);
            bridge_message
                .message_chain
                .push(bridge::MessageContent::Image {
                    url: Some(attachment.url),
                    path: None,
                });
        }
        debug!("dc 桥的消息链：{:#?}", bridge_message.message_chain);

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
                error!("消息发送失败！{:#?}", why);
            }
        }
    }

    #[instrument(skip_all, target = "bridge_dc")]
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::warn!("(Guild={:?})准备连接Discord伺服器", ready.guilds);
        for bridge_config in self.config.bridges.iter() {
            match ctx.http.get_channel(bridge_config.discord.channelId).await {
                Ok(channel) => {
                    let msg = "Message Bridge正在运行中...";
                    let resp = channel
                        .id()
                        .send_message(&ctx.http, |m| {
                            m.content(msg);
                            m
                        })
                        .await;
                    if let Err(e) = resp {
                        error!(msg, err = ?e, "消息发送失败！")
                    } else {
                        info!("已连接到 discord 频道 {}", bridge_config.discord.channelId);
                    }
                }
                Err(e) => error!(
                    channel = bridge_config.discord.channelId,
                    err = ?e,
                    "获取 discord 频道失败！",
                ),
            }
        }
    }
}
