use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{error::Result, utils::get_config_dir};
use std::fs;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub hydrus: HydrusConfig,
    pub saucenao: Option<SauceNaoConfig>,
    pub twitter: Option<TwitterConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HydrusConfig {
    pub api_url: String,
    pub api_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SauceNaoConfig {
    pub api_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TwitterConfig {
    pub consumer_key: String,
    pub consumer_secret: String,
}

impl Config {
    pub fn read() -> Result<Self> {
        let config_dir = get_config_dir()?;
        let config_file_path = config_dir.join(PathBuf::from("config.toml"));

        if !config_file_path.exists() {
            fs::write(&config_file_path, include_str!("assets/config.toml"))?;
        }
        let mut builder = config::Config::builder()
            .add_source(config::File::with_name(config_file_path.to_str().unwrap()));

        let local_config = PathBuf::from(".hydrus-utils.toml");
        if local_config.exists() {
            builder = builder.add_source(config::File::with_name(".hydrus-utils.toml"));
        }
        let settings = builder.build()?;
        tracing::debug!("Config is {settings:?}");

        Ok(settings.try_deserialize()?)
    }

    /// Returns the saucenao configuratio or panics if nothing is configured
    pub fn into_saucenao(self) -> SauceNaoConfig {
        self.saucenao
            .expect("No saucenao key configured. Please add one to the config file.")
    }
}
