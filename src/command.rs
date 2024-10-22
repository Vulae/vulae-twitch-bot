use anyhow::Result;
use twitcheventsub::{MessageData, TwitchEventSubApi};

#[derive(Debug)]
pub enum CommandArgsResult<Args> {
    WrongCommand,
    UnsufficientPermissions,
    BadArguments,
    Execute(Args),
}

pub trait Command<Args> {
    /// Return none if chat_message isn't for this command.
    fn parse_args(&self, chat_message: &MessageData) -> CommandArgsResult<Args>;
    fn execute(
        &mut self,
        args: Args,
        chat_message: &MessageData,
        api: &mut TwitchEventSubApi,
    ) -> Result<()>;
    #[allow(unused)]
    fn update(&mut self, api: &mut TwitchEventSubApi) -> Result<()> {
        Ok(())
    }
}
