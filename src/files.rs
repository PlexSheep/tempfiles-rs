use std::fmt::Display;
use std::str::FromStr;

use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text as MpText};
use actix_web::http::header::ContentType;
use rand::distr::StandardUniform;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, MultipartForm)]
pub struct FileUpload {
    pub name: MpText<String>,
    #[multipart(limit = "1GB")]
    pub file: TempFile,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct SerializeableContentType {
    inner: String,
}

#[derive(Debug)]
pub struct FileID {
    inner: u64,
}

impl Display for FileID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:20}", self.inner)
    }
}

impl Distribution<FileID> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> FileID {
        FileID {
            inner: rng.random(),
        }
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
