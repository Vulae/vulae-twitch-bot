use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::commands::{radio::RadioConfig, simple_reply::SimpleReplyCommandHandler};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "bot-user-id")]
    pub bot_user_id: String,
    #[serde(rename = "data-file")]
    pub data_file: PathBuf,
    #[serde(skip)]
    data: Option<ConfigData>,
    pub radio: RadioConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigData {
    pub simple_reply_commands: SimpleReplyCommandHandler,
}

impl ConfigData {
    fn load(path: &PathBuf) -> Result<Self> {
        Ok(serde_yml::from_str(&std::fs::read_to_string(path)?)?)
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let mut config: Config = toml::from_str(&std::fs::read_to_string("config.toml")?)?;
        config.data = Some(ConfigData::load(&config.data_file)?);
        Ok(config)
    }

    pub fn data(&self) -> &ConfigData {
        self.data.as_ref().expect("This error should never happen.")
    }
}
