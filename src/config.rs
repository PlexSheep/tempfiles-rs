use std::net::SocketAddr;
use std::path::Path;

use actix_web::web;
use log::trace;
use serde::{Deserialize, Serialize};

use crate::errors::ConfigError;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub files: FilesConfig,
    pub accounts: AccountsConfig,
    pub service: ServiceConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FilesConfig {
    pub max_size_kb_anon: u64,
    pub max_size_kb_users: u64,
    pub max_storage_per_user: u64,
    pub max_storage: u64,
    pub storage_dir: String,
    pub default_expiration_days: u64,
    pub delete_old_files: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AccountsConfig {
    /// Allow Anonymous Uploads
    pub allow_anon: bool,
    pub allow_registration: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceConfig {
    #[serde(skip_serializing)]
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

    pub fn largest_possible_upload(&self) -> usize {
        let t = 1024
            * if self.files.max_size_kb_anon < self.files.max_size_kb_users {
                self.files.max_size_kb_users as usize
            } else {
                self.files.max_size_kb_anon as usize
            };
        trace!("largest_possible_upload = {t} Bytes");
        t
    }
}

pub fn actix_config_global(_cfg: &mut web::ServiceConfig) {}
