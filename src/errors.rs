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
    Config(#[from] ConfigError),
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
    #[error("This file does not have a database entry")]
    FileDBEntryNotFound,
    #[error("Could not parse FileID: {0}")]
    BadFileID(#[from] ParseIntError),
    #[error("Template Error: {0}")]
    Template(#[from] minijinja::Error),
    #[error("Could not parse variable from url encoding: {0}")]
    UrlEncoding(#[from] FromUtf8Error),
    #[error("Error while detecting the content type of some file: {0}")]
    ContentTypeDetection(#[from] magic::cookie::Error),
    #[error("Internal Error while detecting the content type of some file: {0}")]
    ContentTypeDetectionInternal(#[from] magic::cookie::OpenError),
    #[error("Could not parse content type: {0}")]
    ParseContentType(#[from] mime::FromStrError),
    #[error("Error while working with password hashes: {0}")]
    PwHashing(argon2::password_hash::Error),
    #[error("Wrong login password")]
    WrongPassword,
    #[error("The requested user does not exist")]
    UserDoesNotExist,
    #[error("Validation of a datastructure failed: {0}")]
    Validation(#[from] validator::ValidationErrors),
    #[error("Error while setting Login Parameters: {0}")]
    LogIn(#[from] actix_identity::error::LoginError),
    #[error("Could not get session identity: {0}")]
    SessionIdentity(#[from] actix_identity::error::GetIdentityError),
    #[error("Could not get file metadata: {0}")]
    FileInfos(#[from] crate::files::FileInfosBuilderError),
    #[error("File exists already in the database")]
    FileExists,
    #[error("User not authorized for this action")]
    Unauthorized,
    #[error("Unknown user kind: {0}")]
    UnknownUserKind(String),
    #[error("DB has no salt for a stored token: {0}")]
    NoSaltStoredForToken(String),
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
