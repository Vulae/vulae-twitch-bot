pub mod neovim;
pub mod radio;
pub mod simple_reply;

use anyhow::{Error, Result};
use twitcheventsub::{MessageData, TwitchEventSubApi};

use crate::{
    command::{Command, CommandArgsResult},
    config::Config,
    twitch_event_handler::TwitchEventHandler,
};

pub struct CommandRegistry {
    radio: radio::Radio,
    neovim: neovim::Neovim,
    simple_reply_commands: simple_reply::SimpleReplyCommandHandler,
}

impl CommandRegistry {
    pub fn initialize(config: &Config) -> Result<Self> {
        Ok(Self {
            radio: radio::Radio::initialize(config.radio.clone())?,
            neovim: neovim::Neovim::initialize()?,
            simple_reply_commands: config.data().simple_reply_commands.clone(),
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

        if let CommandArgsResult::Execute(args) =
            self.simple_reply_commands.parse_args(chat_message)
        {
            if let Err(err) = self.simple_reply_commands.execute(args, chat_message, api) {
                println!("ERR: {:#?}", err);
            }
        }

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
