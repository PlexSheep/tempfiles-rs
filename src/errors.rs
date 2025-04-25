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
}
