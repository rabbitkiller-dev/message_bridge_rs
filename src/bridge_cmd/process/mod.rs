//! 指令处理
//! TODO 枚举所有错误

mod bind_proc;
mod token;

use tracing::instrument;

use crate::bridge::{BridgeMessage, MessageContent};

use super::{BridgeCommand, CommandCentext, CMD_BIND, CMD_CONFIRM_BIND, CMD_UNBIND};

type Feedback = Result<Vec<MessageContent>, String>;

#[inline]
fn simple_feedback(msg: &str) -> Feedback {
    Ok(vec![MessageContent::Plain { text: msg.to_string() }])
}
#[inline]
fn simple_fail(msg: &str) -> Feedback {
    Err(msg.to_string())
}

impl CommandCentext<BridgeMessage> {
    /// # 申请关联
    /// ### Return
    /// 验证口令
    #[instrument(skip_all)]
    async fn req_bind(&self) -> Feedback {
        let Ok(token) = bind_proc::create_session(&self.src_msg.sender_id).await else {
            return simple_fail("申请失败，请联系管理员处理。");
        };
        Ok(vec![MessageContent::Plain {
            text: format!("申请成功。请切换客户端，使用验证码回应请求: {token}"),
        }])
    }

    /// 回应申请
    /// ### Argument
    /// `token` 验证口令
    #[instrument(skip_all)]
    async fn resp_bind(&self, token: &str) -> Feedback {
        if let Err(e) = bind_proc::update_resp(&token, &self.src_msg.sender_id).await {
            tracing::error!("{e}");
            return simple_fail("提交失败，请联系管理员处理。");
        }
        simple_feedback("OK，请回到原客户端进行确认。")
    }

    /// 申请/回应用户关联
    async fn bind(&self) -> Feedback {
        if let BridgeCommand::Bind { token: Some(t) } = &self.token {
            return self.resp_bind(t).await;
        }
        self.req_bind().await
    }

    /// 接收关联
    #[instrument(skip_all)]
    async fn confirm_bind(&self) -> Feedback {
        if let Err(e) = bind_proc::confirm_bind(&self.src_msg.sender_id).await {
            tracing::error!("{e}");
            return simple_fail("关联失败，请联系管理员处理。");
        }
        simple_feedback("完成关联。")
    }

    /// 取消关联
    #[instrument(skip_all)]
    async fn unbind(&self) -> Feedback {
        if let BridgeCommand::Unbind { platform } = &self.token {
            if platform == &self.client {
                return simple_fail("不要做自引用操作");
            }
            if let Err(e) = bind_proc::unbind(&self.src_msg.sender_id, platform).await {
                tracing::error!("{e}");
                return simple_fail("操作失败，请联系管理员处理。");
            }
        };
        simple_feedback("已取消关联。")
    }

    /// 获取指令帮助
    fn get_help(&self) -> Feedback {
        let mut sub = "".to_string();
        if let BridgeCommand::Tips { command: Some(cmd) } = &self.token {
            if cmd.starts_with('!') {
                sub = cmd.to_owned();
            } else {
                sub = format!("!{cmd}");
            }
        }
        // TODO 文本内容通过 toml 文件读写
        let text = match &*sub {
            CMD_BIND => format!(
                "申请关联，获取验证码；或者用验证码回应申请
用法：{CMD_BIND} [口令]
口令\t\t选填。无口令时申请；有口令时回应申请
【申请关联】{CMD_BIND}
【回应申请】{CMD_BIND} 1a2b3c"
            ),
            CMD_CONFIRM_BIND => format!("确定保存关联。无参\n用法: {CMD_CONFIRM_BIND}"),
            CMD_UNBIND => format!(
                "【解除桥用户关联】解除指定平台的关联
用法：{CMD_UNBIND} <平台>
平台\t\t必填，单选。选项：QQ、DC=Discord、TG=Telegram
【用例】{CMD_UNBIND} DC"
            ),
            _ => format!(
                "桥的可用指令：
【申请/回应关联桥用户】{CMD_BIND} [口令]
【确认关联】{CMD_CONFIRM_BIND}
【解除桥用户关联】{CMD_UNBIND} <平台>"
            ),
        };
        Ok(vec![MessageContent::Plain { text }])
    }

    /// # 指令处理
    /// ### Return
    /// - `Some(feedback)` 反馈指令处理结果
    /// - `Err(..)` 失败描述
    pub async fn process_command(&self) -> Feedback {
        use super::BridgeCommand::*;
        match self.token {
            Bind { .. } => self.bind().await,
            ConfirmBind => self.confirm_bind().await,
            Unbind { .. } => self.unbind().await,
            Tips { .. } => self.get_help(),
            // _ => Err("TODO".to_string()),
        }
    }
}
