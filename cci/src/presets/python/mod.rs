use crate::traits::PresetInfo;
use cci_macros::{Preset, PresetEnum};

mod circleci;
mod detectable;
mod gitea;
mod github;
mod gitlab;
mod jenkins;

/// Linter tool options for Python
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, PresetEnum)]
#[preset_enum(default = "Flake8")]
#[serde(rename_all = "lowercase")]
pub enum PythonLinter {
    #[preset_variant(id = "flake8", display = "Flake8")]
    Flake8,
    #[preset_variant(id = "ruff", display = "Ruff")]
    Ruff,
}

impl PythonLinter {
    pub fn name(&self) -> &'static str {
        match self {
            PythonLinter::Flake8 => "flake8",
            PythonLinter::Ruff => "ruff",
        }
    }

    pub fn check_command(&self) -> &'static str {
        match self {
            PythonLinter::Flake8 => "flake8 .",
            PythonLinter::Ruff => "ruff check .",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            PythonLinter::Flake8 => PythonLinter::Ruff,
            PythonLinter::Ruff => PythonLinter::Flake8,
        }
    }
}

/// Formatter tool options for Python
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, PresetEnum)]
#[preset_enum(default = "Black")]
#[serde(rename_all = "lowercase")]
pub enum PythonFormatter {
    #[preset_variant(id = "black", display = "Black")]
    Black,
    #[preset_variant(id = "ruff", display = "Ruff")]
    Ruff,
}

impl PythonFormatter {
    pub fn name(&self) -> &'static str {
        match self {
            PythonFormatter::Black => "black",
            PythonFormatter::Ruff => "ruff",
        }
    }

    pub fn check_command(&self) -> &'static str {
        match self {
            PythonFormatter::Black => "black --check .",
            PythonFormatter::Ruff => "ruff format --check .",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            PythonFormatter::Black => PythonFormatter::Ruff,
            PythonFormatter::Ruff => PythonFormatter::Black,
        }
    }
}

/// Preset for Python application projects
#[derive(Debug, Clone, Preset)]
#[preset(
    id = "python-app",
    name = "Python",
    description = "CI pipeline for Python applications with pytest, linting, and type checking",
    matches = "PythonApp | PythonLibrary"
)]
pub struct PythonAppPreset {
    #[preset_field(default = "\"3.11\".to_string()", hidden = true)]
    pub(super) python_version: String,

    #[preset_field(
        feature = "linting",
        feature_display = "Linting",
        display = "Linter",
        description = "Choose linter tool (None, Flake8, or Ruff)",
        default = "None"
    )]
    pub(super) linter: Option<PythonLinter>,

    #[preset_field(
        feature = "testing",
        feature_display = "Testing",
        display = "Type Checking",
        description = "Enable mypy static type checking",
        default = "false"
    )]
    pub(super) enable_type_check: bool,

    #[preset_field(
        feature = "formatting",
        feature_display = "Formatting",
        display = "Formatter",
        description = "Choose formatter tool (None, Black, or Ruff)",
        default = "None"
    )]
    pub(super) formatter: Option<PythonFormatter>,
}

impl PythonAppPreset {
    /// Constant default instance for registry initialization
    pub const DEFAULT: Self = Self {
        python_version: String::new(),
        linter: None,
        enable_type_check: false,
        formatter: None,
    };
}

impl PresetInfo for PythonAppPreset {
    fn name(&self) -> &str {
        "python-app"
    }

    fn description(&self) -> &str {
        "CI pipeline for Python applications with pytest, linting, and type checking"
    }
}
