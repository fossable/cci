use crate::traits::PresetInfo;
use cci_macros::Preset;

mod circleci;
mod detectable;
mod gitea;
mod github;
mod gitlab;
mod jenkins;

/// CI pipeline for Go applications with testing and linting
#[derive(Debug, Clone, Preset)]
#[preset(category = "Languages")]
pub struct GoApp {
    #[preset_field(default = "\"1.21\".to_string()", hidden = true)]
    pub(super) go_version: String,

    /// Run golangci-lint for code quality
    #[preset_field(display = "Enable Linter", default = "true")]
    pub(super) enable_linter: bool,

    /// Run gosec for security vulnerabilities
    #[preset_field(display = "Security Scan", default = "true")]
    pub(super) enable_security_scan: bool,
}

impl GoApp {
    /// Constant default instance for registry initialization
    pub const DEFAULT: Self = Self {
        go_version: String::new(),
        enable_linter: false,
        enable_security_scan: false,
    };
}

impl PresetInfo for GoApp {
    fn name(&self) -> &str {
        "GoApp"
    }

    fn description(&self) -> &str {
        "CI pipeline for Go applications with testing and linting"
    }
}
