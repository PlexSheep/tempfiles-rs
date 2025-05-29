use std::net::SocketAddr;
use std::path::Path;

use actix_web::web;
use serde::Deserialize;

use crate::errors::ConfigError;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub files: FilesConfig,
    pub accounts: AccountsConfig,
    pub service: ServiceConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FilesConfig {
    pub max_size_kb_anon: u64,
    pub max_size_kb_users: u64,
    pub max_storage_per_user: u64,
    pub max_storage: u64,
    pub storage_dir: String,
    pub default_expiration_days: u64,
    pub delete_old_files: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AccountsConfig {
    pub allow_anon: bool,
    pub allow_registration: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServiceConfig {
    pub secret: String,
    pub rate_limit_window_ms: u64,
    pub rate_limit_max_uploads: u64,
    pub db_sqlite: String,
    pub bind: SocketAddr,
    pub base_url: String,
    pub data_dir: std::path::PathBuf,
    pub clear_interval: u16,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::FileNotFound);
        }
        if !path.is_file() {
            return Err(ConfigError::NotAFile);
        }
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}

pub fn actix_config_global(_cfg: &mut web::ServiceConfig) {
    // todo!()
}
