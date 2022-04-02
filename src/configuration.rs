use serde::Deserialize;
use config::{ConfigError, Config, File};
use std::convert::{TryFrom, TryInto};

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let mut settings = Config::builder();
    settings = settings.add_source(File::with_name("configuration"));
    settings.try_into()
}