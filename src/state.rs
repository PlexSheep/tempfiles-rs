use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use rand::{Rng, SeedableRng};
use sea_orm::{Database, DatabaseConnection};
use tokio::sync::Mutex;

use crate::config::Config;
use crate::errors::Error;
use crate::files::FileID;

const MAX_FID_RETRIES: u32 = 20;

#[derive(Debug)]
pub struct AppState {
    db: DatabaseConnection, // NOTE: closed on drop
    config: Config,
    csprng: Mutex<rand::rngs::StdRng>,
}

impl AppState {
    pub async fn new(config_file_path: impl Into<PathBuf>) -> Result<Self, Error> {
        let config_file_path: PathBuf = config_file_path.into();
        let config = Config::load(&config_file_path)?;

        let csprng = rand::rngs::StdRng::from_os_rng();

        let db_path = std::path::PathBuf::from(&config.service.db_sqlite);
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?
        }
        let db_url = format!("sqlite:{}?mode=rwc", config.service.db_sqlite);
        let db = Database::connect(db_url).await?;

        let a = AppState {
            db,
            config,
            csprng: Mutex::new(csprng),
        };
        a.validate().await?;
        Ok(a)
    }

    async fn validate(&self) -> Result<(), Error> {
        if !self.storage_dir().exists() {
            std::fs::create_dir_all(self.storage_dir())?;
        }
        if !self.storage_dir().is_dir() {
            return Err(Error::StorageDirNotADir);
        }

        self.db().ping().await?;

        self.validate_make_testfile()?;

        Ok(())
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn storage_dir(&self) -> PathBuf {
        std::path::PathBuf::from_str(&self.config.files.storage_dir)
            .expect("Infalliable from_str failed")
    }

    pub async fn csprng(&self) -> tokio::sync::MutexGuard<'_, rand::prelude::StdRng> {
        self.csprng.lock().await
    }

    pub async fn new_fid(&self) -> FileID {
        let mut fid: FileID;
        for _ in (0..MAX_FID_RETRIES) {
            fid = self.csprng().await.random();
            if !self.has_fid(fid).await {
                return fid;
            }
        }
        panic!(
            "new_fid: Took more than {MAX_FID_RETRIES} to generate a new file id. The DB is likely behaving weird."
        )
    }

    pub async fn has_fid(&self, fid: FileID) -> bool {
        todo!()
    }
}

impl AppState {
    fn validate_make_testfile(&self) -> Result<(), Error> {
        const TESTDATA: &[u8] = &[19, 13, 124, 25, 16, 2, 16, 37, 38, 84, 38, 92, 125, 15];

        let mut testfile_path = self.storage_dir();
        testfile_path.push("___testfile");
        let mut testfile = std::fs::OpenOptions::new()
            .write(true)
            .append(false)
            .open(&testfile_path)?;
        testfile.write_all(TESTDATA)?;
        drop(testfile);

        let content = std::fs::read(&testfile_path)?;
        if TESTDATA != content {
            panic!(
                "validate_make_testfile: content written to the testfile does not match the constant TESTDATA"
            )
        }

        std::fs::remove_file(&testfile_path)?;
        Ok(())
    }
}
