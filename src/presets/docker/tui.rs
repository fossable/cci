use crate::detection::ProjectType;
use crate::error::Result;
use crate::presets::docker::{DockerPreset, DockerRegistry};
use crate::traits::{ToCircleCI, ToGitHub, ToGitLab, ToJenkins};
use crate::tui::config::{FeatureMeta, OptionMeta, OptionValue, PresetConfig, TuiPreset};
use crate::tui::state::Platform;
use std::path::PathBuf;

pub struct DockerTuiPreset;

impl TuiPreset for DockerTuiPreset {
    fn preset_id(&self) -> &'static str {
        "docker"
    }

    fn preset_name(&self) -> &'static str {
        "Docker"
    }

    fn preset_description(&self) -> &'static str {
        "CI pipeline for building and pushing Docker images to registries"
    }

    fn features(&self) -> Vec<FeatureMeta> {
        vec![
            FeatureMeta {
                id: "configuration".to_string(),
                display_name: "Configuration".to_string(),
                description: "Basic Docker image configuration".to_string(),
                options: vec![OptionMeta {
                    id: "image_name".to_string(),
                    display_name: "Image Name".to_string(),
                    description: "Docker image name (e.g., myapp)".to_string(),
                    default_value: OptionValue::String("myapp".to_string()),
                    depends_on: None,
                }],
            },
            FeatureMeta {
                id: "registry".to_string(),
                display_name: "Registry".to_string(),
                description: "Container registry configuration".to_string(),
                options: vec![OptionMeta {
                    id: "registry_type".to_string(),
                    display_name: "Registry Type".to_string(),
                    description: "Choose where to push Docker images".to_string(),
                    default_value: OptionValue::Enum {
                        selected: "none".to_string(),
                        variants: vec![
                            "none".to_string(),
                            "dockerhub".to_string(),
                            "github".to_string(),
                        ],
                    },
                    depends_on: None,
                }],
            },
            FeatureMeta {
                id: "optimization".to_string(),
                display_name: "Optimization".to_string(),
                description: "Build optimization settings".to_string(),
                options: vec![
                    OptionMeta {
                        id: "enable_cache".to_string(),
                        display_name: "Enable Cache".to_string(),
                        description: "Use Docker layer caching for faster builds".to_string(),
                        default_value: OptionValue::Bool(true),
                        depends_on: None,
                    },
                    OptionMeta {
                        id: "tags_only".to_string(),
                        display_name: "Tags Only".to_string(),
                        description: "Only push images on git tags (not on branch pushes)".to_string(),
                        default_value: OptionValue::Bool(false),
                        depends_on: None,
                    },
                ],
            },
            FeatureMeta {
                id: "multiarch".to_string(),
                display_name: "Multi-Architecture".to_string(),
                description: "Cross-platform build settings".to_string(),
                options: vec![
                    OptionMeta {
                        id: "enable_qemu".to_string(),
                        display_name: "Enable QEMU".to_string(),
                        description: "Enable cross-architecture builds using QEMU emulation".to_string(),
                        default_value: OptionValue::Bool(false),
                        depends_on: None,
                    },
                    OptionMeta {
                        id: "multiplatform".to_string(),
                        display_name: "Multi-Platform".to_string(),
                        description: "Build for multiple platforms (linux/amd64, linux/arm64)".to_string(),
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
        _language_version: &str,
    ) -> Result<String> {
        let image_name = config
            .get_string("image_name")
            .unwrap_or_else(|| "myapp".to_string());

        let registry = match config.get_enum("registry_type").as_deref() {
            Some("dockerhub") => DockerRegistry::DockerHub,
            Some("github") => DockerRegistry::GitHubRegistry,
            _ => DockerRegistry::None,
        };

        let preset = DockerPreset::builder()
            .image_name(&image_name)
            .registry(registry)
            .cache(config.get_bool("enable_cache"))
            .push_on_tags_only(config.get_bool("tags_only"))
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

    fn matches_project(&self, _project_type: &ProjectType, working_dir: &std::path::Path) -> bool {
        // Docker preset is available for all project types IF a Dockerfile exists
        // Check for common Dockerfile names
        let dockerfile_names = ["Dockerfile", "Dockerfile.dev", "Dockerfile.prod", "dockerfile"];

        for name in &dockerfile_names {
            if working_dir.join(name).exists() {
                return true;
            }
        }

        false
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
