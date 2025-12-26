use crate::detection::ProjectType;
use crate::error::Result;
use crate::presets::go::GoAppPreset;
use crate::traits::{ToCircleCI, ToGitHub, ToGitLab, ToJenkins};
use crate::tui::config::{FeatureMeta, OptionMeta, OptionValue, PresetConfig, TuiPreset};
use crate::tui::state::Platform;

pub struct GoAppTuiPreset;

impl TuiPreset for GoAppTuiPreset {
    fn preset_id(&self) -> &'static str {
        "go-app"
    }

    fn preset_name(&self) -> &'static str {
        "Go App"
    }

    fn preset_description(&self) -> &'static str {
        "CI pipeline for Go applications with testing and linting"
    }

    fn features(&self) -> Vec<FeatureMeta> {
        vec![
            FeatureMeta {
                id: "linting".to_string(),
                display_name: "Linting".to_string(),
                description: "Code quality checks with golangci-lint".to_string(),
                options: vec![OptionMeta {
                    id: "enable_linter".to_string(),
                    display_name: "Enable Linter".to_string(),
                    description: "Run golangci-lint for code quality".to_string(),
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
                    description: "Run gosec for security vulnerabilities".to_string(),
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
        let preset = GoAppPreset::new(
            language_version.to_string(),
            config.get_bool("enable_linter"),
            config.get_bool("enable_security"),
        );

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
        matches!(project_type, ProjectType::GoApp | ProjectType::GoLibrary)
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
