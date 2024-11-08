pub mod command;
pub mod commands;
pub mod twitch_event_handler;

use std::time::Duration;

use anyhow::{Error, Result};
use commands::CommandRegistry;
use twitch_event_handler::TwitchEventHandler;
use twitcheventsub::{ResponseType, TwitchEventSubApi, TwitchKeys};

const API_BOT_USER_ID: &str = "1131985206";

struct TestHandler;
impl TwitchEventHandler for TestHandler {
    fn subscribed_events(&self) -> &[twitcheventsub::Subscription] {
        &[twitcheventsub::Subscription::ChatMessage]
    }

    fn handle_event(
        &mut self,
        event: &twitcheventsub::Event,
        api: &mut twitcheventsub::TwitchEventSubApi,
    ) -> Result<()> {
        if let twitcheventsub::Event::ChatMessage(message) = event {
            // TODO: Make issue on twitcheventsub for this functionality
            //if message.chatter.id == api.client_id() {
            //    return Ok(());
            //}
            if message.chatter.id == API_BOT_USER_ID {
                return Ok(());
            }
            if message.message.text.to_lowercase().contains("uwu") {
                let _ = api.send_chat_message_with_reply("UwU", Some(message.message_id.clone()));
            }
        };
        Ok(())
    }
}

fn main() -> Result<()> {
    let keys = TwitchKeys::from_secrets_env().unwrap();

    let mut handler_commands = CommandRegistry::initialize()?;
    let mut handlers: Vec<Box<dyn TwitchEventHandler>> = vec![Box::new(TestHandler)];

    let api_builder = TwitchEventSubApi::builder(keys)
        .set_redirect_url("http://localhost:3000")
        .generate_new_token_if_insufficent_scope(true)
        .generate_new_token_if_none(true)
        .generate_access_token_on_expire(true)
        .auto_save_load_created_tokens(".user_token.env", ".refresh_token.env");

    // WARNING: twitcheventsub uses a Vec instead of HashMap to keep track of what events are
    // subscribed to. I have no clue if this will break stuff (hopefully not.)
    let api_builder = api_builder.add_subscriptions(handler_commands.subscribed_events().to_vec());
    let api_builder = handlers.iter().fold(api_builder, |api_builder, handler| {
        api_builder.add_subscriptions(handler.subscribed_events().to_vec())
    });

    let mut api = api_builder.build().unwrap();
    println!("Bot started!");

    loop {
        while let Some(response) = api.receive_single_message(Duration::ZERO) {
            let ResponseType::Event(event) = response else {
                continue;
            };
            handler_commands.handle_event(&event, &mut api)?;
            handlers.iter_mut().try_for_each(|handler| {
                handler.handle_event(&event, &mut api)?;
                Ok::<(), Error>(())
            })?;
        }

        handler_commands.update(&mut api)?;

        std::thread::sleep(Duration::from_millis(1));
    }
}
