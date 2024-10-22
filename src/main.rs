pub mod command;
pub mod commands;

use std::time::Duration;

use anyhow::Result;
use commands::CommandRegistry;
use twitcheventsub::{ResponseType, Subscription, TwitchEventSubApi, TwitchKeys};

fn main() -> Result<()> {
    let keys = TwitchKeys::from_secrets_env().unwrap();

    let mut api = TwitchEventSubApi::builder(keys)
        .set_redirect_url("http://localhost:3000")
        .generate_new_token_if_insufficent_scope(true)
        .generate_new_token_if_none(true)
        .generate_access_token_on_expire(true)
        .auto_save_load_created_tokens(".user_token.env", ".refresh_token.env")
        .add_subscription(Subscription::ChatMessage)
        .build()
        .unwrap();

    let mut commands = CommandRegistry::initialize()?;

    loop {
        while let Some(response) = api.receive_single_message(Duration::ZERO) {
            let ResponseType::Event(event) = response else {
                continue;
            };
            match event {
                twitcheventsub::Event::ChatMessage(chat_message) => {
                    commands.try_execute(&chat_message, &mut api)?;
                }
                _ => println!("Unimplemented event handling: {:#?}", event),
            }
        }

        commands.update(&mut api)?;

        std::thread::sleep(Duration::from_millis(1));
    }
}
