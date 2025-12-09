use std::path::PathBuf;

pub type Result<T> = anyhow::Result<T>;

// Helper functions for creating specific error types with context
pub fn cargo_toml_error(msg: impl Into<String>) -> anyhow::Error {
    anyhow::anyhow!("Cargo.toml parsing error: {}", msg.into())
}

pub fn detection_failed_error() -> anyhow::Error {
    anyhow::anyhow!("Project detection failed: no matching project type found")
}

pub fn unsupported_platform_error(platform: impl Into<String>) -> anyhow::Error {
    anyhow::anyhow!("Platform '{}' is not supported", platform.into())
}

pub fn preset_not_found_error(preset: impl Into<String>) -> anyhow::Error {
    anyhow::anyhow!("Preset '{}' not found", preset.into())
}

pub fn file_exists_error(path: PathBuf) -> anyhow::Error {
    anyhow::anyhow!("File already exists: {}", path.display())
}

pub fn validation_error(msg: impl Into<String>) -> anyhow::Error {
    anyhow::anyhow!("Validation error: {}", msg.into())
}

pub fn config_error(msg: impl Into<String>) -> anyhow::Error {
    anyhow::anyhow!("Configuration error: {}", msg.into())
}

pub fn user_cancelled_error() -> anyhow::Error {
    anyhow::anyhow!("User cancelled operation")
}
