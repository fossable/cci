use crate::detection::ProjectType;
use crate::error::Result;
use crate::presets::python::{PythonAppPreset, PythonFormatterTool, PythonLinterTool};
use crate::traits::{ToCircleCI, ToGitHub, ToGitLab, ToJenkins};
use crate::tui::config::{FeatureMeta, OptionMeta, OptionValue, PresetConfig, TuiPreset};
use crate::tui::state::Platform;

pub struct PythonAppTuiPreset;

impl TuiPreset for PythonAppTuiPreset {
    fn preset_id(&self) -> &'static str {
        "python-app"
    }

    fn preset_name(&self) -> &'static str {
        "Python App"
    }

    fn preset_description(&self) -> &'static str {
        "CI pipeline for Python applications with pytest, linting, and type checking"
    }

    fn features(&self) -> Vec<FeatureMeta> {
        vec![
            FeatureMeta {
                id: "linting".to_string(),
                display_name: "Linting".to_string(),
                description: "Code quality checks with configurable tools".to_string(),
                options: vec![
                    OptionMeta {
                        id: "enable_linter".to_string(),
                        display_name: "Enable Linter".to_string(),
                        description: "Run linting checks on code".to_string(),
                        default_value: OptionValue::Bool(true),
                        depends_on: None,
                    },
                    OptionMeta {
                        id: "linter_tool".to_string(),
                        display_name: "Linter Tool".to_string(),
                        description: "Choose between Flake8 or Ruff for linting".to_string(),
                        default_value: OptionValue::Enum {
                            selected: "flake8".to_string(),
                            variants: vec!["flake8".to_string(), "ruff".to_string()],
                        },
                        depends_on: Some("enable_linter".to_string()),
                    },
                ],
            },
            FeatureMeta {
                id: "formatting".to_string(),
                display_name: "Formatting".to_string(),
                description: "Code formatting checks".to_string(),
                options: vec![
                    OptionMeta {
                        id: "enable_formatter".to_string(),
                        display_name: "Enable Formatter".to_string(),
                        description: "Check code formatting compliance".to_string(),
                        default_value: OptionValue::Bool(true),
                        depends_on: None,
                    },
                    OptionMeta {
                        id: "formatter_tool".to_string(),
                        display_name: "Formatter Tool".to_string(),
                        description: "Choose between Black or Ruff for formatting".to_string(),
                        default_value: OptionValue::Enum {
                            selected: "black".to_string(),
                            variants: vec!["black".to_string(), "ruff".to_string()],
                        },
                        depends_on: Some("enable_formatter".to_string()),
                    },
                ],
            },
            FeatureMeta {
                id: "testing".to_string(),
                display_name: "Testing".to_string(),
                description: "Test execution and type checking".to_string(),
                options: vec![
                    OptionMeta {
                        id: "type_check".to_string(),
                        display_name: "Type Checking".to_string(),
                        description: "Enable mypy static type checking".to_string(),
                        default_value: OptionValue::Bool(true),
                        depends_on: None,
                    },
                    OptionMeta {
                        id: "build_wheel".to_string(),
                        display_name: "Build Wheel".to_string(),
                        description: "Build distributable wheel package".to_string(),
                        default_value: OptionValue::Bool(false),
                        depends_on: None,
                    },
                ],
            },
        ]
    }

    fn generate(
        &self,
        config: &PresetConfig,
        platform: Platform,
        language_version: &str,
    ) -> Result<String> {
        // Extract configuration values
        let linter = config.get_bool("enable_linter");
        let linter_tool = match config.get_enum("linter_tool").as_deref() {
            Some("ruff") => PythonLinterTool::Ruff,
            _ => PythonLinterTool::Flake8,
        };

        let formatter = config.get_bool("enable_formatter");
        let formatter_tool = match config.get_enum("formatter_tool").as_deref() {
            Some("ruff") => PythonFormatterTool::Ruff,
            _ => PythonFormatterTool::Black,
        };

        let type_check = config.get_bool("type_check");

        // Build preset using new constructor
        let preset = PythonAppPreset::new(
            language_version.to_string(),
            linter,
            linter_tool,
            type_check,
            formatter,
            formatter_tool,
        );

        // Generate based on platform
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
            ProjectType::PythonApp | ProjectType::PythonLibrary
        )
    }

    fn default_config(&self, detected: bool) -> PresetConfig {
        let mut config = PresetConfig::new(self.preset_id().to_string());

        // Set defaults based on detection
        for feature in self.features() {
            for option in feature.options {
                let value = if detected {
                    option.default_value.clone()
                } else {
                    // Disable non-detected presets
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
