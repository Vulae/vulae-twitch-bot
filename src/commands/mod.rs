pub mod neovim;
mod radio;
mod simple_reply;

use anyhow::{Error, Result};
use simple_reply::SimpleReplyCommand;
use twitcheventsub::{MessageData, TwitchEventSubApi};

use crate::{
    command::{Command, CommandArgsResult},
    create_simple_reply_command,
    twitch_event_handler::TwitchEventHandler,
};

create_simple_reply_command!(CommandGitHub; "github", "gh"; "https://github.com/Vulae");
create_simple_reply_command!(CommandBot; "bot"; "https://github.com/Vulae/vulae-twitch-bot");
create_simple_reply_command!(CommandCommands; "commands", "cmds", "help"; "https://github.com/Vulae/vulae-twitch-bot#commands");
create_simple_reply_command!(CommandDotFiles; "dotfiles", "dotconfig", ".config"; "https://github.com/Vulae/dotfiles");
//create_simple_reply_command!(CommandUwU; "uwu"; "OwO");

pub struct CommandRegistry {
    radio: radio::Radio,
    neovim: neovim::Neovim,
    simple_reply_commands: Vec<Box<dyn SimpleReplyCommand>>,
}

impl CommandRegistry {
    pub fn initialize() -> Result<Self> {
        Ok(Self {
            radio: radio::Radio::initialize()?,
            neovim: neovim::Neovim::initialize()?,
            simple_reply_commands: vec![
                Box::new(CommandBot),
                Box::new(CommandGitHub),
                Box::new(CommandCommands),
                Box::new(CommandDotFiles),
                //Box::new(CommandUwU),
            ],
        })
    }

    pub fn try_execute(
        &mut self,
        chat_message: &MessageData,
        api: &mut TwitchEventSubApi,
    ) -> Result<()> {
        match self.radio.parse_args(chat_message) {
            CommandArgsResult::BadArguments(message) => {
                let _ = api
                    .send_chat_message_with_reply(message, Some(chat_message.message_id.clone()));
            }
            CommandArgsResult::Execute(args) => {
                if let Err(err) = self.radio.execute(args, chat_message, api) {
                    println!("ERR: {:#?}", err);
                }
            }
            _ => {}
        }
        match self.neovim.parse_args(chat_message) {
            CommandArgsResult::BadArguments(message) => {
                let _ = api
                    .send_chat_message_with_reply(message, Some(chat_message.message_id.clone()));
            }
            CommandArgsResult::Execute(args) => {
                if let Err(err) = self.neovim.execute(args, chat_message, api) {
                    println!("ERR: {:#?}", err);
                }
            }
            _ => {}
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
        self.neovim.update(api)?;

        Ok(())
    }
}

impl TwitchEventHandler for CommandRegistry {
    fn subscribed_events(&self) -> &[twitcheventsub::Subscription] {
        &[twitcheventsub::Subscription::ChatMessage]
    }

    fn handle_event(
        &mut self,
        event: &twitcheventsub::Event,
        api: &mut twitcheventsub::TwitchEventSubApi,
    ) -> Result<()> {
        match event {
            twitcheventsub::Event::ChatMessage(message) => self.try_execute(message, api),
            _ => Ok(()),
        }
    }
}
