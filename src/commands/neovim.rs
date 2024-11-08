// TODO: Have the client respond if theme is invalid
// TODO: !theme random, to set theme to random installed theme.

use std::{
    io::Write,
    net::{TcpListener, TcpStream},
};

use anyhow::Result;
use twitcheventsub::{MessageData, TwitchEventSubApi};

use crate::command::{Command, CommandArgsResult};

pub struct Neovim {
    server: TcpListener,
    connections: Vec<TcpStream>,
}

impl Neovim {
    pub fn initialize() -> Result<Self> {
        let server = TcpListener::bind("127.0.0.1:24694")?;
        server.set_nonblocking(true)?;
        Ok(Self {
            server,
            connections: Vec::new(),
        })
    }

    pub fn send(&mut self, message: &[u8]) {
        self.connections.retain_mut(|connection| {
            if let Err(err) = connection.write_all(message) {
                println!("TCP connection send error: {:#?}", err);
                return err.kind() != std::io::ErrorKind::BrokenPipe;
            }
            true
        });
    }
}

#[derive(Debug, Clone)]
pub enum NeovimArgs {
    SetTheme(String),
}

impl Command<NeovimArgs> for Neovim {
    fn parse_args(&self, chat_message: &MessageData) -> CommandArgsResult<NeovimArgs> {
        let mut split = chat_message.message.text.split(" ");
        match split.next() {
            Some("!theme") | Some("!settheme") | Some("!colorscheme") => {
                let Some(theme) = split.next() else {
                    return CommandArgsResult::BadArguments("Usage: !theme [theme]".to_owned());
                };
                CommandArgsResult::Execute(NeovimArgs::SetTheme(theme.to_owned()))
            }
            _ => CommandArgsResult::WrongCommand,
        }
    }

    #[allow(unused)]
    fn execute(
        &mut self,
        args: NeovimArgs,
        chat_message: &MessageData,
        api: &mut TwitchEventSubApi,
    ) -> Result<()> {
        match args {
            NeovimArgs::SetTheme(theme) => self.send(format!("set_theme {}", theme).as_bytes()),
        }
        Ok(())
    }

    fn update(&mut self, _api: &mut TwitchEventSubApi) -> Result<()> {
        if let Some(stream) = self.server.incoming().next() {
            match stream {
                Ok(connection) => {
                    self.connections.push(connection);
                    println!("New connection!");
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(err) => println!("TCP connection error: {:#?}", err),
            }
        }

        Ok(())
    }
}
