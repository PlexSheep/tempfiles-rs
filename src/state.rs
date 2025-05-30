use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use log::{debug, error, info, warn};
use migrations::{MigratorTrait, SchemaManager};
use rand::{Rng, SeedableRng};
use sea_orm::{Database, DatabaseConnection, EntityTrait as _};
use tokio::sync::Mutex;

use crate::config::Config;
use crate::db::schema;
use crate::db::schema::file::{Entity as FileE, Model as FileM};
use crate::errors::Error;
use crate::files::{FileID, FileInfos};
use crate::user::User;

const MAX_FID_RETRIES: u32 = 20;

pub struct AppState {
    pub(crate) db: DatabaseConnection, // NOTE: closed on drop
    pub(crate) config: Config,
    pub(crate) csprng: Mutex<rand::rngs::StdRng>,
    template_reloader: minijinja_autoreload::AutoReloader,
}

impl AppState {
    pub async fn new(config: &Config) -> Result<Self, Error> {
        let csprng = rand::rngs::StdRng::from_os_rng();

        let mut templates_path = config.service.data_dir.clone();
        templates_path.push("templates");

        let template_reloader = minijinja_autoreload::AutoReloader::new(move |notifier| {
            let mut env = minijinja::Environment::new();
            env.set_loader(minijinja::path_loader(&templates_path));
            notifier.watch_path(&templates_path, true);
            Ok(env)
        });

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
            template_reloader,
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

    pub fn templating<'a>(&'a self) -> Result<minijinja_autoreload::EnvironmentGuard<'a>, Error> {
        // PERF: disable template reloading in release build
        Ok(self.template_reloader.acquire_env()?)
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

    pub fn upload_dir_for_fid(
        &self,
        fid: impl Into<FileID>,
        create: bool,
    ) -> Result<PathBuf, Error> {
        let fid: FileID = fid.into();
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

    pub async fn make_file_infos(&self, fid: FileID, name: &str) -> Result<FileInfos, Error> {
        let path = &self.upload_datafile_for_fid(fid, name, false)?;
        let flags = magic::cookie::Flags::MIME_TYPE | magic::cookie::Flags::MIME_ENCODING;
        let cookie = magic::Cookie::open(flags)?;
        let cookie = cookie
            .load(&magic::cookie::DatabasePaths::default())
            .expect("could not load database for libmagic file type detection");

        Ok(FileInfos::builder()
            .fid(fid)
            .name(name.to_owned())
            .url_raw(self.uri_api_file_fid_name(fid, name).to_string())
            .url_infos(self.uri_api_file_fid_name_info(fid, name).to_string())
            .url_frontend(self.uri_frontend_file_fid_name(fid, name).to_string())
            .content_type(
                mime::Mime::from_str(&cookie.file(path).unwrap_or("unknown".to_string()))
                    .unwrap_or(mime::APPLICATION_OCTET_STREAM)
                    .to_string(),
            )
            .filemeta(path)?
            .get_db_info(self.db(), fid)
            .await
            .inspect_err(|e| error!("Could not get DB info for file with id {fid}: {e}"))?
            .build()?)
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

    pub async fn get_file_db_entry(
        &self,
        fid: FileID,
        db: &DatabaseConnection,
    ) -> Result<Option<schema::file::Model>, Error> {
        Ok(crate::db::schema::prelude::File::find_by_id(fid.inner())
            .one(db)
            .await?)
    }

    pub fn get_expiration_offset(&self) -> chrono::TimeDelta {
        #[cfg(not(feature = "devel-quickcycle"))]
        {
            chrono::TimeDelta::days(self.config().files.default_expiration_days as i64)
        }
        #[cfg(feature = "devel-quickcycle")]
        {
            chrono::TimeDelta::minutes(3)
        }
    }

    pub async fn create_file_db_entry(
        &self,
        fid: FileID,
        user: Option<&User>,
        db: &DatabaseConnection,
    ) -> Result<(), Error> {
        if let Some(ent) = self.get_file_db_entry(fid, db).await? {
            warn!("Tried to insert file that already existed: {}", ent.id);
            return Err(Error::FileExists);
        }

        let expiration = chrono::Utc::now().naive_utc() + self.get_expiration_offset();

        let user_id = match user {
            Some(u) => sea_orm::ActiveValue::Set(u.id()),
            None => sea_orm::ActiveValue::NotSet,
        };
        let file_values = schema::file::ActiveModel {
            id: sea_orm::ActiveValue::Set(fid.inner()),
            user_id,
            expiration_time: sea_orm::ActiveValue::Set(expiration),
        };

        crate::db::schema::file::Entity::insert(file_values)
            .exec(db)
            .await?;

        #[cfg(debug_assertions)]
        {
            let file_entry = crate::db::schema::prelude::File::find_by_id(fid.inner())
                .one(db)
                .await?;
            assert!(file_entry.is_some());
            debug!("newly created file_db_entry actually exists :)")
        }

        Ok(())
    }
    pub fn garbage_collection_duration(&self) -> tokio::time::Duration {
        #[cfg(not(feature = "devel-quickcycle"))]
        {
            tokio::time::Duration::from_secs(60 * self.config().service.clear_interval as u64)
        }
        #[cfg(feature = "devel-quickcycle")]
        {
            tokio::time::Duration::from_secs(60)
        }
    }

    pub async fn files(&self) -> Result<Vec<FileM>, Error> {
        Ok(FileE::find().all(self.db()).await?)
    }

    pub fn max_upload_size(&self, user: Option<&User>) -> Result<u64, Error> {
        Ok(match user {
            None => self.config().files.max_size_kb_anon * 1024,
            Some(_) => self.config().files.max_size_kb_users * 1024,
        })
    }
}

pub(crate) mod validators {
    use super::*;
    impl AppState {
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
