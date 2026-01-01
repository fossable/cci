use cci_macros::Preset;

mod github;
mod gitea;
mod gitlab;
mod circleci;
mod jenkins;
mod detectable;
mod preset_info;

/// Preset for Go application projects
#[derive(Debug, Clone, Preset)]
#[preset(
    id = "go-app",
    name = "Go App",
    description = "CI pipeline for Go applications with testing and linting",
    matches = "GoApp | GoLibrary"
)]
pub struct GoAppPreset {
    #[preset_field(
        ron_field = "version",
        default = "\"1.21\".to_string()",
        hidden = true
    )]
    pub(super) go_version: String,

    #[preset_field(
        feature = "linting",
        feature_display = "Linting",
        feature_description = "Code quality checks with golangci-lint",
        display = "Enable Linter",
        description = "Run golangci-lint for code quality",
        default = "true"
    )]
    pub(super) enable_linter: bool,

    #[preset_field(
        id = "enable_security",
        feature = "security",
        feature_display = "Security",
        feature_description = "Security vulnerability scanning",
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
