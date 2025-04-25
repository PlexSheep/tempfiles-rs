use std::path::PathBuf;
use std::sync::Arc;

use actix_web::web;
use sea_orm::{Database, DatabaseConnection};

use crate::config::Config;
use crate::errors::Error;

#[derive(Debug, Clone)]
pub struct AppState {
    db: DatabaseConnection, // NOTE: never explicitly closed
    config: Config,
}

impl AppState {
    pub async fn new(config_file_path: impl Into<PathBuf>) -> Result<Self, Error> {
        let config_file_path: PathBuf = config_file_path.into();
        let config = Config::load(&config_file_path)?;

        let db_path = std::path::PathBuf::from(&config.service.db_sqlite);
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?
        }
        let db_url = format!("sqlite:{}?mode=rwc", config.service.db_sqlite);
        let db = Database::connect(db_url).await?;

        Ok(AppState { db, config })
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}
