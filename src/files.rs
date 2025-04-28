use std::fmt::Display;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::str::FromStr;
use std::time::SystemTime;

use actix_multipart::form::{MultipartForm, tempfile::TempFile};
use actix_web::http::Uri;
use actix_web::http::header::ContentType;
use chrono::Utc;
use rand::distr::StandardUniform;
use rand::prelude::*;
use serde::{Deserialize, Serialize, Serializer};

use crate::errors::Error;

#[derive(Debug, MultipartForm)]
pub struct FileUpload {
    #[multipart(limit = "1GB")]
    pub file: TempFile,
}

#[derive(Debug, Serialize)]
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
    #[serde(serialize_with = "ser_systime")]
    pub time_modified: SystemTime,
    #[serde(serialize_with = "ser_systime")]
    pub time_accessed: SystemTime,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct SerializeableContentType {
    inner: String,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FileID {
    inner: u64,
}

impl FileInfos {
    pub fn build(
        fid: FileID,
        name: &str,
        uri_raw: Uri,
        uri_infos: Uri,
        uri_frontend: Uri,
        path: &Path,
        content_type: mime::Mime,
    ) -> Result<Self, Error> {
        let fsmeta = std::fs::metadata(path)?;

        let infos = Self {
            fid,
            name: name.to_string(),
            url_raw: uri_raw.to_string(),
            url_infos: uri_infos.to_string(),
            url_frontend: uri_frontend.to_string(),
            size: fsmeta.size(),
            time_created: fsmeta.created()?,
            time_modified: fsmeta.modified()?,
            time_accessed: fsmeta.accessed()?,
            content_type: content_type.to_string(),
            size_human: human_bytes::human_bytes(fsmeta.size() as u32),
        };

        Ok(infos)
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

fn ser_systime<S: Serializer>(time: &SystemTime, s: S) -> Result<S::Ok, S::Error> {
    let datetime: chrono::DateTime<Utc> = chrono::DateTime::from(*time);
    format!("{datetime}").serialize(s)
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
