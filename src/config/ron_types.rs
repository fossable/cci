use serde::{Deserialize, Serialize};

/// Top-level CCI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CciConfig {
    pub presets: Vec<PresetChoice>,
}

/// Preset choice enum - supports all available presets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PresetChoice {
    Python(PythonConfig),
    Rust(RustConfig),
    GoApp(GoAppConfig),
    Docker(DockerConfig),
}

// =============================================================================
// Python Preset Configuration
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonConfig {
    pub version: String,
    #[serde(default)]
    pub linter: LinterConfig,
    #[serde(default)]
    pub formatter: FormatterConfig,
    #[serde(default)]
    pub type_check: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinterConfig {
    pub enabled: bool,
    pub tool: LinterTool,
}

impl Default for LinterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            tool: LinterTool::Flake8,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatterConfig {
    pub enabled: bool,
    pub tool: FormatterTool,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            tool: FormatterTool::Black,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinterTool {
    Flake8,
    Ruff,
}

impl LinterTool {
    pub fn as_str(&self) -> &'static str {
        match self {
            LinterTool::Flake8 => "flake8",
            LinterTool::Ruff => "ruff",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormatterTool {
    Black,
    Ruff,
}

impl FormatterTool {
    pub fn as_str(&self) -> &'static str {
        match self {
            FormatterTool::Black => "black",
            FormatterTool::Ruff => "ruff",
        }
    }
}

// =============================================================================
// Rust Preset Configuration
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustConfig {
    pub version: String,
    #[serde(default)]
    pub coverage: bool,
    #[serde(default)]
    pub linter: bool,
    #[serde(default)]
    pub security: bool,
    #[serde(default)]
    pub formatter: bool,
    #[serde(default)]
    pub build_release: bool,
}

// =============================================================================
// Go App Preset Configuration
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoAppConfig {
    pub version: String,
    #[serde(default)]
    pub linter: bool,
    #[serde(default)]
    pub security: bool,
}

// =============================================================================
// Docker Preset Configuration
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    pub image_name: String,
    #[serde(default)]
    pub registry: DockerRegistryChoice,
    #[serde(default = "default_dockerfile_path")]
    pub dockerfile_path: String,
    #[serde(default = "default_build_context")]
    pub build_context: String,
    #[serde(default)]
    pub build_args: Vec<(String, String)>,
    #[serde(default)]
    pub cache: bool,
    #[serde(default)]
    pub tags_only: bool,
}

fn default_dockerfile_path() -> String {
    "./Dockerfile".to_string()
}

fn default_build_context() -> String {
    ".".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DockerRegistryChoice {
    None,
    DockerHub,
    GitHubRegistry,
}

impl Default for DockerRegistryChoice {
    fn default() -> Self {
        DockerRegistryChoice::None
    }
}

impl DockerRegistryChoice {
    pub fn as_str(&self) -> &'static str {
        match self {
            DockerRegistryChoice::None => "none",
            DockerRegistryChoice::DockerHub => "dockerhub",
            DockerRegistryChoice::GitHubRegistry => "github",
        }
    }
}
