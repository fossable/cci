use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML serialization error: {0}")]
    YamlSerialization(#[from] serde_yaml::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDeserialization(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerialization(#[from] toml::ser::Error),

    #[error("Cargo.toml parsing error: {0}")]
    CargoToml(String),

    #[error("Project detection failed: no matching project type found")]
    DetectionFailed,

    #[error("Platform '{0}' is not supported")]
    UnsupportedPlatform(String),

    #[error("Preset '{0}' not found")]
    PresetNotFound(String),

    #[error("File already exists: {0}")]
    FileExists(PathBuf),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("User cancelled operation")]
    UserCancelled,
}
