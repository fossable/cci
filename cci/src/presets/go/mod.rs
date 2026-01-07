use crate::traits::PresetInfo;
use cci_macros::Preset;

mod circleci;
mod detectable;
mod gitea;
mod github;
mod gitlab;
mod jenkins;

/// Preset for Go application projects
#[derive(Debug, Clone, Preset)]
#[preset(
    id = "go-app",
    name = "Go App",
    description = "CI pipeline for Go applications with testing and linting",
    matches = "GoApp | GoLibrary"
)]
pub struct GoAppPreset {
    #[preset_field(default = "\"1.21\".to_string()", hidden = true)]
    pub(super) go_version: String,

    #[preset_field(
        feature = "linting",
        feature_display = "Linting",
        display = "Enable Linter",
        description = "Run golangci-lint for code quality",
        default = "true"
    )]
    pub(super) enable_linter: bool,

    #[preset_field(
        feature = "security",
        feature_display = "Security",
        display = "Security Scan",
        description = "Run gosec for security vulnerabilities",
        default = "true"
    )]
    pub(super) enable_security_scan: bool,
}

impl GoAppPreset {
    /// Constant default instance for registry initialization
    pub const DEFAULT: Self = Self {
        go_version: String::new(),
        enable_linter: false,
        enable_security_scan: false,
    };
}

impl PresetInfo for GoAppPreset {
    fn name(&self) -> &str {
        "go-app"
    }

    fn description(&self) -> &str {
        "CI pipeline for Go applications with testing and linting"
    }
}
