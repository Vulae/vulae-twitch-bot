mod radio;
mod simple_reply;

use anyhow::{Error, Result};
use simple_reply::SimpleReplyCommand;
use twitcheventsub::{MessageData, TwitchEventSubApi};

use crate::{
    command::{Command, CommandArgsResult},
    create_simple_reply_command,
};

create_simple_reply_command!(CommandBot; "bot"; "https://github.com/Vulae/vulae-twitch-bot");
create_simple_reply_command!(CommandGitHub; "github", "gh"; "https://github.com/Vulae");
create_simple_reply_command!(CommandUwU; "uwu"; "OwO");

pub struct CommandRegistry {
    radio: radio::Radio,
    simple_reply_commands: Vec<Box<dyn SimpleReplyCommand>>,
}

impl CommandRegistry {
    pub fn initialize() -> Result<Self> {
        Ok(Self {
            radio: radio::Radio::initialize()?,
            simple_reply_commands: vec![
                Box::new(CommandBot),
                Box::new(CommandGitHub),
                Box::new(CommandUwU),
            ],
        })
    }

    pub fn try_execute(
        &mut self,
        chat_message: &MessageData,
        api: &mut TwitchEventSubApi,
    ) -> Result<()> {
        if let CommandArgsResult::Execute(args) = self.radio.parse_args(chat_message) {
            if let Err(err) = self.radio.execute(args, chat_message, api) {
                println!("ERR: {:#?}", err);
            }
        }

        self.simple_reply_commands
            .iter_mut()
            .try_for_each(|command| {
                if let CommandArgsResult::Execute(args) = command.parse_args(chat_message) {
                    command.execute(args, chat_message, api)?;
                }
                Ok::<(), Error>(())
            })?;

        Ok(())
    }

    pub fn update(&mut self, api: &mut TwitchEventSubApi) -> Result<()> {
        self.radio.update(api)?;

        Ok(())
    }
}
