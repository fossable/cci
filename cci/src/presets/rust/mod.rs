use cci_macros::Preset;

mod github;
mod gitea;
mod gitlab;
mod circleci;
mod jenkins;
mod detectable;
mod preset_info;

/// Unified preset for Rust projects (binaries, libraries, and workspaces)
#[derive(Debug, Clone, Preset)]
#[preset(
    id = "rust",
    name = "Rust",
    description = "CI pipeline for Rust projects (binaries, libraries, and workspaces)",
    matches = "RustBinary | RustLibrary | RustWorkspace"
)]
pub struct RustPreset {
    #[preset_field(
        ron_field = "version",
        default = "\"stable\".to_string()",
        hidden = true
    )]
    pub(super) rust_version: String,

    #[preset_field(
        feature = "testing",
        feature_display = "Testing",
        feature_description = "Test coverage reporting",
        display = "Code Coverage",
        description = "Enable code coverage reporting with tarpaulin",
        default = "true"
    )]
    pub(super) enable_coverage: bool,

    #[preset_field(
        feature = "linting",
        feature_display = "Linting",
        feature_description = "Code quality checks",
        display = "Clippy Linter",
        description = "Run Clippy linter for code quality",
        default = "true"
    )]
    pub(super) enable_linter: bool,

    #[preset_field(
        id = "enable_security",
        feature = "security",
        feature_display = "Security",
        feature_description = "Security vulnerability scanning",
        display = "Security Scan",
        description = "Run cargo-audit for dependency vulnerabilities",
        default = "true"
    )]
    pub(super) enable_security_scan: bool,

    #[preset_field(
        id = "enable_formatter",
        feature = "formatting",
        feature_display = "Formatting",
        feature_description = "Code formatting checks",
        display = "Rustfmt Check",
        description = "Check code formatting with rustfmt",
        default = "true"
    )]
    pub(super) enable_format_check: bool,

    #[preset_field(
        feature = "building",
        feature_display = "Building",
        feature_description = "Release binary builds",
        display = "Build Release",
        description = "Build optimized release binary in CI",
        default = "true"
    )]
    pub(super) build_release: bool,
}

impl Default for RustPreset {
    fn default() -> Self {
        Self {
            rust_version: "stable".to_string(),
            enable_coverage: false,
            enable_linter: false,
            enable_security_scan: false,
            enable_format_check: false,
            build_release: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{ToGitHub, PresetInfo};

    #[test]
    fn test_default() {
        let preset = RustPreset::default();
        assert_eq!(preset.rust_version, "stable");
        // Defaults changed - all features are now enabled by default
        assert!(preset.enable_coverage);
        assert!(preset.enable_linter);
        assert!(preset.enable_security_scan);
        assert!(preset.enable_format_check);
        assert!(preset.build_release);
    }

    #[test]
    fn test_with_options() {
        let preset = RustPreset {
            rust_version: "1.75.0".to_string(),
            enable_coverage: true,
            enable_linter: true,
            enable_security_scan: false,
            enable_format_check: false,
            build_release: true,
        };

        assert_eq!(preset.rust_version, "1.75.0");
        assert!(preset.enable_coverage);
        assert!(preset.enable_linter);
        assert!(preset.build_release);
    }

    #[test]
    fn test_to_github_basic() {
        let preset = RustPreset {
            rust_version: "stable".to_string(),
            enable_coverage: false,
            enable_linter: false,
            enable_security_scan: false,
            enable_format_check: false,
            build_release: false,
        };
        let workflow = preset.to_github().unwrap();

        assert_eq!(workflow.name, "CI");
        assert!(workflow.jobs.contains_key("rust/test"));
        assert!(!workflow.jobs.contains_key("rust/lint"));
    }

    #[test]
    fn test_to_github_with_lint() {
        let preset = RustPreset {
            rust_version: "stable".to_string(),
            enable_coverage: false,
            enable_linter: true,
            enable_security_scan: false,
            enable_format_check: false,
            build_release: false,
        };
        let workflow = preset.to_github().unwrap();

        assert!(workflow.jobs.contains_key("rust/test"));
        assert!(workflow.jobs.contains_key("rust/lint"));
    }

    #[test]
    fn test_preset_info() {
        let preset = RustPreset::default();
        assert_eq!(preset.name(), "rust");
        assert!(!preset.description().is_empty());
    }
}
