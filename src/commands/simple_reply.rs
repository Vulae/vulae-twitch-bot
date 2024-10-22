use anyhow::Result;
use twitcheventsub::{MessageData, TwitchEventSubApi};

use crate::command::{Command, CommandArgsResult};

pub trait SimpleReplyCommand {
    fn names(&self) -> &[&str];
    fn reply(&self) -> &str;
}

impl Command<()> for dyn SimpleReplyCommand {
    fn parse_args(&self, chat_message: &MessageData) -> CommandArgsResult<()> {
        if self.names().iter().any(|name| {
            chat_message
                .message
                .text
                .to_lowercase()
                .starts_with(&format!("!{}", name.to_lowercase()))
        }) {
            CommandArgsResult::Execute(())
        } else {
            CommandArgsResult::WrongCommand
        }
    }

    fn execute(
        &mut self,
        _args: (),
        chat_message: &MessageData,
        api: &mut TwitchEventSubApi,
    ) -> Result<()> {
        let _ =
            api.send_chat_message_with_reply(self.reply(), Some(chat_message.message_id.clone()));
        Ok(())
    }
}

#[macro_export]
macro_rules! create_simple_reply_command {
    ($struct_name:ident; $($name:literal),+; $message:literal) => {
        struct $struct_name;

        impl $crate::commands::simple_reply::SimpleReplyCommand for $struct_name {
            fn names(&self) -> &[&str] {
                &[
                    $(
                        $name,
                    )+
                ]
            }
            fn reply(&self) -> &str {
                $message
            }
        }
    };
}
