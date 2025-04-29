use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use log::{debug, error, info};
use migrations::{MigratorTrait, SchemaManager};
use rand::{Rng, SeedableRng};
use sea_orm::{Database, DatabaseConnection};
use tokio::sync::Mutex;

use crate::config::Config;
use crate::errors::Error;
use crate::files::{FileID, FileInfos};

const MAX_FID_RETRIES: u32 = 20;

#[derive(Debug)]
pub struct AppState<'templates> {
    pub(crate) db: DatabaseConnection, // NOTE: closed on drop
    pub(crate) config: Config,
    pub(crate) csprng: Mutex<rand::rngs::StdRng>,
    pub(crate) templating: minijinja::Environment<'templates>,
}

impl AppState<'_> {
    pub async fn new(config: &Config) -> Result<Self, Error> {
        let csprng = rand::rngs::StdRng::from_os_rng();

        let mut templates_path = config.service.data_dir.clone();
        templates_path.push("templates");
        let mut templates = minijinja::Environment::new();
        templates.set_loader(minijinja::path_loader(&templates_path));

        let db_path = std::path::PathBuf::from(&config.service.db_sqlite);
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?
        }
        let db_url = format!("sqlite:{}?mode=rwc", config.service.db_sqlite);
        debug!("DB url: {db_url}");
        let db = Database::connect(db_url).await?;

        let a = AppState {
            db,
            config: config.clone(),
            csprng: Mutex::new(csprng),
            templating: templates,
        };
        a.run_migrations_if_needed().await?;

        a.validate().await?;
        Ok(a)
    }

    async fn run_migrations_if_needed(&self) -> Result<(), Error> {
        migrations::Migrator::status(&self.db).await?;
        info!("running all pending migrations on database...");
        migrations::Migrator::up(&self.db, None).await?;
        Ok(())
    }

    async fn validate(&self) -> Result<(), Error> {
        info!("validating file storage...");
        if !self.storage_dir().exists() {
            std::fs::create_dir_all(self.storage_dir())?;
        }
        if !self.storage_dir().is_dir() {
            return Err(Error::StorageDirNotADir);
        }

        self.validate_make_testfile()?;

        info!("validating the database...");
        self.db().ping().await?;
        self.validate_db_tables_exist().await?;

        info!("finished validations");

        Ok(())
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn templating(&self) -> &minijinja::Environment {
        &self.templating
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
        for _ in 0..MAX_FID_RETRIES {
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
        let mut path = self.storage_dir();
        path.push(fid.to_string());

        path.exists()
    }

    pub fn upload_dir_for_fid(&self, fid: FileID, create: bool) -> Result<PathBuf, Error> {
        let mut p = self.storage_dir();
        p.push(fid.to_string());
        if create {
            if let Err(e) = std::fs::create_dir(&p) {
                if !matches!(e.kind(), std::io::ErrorKind::AlreadyExists) {
                    return Err(e.into());
                }
            }
        }
        Ok(p)
    }

    pub fn upload_datafile_for_fid(
        &self,
        fid: FileID,
        name: &str,
        create_dirs: bool,
    ) -> Result<PathBuf, Error> {
        let mut p = self.storage_dir();
        p.push(fid.to_string());
        p.push("data");
        if create_dirs {
            if let Err(e) = std::fs::create_dir_all(&p) {
                if !matches!(e.kind(), std::io::ErrorKind::AlreadyExists) {
                    return Err(e.into());
                }
            }
        }
        p.push(name);
        Ok(p)
    }

    pub fn make_file_infos(&self, fid: FileID, name: &str) -> Result<FileInfos, Error> {
        let path = &self.upload_datafile_for_fid(fid, name, false)?;
        let flags = magic::cookie::Flags::MIME_TYPE | magic::cookie::Flags::MIME_ENCODING;
        let cookie = magic::Cookie::open(flags)?;
        let cookie = cookie
            .load(&magic::cookie::DatabasePaths::default())
            .expect("could not load database for libmagic file type detection");
        FileInfos::build(
            fid,
            name,
            self.uri_api_file_fid_name(fid, name),
            self.uri_api_file_fid_name_info(fid, name),
            self.uri_frontend_file_fid_name(fid, name),
            path,
            mime::Mime::from_str(&cookie.file(path).unwrap_or("unknown".to_string()))
                .unwrap_or(mime::APPLICATION_OCTET_STREAM),
            None, // TODO: somehow add user
        )
    }

    pub fn get_filename_for_fid(&self, fid: FileID) -> Result<String, Error> {
        let mut path: PathBuf = self.upload_dir_for_fid(fid, false)?;
        path.push("data");
        debug!("fid path: {}", path.display());
        if !path.exists() {
            debug!("does not exist");
            return Err(Error::FileNotFound);
        }

        let count_items = path.read_dir().into_iter().count();
        if count_items != 1 {
            error!("items in the directory: {count_items:?}");
            return Err(Error::NotOneFileInStorageDir(count_items));
        }
        let mut dir_ents: std::fs::ReadDir = path.read_dir()?;

        let item: Result<_, std::io::Error> = dir_ents
            .next()
            .expect("No dirent despite count_items being 1");
        if let Err(e) = item {
            return Err(e.into());
        }
        let name = item
            .as_ref()
            .unwrap()
            .file_name()
            .to_string_lossy()
            .to_string();
        Ok(name)
    }
}

pub(crate) mod validators {
    use super::*;
    impl AppState<'_> {
        pub(crate) fn validate_make_testfile(&self) -> Result<(), Error> {
            debug!("validate_make_testfile");
            const TESTDATA: &[u8] = &[19, 13, 124, 25, 16, 2, 16, 37, 38, 84, 38, 92, 125, 15];

            let mut testfile_path = self.storage_dir();
            testfile_path.push("___testfile");
            let mut testfile = std::fs::File::create(&testfile_path)?;
            testfile.write_all(TESTDATA)?;
            testfile.sync_all()?;

            let content = std::fs::read(&testfile_path)?;
            if TESTDATA != content {
                panic!(
                    "validate_make_testfile: content written to the testfile does not match the constant TESTDATA"
                )
            }

            std::fs::remove_file(&testfile_path)?;
            Ok(())
        }

        pub(crate) async fn validate_db_tables_exist(&self) -> Result<(), Error> {
            debug!("validate_db_tables_exist");
            let schema_manager = SchemaManager::new(self.db());

            if !schema_manager.has_table("user").await? {
                panic!("validate_db_tables_exist: table 'user' is missing")
            }

            Ok(())
        }

        pub(crate) fn validate_config_base_url(&self) -> Result<(), Error> {
            todo!()
        }
    }
}

pub fn load_config(config_file_path: impl Into<PathBuf>) -> Result<Config, Error> {
    let config_file_path: PathBuf = config_file_path.into();
    let config = Config::load(&config_file_path)?;

    Ok(config)
}
