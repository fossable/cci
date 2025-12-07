use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Step {
    Checkout,

    SetupToolchain {
        language: Language,
        version: String,
    },

    Cache {
        paths: Vec<String>,
        key: String,
    },

    RestoreCache {
        key: String,
    },

    RunCommand {
        name: String,
        command: String,
        working_dir: Option<String>,
    },

    InstallDependencies {
        language: Language,
    },

    RunTests {
        language: Language,
        coverage: bool,
    },

    RunLinter {
        language: Language,
        tool: String,
    },

    SecurityScan {
        language: Language,
        tool: String,
    },

    Build {
        language: Language,
        artifact_paths: Vec<String>,
    },

    UploadArtifact {
        name: String,
        paths: Vec<String>,
    },

    UploadCoverage {
        provider: CoverageProvider,
    },

    PublishPackage {
        registry: Registry,
        token_env: String,
    },

    CreateRelease {
        tag_pattern: String,
        artifacts: Vec<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    Python,
    Go,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CoverageProvider {
    Codecov,
    Coveralls,
    CodeClimate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Registry {
    CratesIo,
    PyPI,
    Npm,
}
