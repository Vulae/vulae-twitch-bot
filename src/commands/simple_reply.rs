use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use twitcheventsub::MessageData;

use crate::command::{Command, CommandArgsResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleReplyCommand {
    names: Vec<String>,
    responds: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SimpleReplyCommandHandler(HashMap<String, SimpleReplyCommand>);

impl Command<String> for SimpleReplyCommandHandler {
    fn parse_args(&self, chat_message: &MessageData) -> CommandArgsResult<String> {
        self.0
            .iter()
            .find(|(_, command)| {
                command.names.iter().any(|name| {
                    chat_message
                        .message
                        .text
                        .to_lowercase()
                        .starts_with(&format!("!{}", name))
                })
            })
            .map(|(key, _)| CommandArgsResult::Execute(key.clone()))
            .unwrap_or(CommandArgsResult::WrongCommand)
    }

    fn execute(
        &mut self,
        args: String,
        chat_message: &twitcheventsub::MessageData,
        api: &mut twitcheventsub::TwitchEventSubApi,
    ) -> anyhow::Result<()> {
        if let Some(command) = self.0.get(&args) {
            let _ = api.send_chat_message_with_reply(
                &command.responds,
                Some(chat_message.message_id.clone()),
            );
        }
        Ok(())
    }
}
