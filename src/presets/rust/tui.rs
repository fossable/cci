use crate::detection::ProjectType;
use crate::error::Result;
use crate::presets::rust::{RustBinaryPreset, RustLibraryPreset};
use crate::traits::{ToCircleCI, ToGitHub, ToGitLab, ToJenkins};
use crate::tui::config::{FeatureMeta, OptionMeta, OptionValue, PresetConfig, TuiPreset};
use crate::tui::state::Platform;

pub struct RustLibraryTuiPreset;

impl TuiPreset for RustLibraryTuiPreset {
    fn preset_id(&self) -> &'static str {
        "rust-library"
    }

    fn preset_name(&self) -> &'static str {
        "Rust Library"
    }

    fn preset_description(&self) -> &'static str {
        "CI pipeline for Rust library projects with testing, linting, and optional coverage"
    }

    fn features(&self) -> Vec<FeatureMeta> {
        vec![
            FeatureMeta {
                id: "testing".to_string(),
                display_name: "Testing".to_string(),
                description: "Test coverage reporting".to_string(),
                options: vec![OptionMeta {
                    id: "enable_coverage".to_string(),
                    display_name: "Code Coverage".to_string(),
                    description: "Enable code coverage reporting with tarpaulin".to_string(),
                    default_value: OptionValue::Bool(true),
                    depends_on: None,
                }],
            },
            FeatureMeta {
                id: "linting".to_string(),
                display_name: "Linting".to_string(),
                description: "Code quality checks".to_string(),
                options: vec![OptionMeta {
                    id: "enable_linter".to_string(),
                    display_name: "Clippy Linter".to_string(),
                    description: "Run Clippy linter for code quality".to_string(),
                    default_value: OptionValue::Bool(true),
                    depends_on: None,
                }],
            },
            FeatureMeta {
                id: "formatting".to_string(),
                display_name: "Formatting".to_string(),
                description: "Code formatting checks".to_string(),
                options: vec![OptionMeta {
                    id: "enable_formatter".to_string(),
                    display_name: "Rustfmt Check".to_string(),
                    description: "Check code formatting with rustfmt".to_string(),
                    default_value: OptionValue::Bool(true),
                    depends_on: None,
                }],
            },
            FeatureMeta {
                id: "security".to_string(),
                display_name: "Security".to_string(),
                description: "Security vulnerability scanning".to_string(),
                options: vec![OptionMeta {
                    id: "enable_security".to_string(),
                    display_name: "Security Scan".to_string(),
                    description: "Run cargo-audit for dependency vulnerabilities".to_string(),
                    default_value: OptionValue::Bool(true),
                    depends_on: None,
                }],
            },
        ]
    }

    fn generate(
        &self,
        config: &PresetConfig,
        platform: Platform,
        language_version: &str,
    ) -> Result<String> {
        let preset = RustLibraryPreset::builder()
            .rust_version(language_version)
            .coverage(config.get_bool("enable_coverage"))
            .linter(config.get_bool("enable_linter"))
            .format_check(config.get_bool("enable_formatter"))
            .security_scan(config.get_bool("enable_security"))
            .build();

        let output: String = match platform {
            Platform::GitHub => {
                let workflow = preset.to_github()?;
                serde_yaml::to_string(&workflow)?
            }
            Platform::GitLab => {
                let config = preset.to_gitlab()?;
                serde_yaml::to_string(&config)?
            }
            Platform::CircleCI => {
                let config = preset.to_circleci()?;
                serde_yaml::to_string(&config)?
            }
            Platform::Jenkins => {
                let jenkins_config = preset.to_jenkins()?;
                jenkins_to_string(&jenkins_config)
            }
        };

        Ok(output)
    }

    fn matches_project(&self, project_type: &ProjectType, _working_dir: &std::path::Path) -> bool {
        matches!(
            project_type,
            ProjectType::RustLibrary | ProjectType::RustWorkspace
        )
    }

    fn default_config(&self, detected: bool) -> PresetConfig {
        let mut config = PresetConfig::new(self.preset_id().to_string());

        for feature in self.features() {
            for option in feature.options {
                let value = if detected {
                    option.default_value.clone()
                } else {
                    match option.default_value {
                        OptionValue::Bool(_) => OptionValue::Bool(false),
                        other => other,
                    }
                };
                config.set(option.id, value);
            }
        }

        config
    }
}

pub struct RustBinaryTuiPreset;

impl TuiPreset for RustBinaryTuiPreset {
    fn preset_id(&self) -> &'static str {
        "rust-binary"
    }

    fn preset_name(&self) -> &'static str {
        "Rust Binary"
    }

    fn preset_description(&self) -> &'static str {
        "CI pipeline for Rust binary/application projects with building and testing"
    }

    fn features(&self) -> Vec<FeatureMeta> {
        vec![
            FeatureMeta {
                id: "linting".to_string(),
                display_name: "Linting".to_string(),
                description: "Code quality checks".to_string(),
                options: vec![OptionMeta {
                    id: "enable_linter".to_string(),
                    display_name: "Clippy Linter".to_string(),
                    description: "Run Clippy linter for code quality".to_string(),
                    default_value: OptionValue::Bool(true),
                    depends_on: None,
                }],
            },
            FeatureMeta {
                id: "building".to_string(),
                display_name: "Building".to_string(),
                description: "Release binary builds".to_string(),
                options: vec![OptionMeta {
                    id: "build_release".to_string(),
                    display_name: "Build Release".to_string(),
                    description: "Build optimized release binary in CI".to_string(),
                    default_value: OptionValue::Bool(true),
                    depends_on: None,
                }],
            },
        ]
    }

    fn generate(
        &self,
        config: &PresetConfig,
        platform: Platform,
        language_version: &str,
    ) -> Result<String> {
        let preset = RustBinaryPreset::builder()
            .rust_version(language_version)
            .linter(config.get_bool("enable_linter"))
            .build_release(config.get_bool("build_release"))
            .build();

        let output: String = match platform {
            Platform::GitHub => {
                let workflow = preset.to_github()?;
                serde_yaml::to_string(&workflow)?
            }
            Platform::GitLab => {
                let config = preset.to_gitlab()?;
                serde_yaml::to_string(&config)?
            }
            Platform::CircleCI => {
                let config = preset.to_circleci()?;
                serde_yaml::to_string(&config)?
            }
            Platform::Jenkins => {
                let jenkins_config = preset.to_jenkins()?;
                jenkins_to_string(&jenkins_config)
            }
        };

        Ok(output)
    }

    fn matches_project(&self, project_type: &ProjectType, _working_dir: &std::path::Path) -> bool {
        matches!(project_type, ProjectType::RustBinary)
    }

    fn default_config(&self, detected: bool) -> PresetConfig {
        let mut config = PresetConfig::new(self.preset_id().to_string());

        for feature in self.features() {
            for option in feature.options {
                let value = if detected {
                    option.default_value.clone()
                } else {
                    match option.default_value {
                        OptionValue::Bool(_) => OptionValue::Bool(false),
                        other => other,
                    }
                };
                config.set(option.id, value);
            }
        }

        config
    }
}

// Helper function to convert Jenkins config to string
fn jenkins_to_string(config: &crate::platforms::jenkins::models::JenkinsConfig) -> String {
    let mut result = String::new();
    result.push_str("pipeline {\n");
    result.push_str(&format!("    agent {{\n"));
    result.push_str(&format!("        label '{}'\n", config.agent));
    result.push_str(&format!("    }}\n\n"));

    if !config.environment.is_empty() {
        result.push_str("    environment {\n");
        for (key, value) in &config.environment {
            result.push_str(&format!("        {} = '{}'\n", key, value));
        }
        result.push_str("    }\n\n");
    }

    result.push_str("    stages {\n");
    for stage in &config.stages {
        result.push_str(&format!("        stage('{}') {{\n", stage.name));
        result.push_str("            steps {\n");
        for step in &stage.steps {
            result.push_str(&format!("                {}\n", step));
        }
        result.push_str("            }\n");
        result.push_str("        }\n");
    }
    result.push_str("    }\n");
    result.push_str("}\n");
    result
}
