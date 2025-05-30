use std::fmt::Display;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::str::FromStr;
use std::time::SystemTime;

use actix_multipart::form::{MultipartForm, tempfile::TempFile};
use actix_web::http::header::ContentType;
use chrono::{NaiveDateTime, Utc};
use derive_builder::Builder;
use log::{debug, warn};
use rand::distr::StandardUniform;
use rand::prelude::*;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize, Serializer};

use crate::db::types::RawFileID;
use crate::errors::Error;
use crate::user::User;

#[derive(Debug, MultipartForm)]
pub struct FileUpload {
    #[multipart(limit = "10GB")]
    pub file: TempFile,
}

#[derive(Debug, Serialize, Builder)]
pub struct FileInfos {
    pub fid: FileID,
    pub name: String,
    pub url_raw: String,
    pub url_infos: String,
    pub url_frontend: String,
    pub size: u64,
    /// human readable size
    pub size_human: String,
    pub content_type: String,
    pub time_created: NaiveDateTime,
    #[serde(serialize_with = "ser_uploader")]
    pub uploader: Option<User>,
    pub time_modified: NaiveDateTime,
    pub time_accessed: NaiveDateTime,
    pub time_expiration: NaiveDateTime,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct SerializeableContentType {
    inner: String,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FileID {
    inner: RawFileID,
}

impl FileID {
    pub fn inner(&self) -> RawFileID {
        self.inner
    }
}

impl FileInfos {
    pub fn builder() -> FileInfosBuilder {
        FileInfosBuilder::default()
    }

    pub fn content_type(&self) -> Result<mime::Mime, Error> {
        Ok(mime::Mime::from_str(&self.content_type)?)
    }
}

impl FileInfosBuilder {
    pub fn filemeta(&mut self, path: &Path) -> Result<&mut Self, Error> {
        let fsmeta = std::fs::metadata(path)?;

        self.size(fsmeta.size())
            .time_created(systime_to_chrono(fsmeta.created()?))
            .time_modified(systime_to_chrono(fsmeta.modified()?))
            .time_accessed(systime_to_chrono(fsmeta.accessed()?))
            .size_human(human_bytes::human_bytes(fsmeta.size() as u32));

        Ok(self)
    }

    pub async fn get_db_info(
        &mut self,
        db: &sea_orm::DatabaseConnection,
        fid: FileID,
    ) -> Result<&mut Self, Error> {
        let file_entry = crate::db::schema::prelude::File::find_by_id(fid.inner())
            .one(db)
            .await?;
        if file_entry.is_none() {
            return Err(Error::FileDBEntryNotFound);
        }
        let file_meta = file_entry.unwrap();

        let user = match User::get_by_id(file_meta.user_id, db).await {
            Ok(user) => Some(user),
            Err(Error::UserDoesNotExist) => {
                warn!("Uploader with unknown user id: {}", file_meta.user_id);
                None
            }
            Err(other) => return Err(other),
        };

        debug!("User for fent: {user:?}");

        self.uploader(user);
        self.time_expiration(file_meta.expiration_time);

        Ok(self)
    }
}

impl Display for FileID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl Distribution<FileID> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> FileID {
        FileID {
            inner: rng.random(),
        }
    }
}

impl FromStr for FileID {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(FileID { inner: s.parse()? })
    }
}

impl From<ContentType> for SerializeableContentType {
    fn from(value: ContentType) -> Self {
        SerializeableContentType {
            inner: value.to_string(),
        }
    }
}

impl From<mime::Mime> for SerializeableContentType {
    fn from(value: mime::Mime) -> Self {
        SerializeableContentType {
            inner: value.to_string(),
        }
    }
}

impl From<SerializeableContentType> for ContentType {
    fn from(value: SerializeableContentType) -> Self {
        Self(mime::Mime::from_str(&value.to_string()).unwrap())
    }
}

impl Display for SerializeableContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl From<FileID> for RawFileID {
    fn from(value: FileID) -> Self {
        value.inner()
    }
}

impl From<RawFileID> for FileID {
    fn from(inner: RawFileID) -> Self {
        FileID { inner }
    }
}

fn ser_uploader<S: Serializer>(user: &Option<User>, s: S) -> Result<S::Ok, S::Error> {
    debug!("serializing user for file entry: {user:?}");
    match user {
        None => "Anonymous".serialize(s),
        Some(user) => user.username().serialize(s),
    }
}

fn systime_to_chrono(t: SystemTime) -> chrono::NaiveDateTime {
    let datetime: chrono::DateTime<Utc> = chrono::DateTime::from(t);
    datetime.naive_utc()
}

#[cfg(test)]
mod test {
    use super::FileID;

    #[test]
    fn test_fid() {
        let mut fid: FileID;
        for _ in 0..1000 {
            fid = rand::random();
            assert!(!fid.to_string().contains(" "))
        }
    }
}
