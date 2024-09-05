use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub telegram: TelegramConfig,
    pub seatalk: SeatalkConfig,
}

#[derive(Debug, Deserialize)]
pub struct TelegramConfig {
    pub api_token: String,
}

#[derive(Debug, Deserialize)]
pub struct SeatalkConfig {
    pub host: String,
    pub app_id: String,
    pub app_secret: String,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config/production.yml"))
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?;

        s.try_deserialize()
    }
}
