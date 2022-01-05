use config::{ConfigError, Config, File};
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub host: String,
    pub port: u32,
    pub ping_frequency_secs: u64,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name("Settings"))?;
        s.try_into()
    }
}
