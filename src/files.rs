use std::fmt::Display;
use std::path::Path;
use std::str::FromStr;

use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text as MpText};
use actix_web::http::Uri;
use actix_web::http::header::ContentType;
use rand::distr::StandardUniform;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::errors::Error;

#[derive(Debug, MultipartForm)]
pub struct FileUpload {
    pub name: MpText<String>,
    #[multipart(limit = "1GB")]
    pub file: TempFile,
}

#[derive(Debug, Serialize)]
pub struct FileInfos {
    pub fid: FileID,
    pub name: String,
    pub uri: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct SerializeableContentType {
    inner: String,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct FileID {
    inner: u64,
}

impl FileInfos {
    pub fn build(fid: FileID, name: &str, uri: Uri, path: &Path) -> Result<Self, Error> {
        let infos = Self {
            fid,
            name: name.to_string(),
            uri: uri.to_string(),
        };

        Ok(infos)
    }
}

impl Display for FileID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:020}", self.inner)
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
