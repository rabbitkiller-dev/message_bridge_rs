// TODO 交互式操作的上下文

use clap::{FromArgMatches, Subcommand};
use std::sync::Arc;

use crate::{
    bridge::{
        manager::BRIDGE_USER_MANAGER,
        pojo::{BridgeMessageRefPO, BridgeSendMessageForm},
        BridgeClient, BridgeMessage, MessageContent,
    },
    elr,
};

use super::{BridgeCommand, CommandCentext, CommandMessageParser};

/// 识别解析以 BridgeMessage 为载体的指令
impl CommandMessageParser<BridgeMessage> for BridgeMessage {
    #[tracing::instrument(skip_all)]
    fn try_parse(&self, from_client: &str) -> Result<CommandCentext<BridgeMessage>, &'static str> {
        let ctx = &self.message_chain;
        let Some(MessageContent::Plain { text }) = ctx.first() else {
            return Err("获取不到文本！");
        };
        let text = text.trim();
        if text.is_empty() || !text.starts_with('!') {
            return Err("空消息；或前缀错误！");
        }
        let args = text.split_whitespace();
        let patter = BridgeCommand::augment_subcommands(clap::Command::new("cc").no_binary_name(true));
        // let mat = elr!(patter.clone().try_get_matches_from(args) ;; return None);
        // let cmd = elr!(BridgeCommand::from_arg_matches(&mat) ;; return None);
        let Ok(mat) = patter.clone().try_get_matches_from(args) else {
            return Err("此消息不符合指令格式！");
        };
        let Ok(cmd) = BridgeCommand::from_arg_matches(&mat) else {
            return Err("未匹配相关指令！");
        };
        Ok(CommandCentext {
            client: from_client.to_string(),
            src_msg: self.clone(),
            ctx: patter,
            token: cmd,
        })
    }
}

/// 接收桥内消息，尝试处理
#[tracing::instrument(skip_all)]
pub async fn listen(bridge: Arc<BridgeClient>) {
    let mut subs = bridge.sender.subscribe();
    loop {
        let message = elr!(subs.recv().await ;; continue);
        // 匹配消息是否是命令
        let cmd = match message.try_parse(&bridge.name) {
            Ok(cmd) => cmd,
            Err(e) => {
                tracing::debug!("{e}");
                continue;
            }
        };
        tracing::info!("[指令] {:?}", cmd.token);
        // 指令反馈
        let feedback = match cmd.process_command().await {
            Ok(fb) => fb,
            Err(e) => {
                tracing::error!("{e}");
                continue;
            }
        };
        let Some(user) = BRIDGE_USER_MANAGER.lock().await.like("00000001", "CMD").await else {
            tracing::warn!("无法获取CMD用户！");
            continue;
        };
        let bridge_msg = BridgeSendMessageForm {
            origin_message: BridgeMessageRefPO {
                origin_id: uuid::Uuid::new_v4().to_string(),
                platform: "CMD".to_string(),
            },
            avatar_url: Some("https://q1.qlogo.cn/g?b=qq&nk=3245538509&s=100".to_string()),
            bridge_config: message.bridge_config.clone(),
            message_chain: feedback,
            sender_id: user.id,
        };
        bridge.send_message(bridge_msg).await
    } // loop
}
