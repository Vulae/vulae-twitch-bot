pub mod command;
pub mod commands;
pub mod config;
pub mod twitch_event_handler;

use std::time::Duration;

use anyhow::{Error, Result};
use commands::CommandRegistry;
use config::Config;
use twitch_event_handler::TwitchEventHandler;
use twitcheventsub::{ResponseType, TwitchEventSubApi, TwitchKeys};

fn main() -> Result<()> {
    let config = Config::load()?;

    let keys = TwitchKeys::from_secrets_env().unwrap();

    let mut handler_commands = CommandRegistry::initialize(&config)?;
    let mut handlers: Vec<Box<dyn TwitchEventHandler>> = vec![];

    let api_builder = TwitchEventSubApi::builder(keys)
        .set_redirect_url("http://localhost:3000")
        .generate_new_token_if_insufficent_scope(true)
        .generate_new_token_if_none(true)
        .generate_access_token_on_expire(true)
        .auto_save_load_created_tokens(".user_token.env", ".refresh_token.env");

    // WARNING: twitcheventsub uses a Vec instead of HashSet to keep track of what events are
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
