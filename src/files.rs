use std::fmt::Display;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::str::FromStr;
use std::time::SystemTime;

use actix_multipart::form::{MultipartForm, tempfile::TempFile};
use actix_web::http::header::ContentType;
use chrono::{NaiveDateTime, Utc};
use derive_builder::Builder;
use rand::distr::StandardUniform;
use rand::prelude::*;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize, Serializer};

use crate::errors::Error;
use crate::user::User;

#[derive(Debug, MultipartForm)]
pub struct FileUpload {
    #[multipart(limit = "1GB")]
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
    #[serde(serialize_with = "ser_systime")]
    pub time_created: SystemTime,
    #[serde(serialize_with = "ser_uploader")]
    pub uploader: Option<User>,
    #[serde(serialize_with = "ser_systime")]
    pub time_modified: SystemTime,
    #[serde(serialize_with = "ser_systime")]
    pub time_accessed: SystemTime,
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
    inner: i32,
}

impl FileID {
    pub fn inner(&self) -> i32 {
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
            .time_created(fsmeta.created()?)
            .time_modified(fsmeta.modified()?)
            .time_accessed(fsmeta.accessed()?)
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

        let user = match User::get_by_id(file_meta.id, db).await {
            Ok(user) => Some(user),
            Err(Error::UserDoesNotExist) => None,
            Err(other) => return Err(other),
        };

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

impl From<FileID> for i32 {
    fn from(value: FileID) -> Self {
        value.inner()
    }
}

fn ser_systime<S: Serializer>(time: &SystemTime, s: S) -> Result<S::Ok, S::Error> {
    let datetime: chrono::DateTime<Utc> = chrono::DateTime::from(*time);
    format!("{datetime}").serialize(s)
}

fn ser_uploader<S: Serializer>(user: &Option<User>, s: S) -> Result<S::Ok, S::Error> {
    match user {
        None => "Anonymous".serialize(s),
        Some(user) => user.username().serialize(s),
    }
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
