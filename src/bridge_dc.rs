use std::path::Path;
use std::sync::Arc;

use regex::Regex;
use serenity::async_trait;
use serenity::http::Http;
use serenity::model::channel::AttachmentType;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::guild::Member;
use serenity::model::webhook::Webhook;
use serenity::model::Timestamp;
use serenity::prelude::*;
use tracing::{debug, error, info, instrument, trace, warn};

use crate::bridge::BridgeClientPlatform;
use crate::bridge_message_history::{BridgeMessageHistory, Platform};
use crate::{bridge, Config};

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
        let mut fils: Vec<AttachmentType> = Vec::new();
        for chain in &message.message_chain {
            match chain {
                bridge::MessageContent::Plain { text } => content.push(text.clone()),
                bridge::MessageContent::Image { url, path } => {
                    trace!(?url, ?path, "图片");
                    if let Some(path) = path {
                        let path = Path::new(path);
                        fils.push(AttachmentType::Path(path));
                        continue;
                    }
                    if let Some(url) = url {
                        let url = url::Url::parse(url).unwrap();
                        fils.push(AttachmentType::Image(url));
                    }
                }
                bridge::MessageContent::At { username, .. } => {
                    trace!("用户'{}'收到@", username);
                    let re = Regex::new(r"@\[DC\] ([^\n]+)?#(\d\d\d\d)").unwrap();
                    let caps = re.captures(username);
                    match caps {
                        Some(caps) => {
                            let name = caps.get(1).unwrap();
                            let dis = caps.get(2).unwrap();
                            let member =
                                find_member_by_name(&http, guild_id.0, name.as_str(), dis.as_str())
                                    .await;
                            if let Some(member) = member {
                                content.push(format!("<@{}>", member.user.id.0));
                            } else {
                                content.push(username.clone());
                                warn!("@用户无法解析: {}", username);
                            }
                        }
                        None => {
                            content.push(username.clone());
                            warn!("@用户无法解析: {}", username);
                        }
                    }
                }
                _ => warn!(unit = ?chain, "无法识别的MessageChain"),
            };
        }
        debug!(?content, ?fils, "桥内消息链组装完成");

        if let BridgeClientPlatform::Cmd = message.user.platform {
            match http
                .get_channel(message.bridge_config.discord.channelId)
                .await
            {
                Ok(channel) => {
                    if let Err(err) = channel
                        .id()
                        .send_message(&http, |w| w.content(content.join("")))
                        .await
                    {
                        error!(?err, "发送指令反馈失败！")
                    } else {
                        debug!("指令反馈完成");
                    }
                }
                Err(err) => error!(?err, "获取 discord 频道失败！"),
            }
            continue;
        }

        let resp = webhook
            .execute(&http, false, |w| {
                // 配置发送者头像
                if let Some(url) = &message.user.avatar_url {
                    w.avatar_url(url.as_str());
                }
                debug!("消息头像url：{:?}", message.user.avatar_url);
                // 配置发送者用户名
                w.username(message.user.name.clone());
                if content.len() == 0 && fils.len() == 0 {
                    content.push("{本次发送的消息没有内容}".to_string());
                }
                w.add_files(fils).content(content.join(""))
            })
            .await;
        if let Err(err) = resp {
            error!(?err, "消息同步失败！")
        } else {
            info!("已同步消息")
        }
    }
}

pub async fn start(config: Arc<Config>, bridge: Arc<bridge::BridgeClient>) {
    let token = &config.discordConfig.botToken;
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            config: config.clone(),
            bridge: bridge.clone(),
        })
        .await
        .expect("Err creating client");

    let cache = client.cache_and_http.clone();
    info!("bridge_dc ready");

    tokio::select! {
        _ = client.start() => {
            warn!("dc client exited");
        },
        _ = dc(bridge.clone(), cache.http.clone()) => {
            warn!("bridge_dc listening is closed");
        },
    }
}

pub struct Handler {
    pub config: Arc<Config>,
    pub bridge: Arc<bridge::BridgeClient>,
}

#[async_trait]
impl EventHandler for Handler {
    #[instrument(skip_all, name = "bridge_dc_recv")]
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.id == self.config.discordConfig.botId {
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
        debug!("discord user: {:#?}", user);
        //         bridge_log::BridgeLog::write_log(
        //             format!(
        //                 r#"discord桥要发送的消息
        // {}
        // {}"#,
        //                 user.name, msg.content
        //             )
        //                 .as_str(),
        //         );
        info!("{} -> {}", user.name, msg.content);

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
        );

        let result = crate::utils::parser_message(&msg.content);
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
                        .push(bridge::MessageContent::At {
                            bridge_user_id: None,
                            username,
                        });
                }
                crate::utils::MarkdownAst::AtInDiscordUser { id } => {
                    let id: u64 = id.parse::<u64>().unwrap();
                    let member = ctx
                        .http
                        .get_member(msg.guild_id.unwrap().0, id)
                        .await
                        .unwrap();
                    let member_name =
                        format!("[DC] {}#{}", member.user.name, member.user.discriminator);
                    trace!("用户'{}'收到@", member_name);
                    bridge_message
                        .message_chain
                        .push(bridge::MessageContent::At {
                            bridge_user_id: None,
                            username: member_name,
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
        info!("准备连接伺服器：{:?}", ready.guilds);
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

/**
 * 通过名称和discriminator查询成员
 */
#[instrument(level = "debug", skip(http), ret)]
async fn find_member_by_name(
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
