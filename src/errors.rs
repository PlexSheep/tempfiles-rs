use actix_web::http::header::HeaderValue;
use log::{error, warn};
use std::num::ParseIntError;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("The config file was not found")]
    FileNotFound,
    #[error("The path of the config file is not a file")]
    NotAFile,
    #[error("Could not read the config file: {0}")]
    CouldNotReadFile(#[from] std::io::Error),
    #[error("Bad TOML in the config file: {0}")]
    ConfigSyntax(#[from] toml::de::Error),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ConfigError(#[from] ConfigError),
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("DB Error: {0}")]
    Db(#[from] sea_orm::DbErr),
    #[error("The path of the storage directory is not a directory")]
    StorageDirNotADir,
    #[error("A storage directory is holding more or less than one file: {0}")]
    NotOneFileInStorageDir(usize),
    #[error("This file does not exist on this server")]
    FileNotFound,
    #[error("Could not parse FileID: {0}")]
    BadFileID(#[from] ParseIntError),
    #[error("Template Error: {0}")]
    Template(#[from] minijinja::Error),
    #[error("Could not parse variable from url encoding: {0}")]
    UrlEncoding(#[from] FromUtf8Error),
}

impl actix_web::error::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::FileNotFound => actix_web::http::StatusCode::NOT_FOUND,
            Self::BadFileID(_) => actix_web::http::StatusCode::BAD_REQUEST,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let mut res = actix_web::HttpResponse::new(self.status_code());
        warn!("Returning Error ({}): {self}", res.status());

        res.headers_mut().insert(
            actix_web::http::header::CONTENT_TYPE,
            // HACK: this conversion is probably unneeded
            HeaderValue::from_str(mime::TEXT_PLAIN_UTF_8.as_ref()).unwrap(),
        );

        res.set_body(actix_web::body::BoxBody::new(format!("{self}")))
    }
}
