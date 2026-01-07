use crate::traits::PresetInfo;
use cci_macros::Preset;

mod circleci;
mod detectable;
mod gitea;
mod github;
mod gitlab;
mod jenkins;

/// Unified preset for Rust projects (binaries, libraries, and workspaces)
#[derive(Debug, Clone, Preset)]
#[preset(
    id = "rust",
    name = "Rust",
    description = "CI pipeline for Rust projects (binaries, libraries, and workspaces)",
    matches = "RustBinary | RustLibrary | RustWorkspace"
)]
pub struct RustPreset {
    #[preset_field(default = "\"stable\".to_string()", hidden = true)]
    pub(super) rust_version: String,

    #[preset_field(
        feature = "testing",
        feature_display = "Testing",
        display = "Code Coverage",
        description = "Enable code coverage reporting with tarpaulin",
        default = "false"
    )]
    pub(super) enable_coverage: bool,

    #[preset_field(
        feature = "linting",
        feature_display = "Linting",
        display = "Clippy Linter",
        description = "Run Clippy linter for code quality",
        default = "false"
    )]
    pub(super) enable_linter: bool,

    #[preset_field(
        feature = "security",
        feature_display = "Security",
        display = "Security Scan",
        description = "Run cargo-audit for dependency vulnerabilities",
        default = "false"
    )]
    pub(super) enable_security_scan: bool,

    #[preset_field(
        feature = "formatting",
        feature_display = "Formatting",
        display = "Rustfmt Check",
        description = "Check code formatting with rustfmt",
        default = "false"
    )]
    pub(super) enable_format_check: bool,

    #[preset_field(
        feature = "building",
        feature_display = "Building",
        display = "Build Release",
        description = "Build optimized release binary in CI",
        default = "false"
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

impl PresetInfo for RustPreset {
    fn name(&self) -> &str {
        "rust"
    }

    fn description(&self) -> &str {
        "CI pipeline for Rust projects (binaries, libraries, and workspaces)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{PresetInfo, ToGitHub};

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
