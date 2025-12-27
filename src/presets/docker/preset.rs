use crate::detection::ProjectType;
use crate::editor::config::{EditorPreset, FeatureMeta, OptionMeta, OptionValue, PresetConfig};
use crate::editor::state::Platform;
use crate::error::Result;
use crate::platforms::circleci::models::CircleCIConfig;
use crate::platforms::github::models::{
    GitHubJob, GitHubStep, GitHubTriggerConfig, GitHubTriggers, GitHubWorkflow,
};
use crate::platforms::gitlab::models::GitLabCI;
use crate::platforms::helpers::generate_for_platform;
use crate::platforms::jenkins::models::JenkinsConfig;
use crate::traits::{Detectable, PresetInfo, ToCircleCI, ToGitea, ToGitHub, ToGitLab, ToJenkins};
use std::collections::BTreeMap;
use std::path::Path;

/// Container registry options for Docker image pushing
#[derive(Debug, Clone, PartialEq)]
pub enum DockerRegistry {
    /// Push to Docker Hub (requires DOCKER_USERNAME and DOCKER_PASSWORD secrets)
    DockerHub,
    /// Push to GitHub Container Registry (uses GITHUB_TOKEN)
    GitHubRegistry,
    /// Don't push images (build only)
    None,
}

/// Preset for Docker-based projects with optional registry pushing
#[derive(Debug, Clone)]
pub struct DockerPreset {
    /// Docker image name (e.g., "myorg/myapp")
    image_name: String,
    /// Registry to push to
    registry: DockerRegistry,
    /// Dockerfile path (default: "./Dockerfile")
    dockerfile_path: String,
    /// Docker build context (default: ".")
    build_context: String,
    /// Additional build arguments
    build_args: Vec<(String, String)>,
    /// Enable Docker layer caching
    enable_cache: bool,
    /// Push on tags only (if false, pushes on main/master branch)
    push_on_tags_only: bool,
}

impl DockerPreset {
    pub fn new(
        image_name: String,
        registry: DockerRegistry,
        dockerfile_path: String,
        build_context: String,
        build_args: Vec<(String, String)>,
        enable_cache: bool,
        push_on_tags_only: bool,
    ) -> Self {
        Self {
            image_name,
            registry,
            dockerfile_path,
            build_context,
            build_args,
            enable_cache,
            push_on_tags_only,
        }
    }

    /// Create a new DockerPreset from editor configuration
    pub fn from_config(config: &PresetConfig, _version: &str) -> Self {
        let image_name = config
            .get_string("image_name")
            .unwrap_or_else(|| "myapp".to_string());

        let registry = match config.get_enum("registry_type").as_deref() {
            Some("dockerhub") => DockerRegistry::DockerHub,
            Some("github") => DockerRegistry::GitHubRegistry,
            _ => DockerRegistry::None,
        };

        Self::new(
            image_name,
            registry,
            "./Dockerfile".to_string(),
            ".".to_string(),
            vec![],
            config.get_bool("enable_cache"),
            config.get_bool("tags_only"),
        )
    }

    /// Constant default instance for registry initialization
    pub const DEFAULT: Self = Self {
        image_name: String::new(),
        registry: DockerRegistry::None,
        dockerfile_path: String::new(),
        build_context: String::new(),
        build_args: vec![],
        enable_cache: false,
        push_on_tags_only: false,
    };
}

impl Default for DockerPreset {
    fn default() -> Self {
        Self {
            image_name: "myapp".to_string(),
            registry: DockerRegistry::None,
            dockerfile_path: "./Dockerfile".to_string(),
            build_context: ".".to_string(),
            build_args: vec![],
            enable_cache: false,
            push_on_tags_only: false,
        }
    }
}

impl ToGitHub for DockerPreset {
    fn to_github(&self) -> Result<GitHubWorkflow> {
        let mut jobs = BTreeMap::new();

        // Build and optionally push Docker image
        let mut build_steps = vec![GitHubStep {
            name: Some("Checkout code".to_string()),
            uses: Some("actions/checkout@v4".to_string()),
            run: None,
            with: None,
            env: None,
        }];

        // Add Docker buildx setup for better caching and multi-platform support
        build_steps.push(GitHubStep {
            name: Some("Set up Docker Buildx".to_string()),
            uses: Some("docker/setup-buildx-action@v3".to_string()),
            run: None,
            with: None,
            env: None,
        });

        // Login step depends on registry type
        match &self.registry {
            DockerRegistry::DockerHub => {
                build_steps.push(GitHubStep {
                    name: Some("Login to Docker Hub".to_string()),
                    uses: Some("docker/login-action@v3".to_string()),
                    run: None,
                    with: Some(BTreeMap::from([
                        (
                            "username".to_string(),
                            serde_yaml::Value::String("${{ secrets.DOCKER_USERNAME }}".to_string()),
                        ),
                        (
                            "password".to_string(),
                            serde_yaml::Value::String("${{ secrets.DOCKER_PASSWORD }}".to_string()),
                        ),
                    ])),
                    env: None,
                });
            }
            DockerRegistry::GitHubRegistry => {
                build_steps.push(GitHubStep {
                    name: Some("Login to GitHub Container Registry".to_string()),
                    uses: Some("docker/login-action@v3".to_string()),
                    run: None,
                    with: Some(BTreeMap::from([
                        (
                            "registry".to_string(),
                            serde_yaml::Value::String("ghcr.io".to_string()),
                        ),
                        (
                            "username".to_string(),
                            serde_yaml::Value::String("${{ github.actor }}".to_string()),
                        ),
                        (
                            "password".to_string(),
                            serde_yaml::Value::String("${{ secrets.GITHUB_TOKEN }}".to_string()),
                        ),
                    ])),
                    env: None,
                });
            }
            DockerRegistry::None => {}
        }

        // Extract metadata for tags and labels
        build_steps.push(GitHubStep {
            name: Some("Extract Docker metadata".to_string()),
            uses: Some("docker/metadata-action@v5".to_string()),
            run: None,
            with: Some(BTreeMap::from([
                (
                    "images".to_string(),
                    serde_yaml::Value::String(match &self.registry {
                        DockerRegistry::GitHubRegistry => {
                            format!("ghcr.io/${{ github.repository_owner }}/{}", self.image_name)
                        }
                        _ => self.image_name.clone(),
                    }),
                ),
                (
                    "tags".to_string(),
                    serde_yaml::Value::String(
                        "type=ref,event=branch\ntype=ref,event=pr\ntype=semver,pattern={{version}}\ntype=semver,pattern={{major}}.{{minor}}".to_string(),
                    ),
                ),
            ])),
            env: None,
        });

        // Build and push step
        let mut build_push_with = BTreeMap::from([
            (
                "context".to_string(),
                serde_yaml::Value::String(self.build_context.clone()),
            ),
            (
                "file".to_string(),
                serde_yaml::Value::String(self.dockerfile_path.clone()),
            ),
            (
                "tags".to_string(),
                serde_yaml::Value::String("${{ steps.meta.outputs.tags }}".to_string()),
            ),
            (
                "labels".to_string(),
                serde_yaml::Value::String("${{ steps.meta.outputs.labels }}".to_string()),
            ),
        ]);

        // Only push if registry is configured
        if self.registry != DockerRegistry::None {
            build_push_with.insert(
                "push".to_string(),
                serde_yaml::Value::String("true".to_string()),
            );
        }

        // Add caching if enabled
        if self.enable_cache {
            build_push_with.insert(
                "cache-from".to_string(),
                serde_yaml::Value::String("type=gha".to_string()),
            );
            build_push_with.insert(
                "cache-to".to_string(),
                serde_yaml::Value::String("type=gha,mode=max".to_string()),
            );
        }

        // Add build args if present
        if !self.build_args.is_empty() {
            let build_args_str = self
                .build_args
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("\n");
            build_push_with.insert(
                "build-args".to_string(),
                serde_yaml::Value::String(build_args_str),
            );
        }

        build_steps.push(GitHubStep {
            name: Some("Build and push Docker image".to_string()),
            uses: Some("docker/build-push-action@v5".to_string()),
            run: None,
            with: Some(build_push_with),
            env: None,
        });

        jobs.insert(
            "docker/build".to_string(),
            GitHubJob {
                runs_on: "ubuntu-latest".to_string(),
                steps: build_steps,
                needs: None,
                timeout_minutes: Some(30),
                continue_on_error: None,
            },
        );

        // Configure triggers based on push settings
        let triggers = if self.push_on_tags_only {
            GitHubTriggers::Detailed(BTreeMap::from([
                (
                    "push".to_string(),
                    GitHubTriggerConfig {
                        branches: None,
                        tags: Some(vec!["v*".to_string()]),
                    },
                ),
                (
                    "pull_request".to_string(),
                    GitHubTriggerConfig {
                        branches: Some(vec!["main".to_string(), "master".to_string()]),
                        tags: None,
                    },
                ),
            ]))
        } else {
            GitHubTriggers::Detailed(BTreeMap::from([
                (
                    "push".to_string(),
                    GitHubTriggerConfig {
                        branches: Some(vec!["main".to_string(), "master".to_string()]),
                        tags: Some(vec!["v*".to_string()]),
                    },
                ),
                (
                    "pull_request".to_string(),
                    GitHubTriggerConfig {
                        branches: Some(vec!["main".to_string(), "master".to_string()]),
                        tags: None,
                    },
                ),
            ]))
        };

        Ok(GitHubWorkflow {
            name: "Docker Build and Push".to_string(),
            on: triggers,
            env: None,
            jobs,
        })
    }
}

impl ToGitea for DockerPreset {
    fn to_gitea(&self) -> Result<crate::platforms::gitea::models::GiteaWorkflow> {
        // Gitea Actions uses the same workflow format as GitHub Actions
        self.to_github()
    }
}

impl ToGitLab for DockerPreset {
    fn to_gitlab(&self) -> Result<GitLabCI> {
        use crate::platforms::gitlab::models::*;
        use std::collections::BTreeMap;

        let mut jobs = BTreeMap::new();

        let mut script = vec![
            "docker login -u $CI_REGISTRY_USER -p $CI_REGISTRY_PASSWORD $CI_REGISTRY".to_string(),
        ];

        // Prepare build command
        let mut build_cmd = format!(
            "docker build -t {} -f {}",
            self.image_name, self.dockerfile_path
        );

        // Add build args
        for (key, value) in &self.build_args {
            build_cmd.push_str(&format!(" --build-arg {}={}", key, value));
        }

        build_cmd.push_str(&format!(" {}", self.build_context));
        script.push(build_cmd);

        // Push if registry is configured
        if self.registry != DockerRegistry::None {
            script.push(format!("docker push {}", self.image_name));
        }

        jobs.insert(
            "docker/build".to_string(),
            GitLabJob {
                stage: "build".to_string(),
                image: Some("docker:latest".to_string()),
                script,
                before_script: None,
                after_script: None,
                needs: None,
                cache: None,
                artifacts: None,
                only: if self.push_on_tags_only {
                    Some(GitLabOnly {
                        refs: Some(vec!["tags".to_string()]),
                    })
                } else {
                    None
                },
                timeout: None,
            },
        );

        Ok(GitLabCI {
            stages: Some(vec!["build".to_string()]),
            variables: None,
            cache: None,
            jobs,
        })
    }
}

impl ToCircleCI for DockerPreset {
    fn to_circleci(&self) -> Result<CircleCIConfig> {
        use crate::platforms::circleci::models::*;
        use std::collections::BTreeMap;

        let mut steps = vec![CircleCIStep::Simple("checkout".to_string())];

        // Setup remote Docker
        steps.push(CircleCIStep::Simple("setup_remote_docker".to_string()));

        // Login based on registry
        match &self.registry {
            DockerRegistry::DockerHub => {
                steps.push(CircleCIStep::Command {
                    run: CircleCIRun::Simple(
                        "echo $DOCKER_PASSWORD | docker login -u $DOCKER_USERNAME --password-stdin"
                            .to_string(),
                    ),
                });
            }
            DockerRegistry::GitHubRegistry => {
                steps.push(CircleCIStep::Command {
                    run: CircleCIRun::Simple(
                        "echo $GITHUB_TOKEN | docker login ghcr.io -u $GITHUB_USERNAME --password-stdin"
                            .to_string(),
                    ),
                });
            }
            DockerRegistry::None => {}
        }

        // Build command
        let mut build_cmd = format!(
            "docker build -t {} -f {}",
            self.image_name, self.dockerfile_path
        );

        for (key, value) in &self.build_args {
            build_cmd.push_str(&format!(" --build-arg {}={}", key, value));
        }

        build_cmd.push_str(&format!(" {}", self.build_context));

        steps.push(CircleCIStep::Command {
            run: CircleCIRun::Detailed {
                name: "Build Docker image".to_string(),
                command: build_cmd,
            },
        });

        // Push if registry configured
        if self.registry != DockerRegistry::None {
            steps.push(CircleCIStep::Command {
                run: CircleCIRun::Detailed {
                    name: "Push Docker image".to_string(),
                    command: format!("docker push {}", self.image_name),
                },
            });
        }

        let mut jobs = BTreeMap::new();
        jobs.insert(
            "docker/build".to_string(),
            CircleCIJob {
                docker: vec![CircleCIDocker {
                    image: "cimg/base:stable".to_string(),
                }],
                steps,
                environment: None,
            },
        );

        Ok(CircleCIConfig {
            version: "2.1".to_string(),
            orbs: None,
            jobs,
            workflows: BTreeMap::from([(
                "main".to_string(),
                CircleCIWorkflow {
                    jobs: vec![CircleCIWorkflowJob::Simple("docker/build".to_string())],
                },
            )]),
        })
    }
}

impl ToJenkins for DockerPreset {
    fn to_jenkins(&self) -> Result<JenkinsConfig> {
        use crate::platforms::jenkins::models::*;

        let mut steps = Vec::new();

        // Login command
        let login_cmd = match &self.registry {
            DockerRegistry::DockerHub => {
                "sh 'echo $DOCKER_PASSWORD | docker login -u $DOCKER_USERNAME --password-stdin'"
                    .to_string()
            }
            DockerRegistry::GitHubRegistry => {
                "sh 'echo $GITHUB_TOKEN | docker login ghcr.io -u $GITHUB_USERNAME --password-stdin'"
                    .to_string()
            }
            DockerRegistry::None => String::new(),
        };

        if !login_cmd.is_empty() {
            steps.push(login_cmd);
        }

        // Build command
        let mut build_cmd = format!(
            "sh 'docker build -t {} -f {}",
            self.image_name, self.dockerfile_path
        );

        for (key, value) in &self.build_args {
            build_cmd.push_str(&format!(" --build-arg {}={}", key, value));
        }

        build_cmd.push_str(&format!(" {}'", self.build_context));
        steps.push(build_cmd);

        // Push command
        if self.registry != DockerRegistry::None {
            steps.push(format!("sh 'docker push {}'", self.image_name));
        }

        Ok(JenkinsConfig {
            agent: "any".to_string(),
            environment: vec![],
            stages: vec![JenkinsStage {
                name: "Docker Build".to_string(),
                steps,
            }],
        })
    }
}

impl Detectable for DockerPreset {
    fn matches_github(&self, workflow: &GitHubWorkflow) -> bool {
        // Check for Docker build actions
        let has_docker_build = workflow.jobs.values().any(|job| {
            job.steps.iter().any(|step| {
                step.uses
                    .as_ref()
                    .map(|u| u.contains("docker/build-push-action"))
                    .unwrap_or(false)
            })
        });

        // Check for docker build commands
        let has_docker_commands = workflow.jobs.values().any(|job| {
            job.steps.iter().any(|step| {
                step.run
                    .as_ref()
                    .map(|r| r.contains("docker build"))
                    .unwrap_or(false)
            })
        });

        has_docker_build || has_docker_commands
    }

    fn matches_gitea(&self, workflow: &crate::platforms::gitea::models::GiteaWorkflow) -> bool {
        // Gitea Actions uses the same workflow format as GitHub Actions
        self.matches_github(workflow)
    }

    fn matches_gitlab(&self, config: &GitLabCI) -> bool {
        config.jobs.values().any(|job| {
            job.script
                .iter()
                .any(|cmd| cmd.contains("docker build") || cmd.contains("docker push"))
        })
    }

    fn matches_circleci(&self, config: &CircleCIConfig) -> bool {
        config.jobs.values().any(|job| {
            job.steps.iter().any(|step| {
                if let crate::platforms::circleci::models::CircleCIStep::Command { run } = step {
                    match run {
                        crate::platforms::circleci::models::CircleCIRun::Simple(cmd) => {
                            cmd.contains("docker build")
                        }
                        crate::platforms::circleci::models::CircleCIRun::Detailed {
                            command, ..
                        } => command.contains("docker build"),
                    }
                } else {
                    false
                }
            })
        })
    }

    fn matches_jenkins(&self, config: &JenkinsConfig) -> bool {
        config.stages.iter().any(|stage| {
            stage
                .steps
                .iter()
                .any(|step| step.contains("docker build"))
        })
    }
}

impl PresetInfo for DockerPreset {
    fn name(&self) -> &str {
        "docker"
    }

    fn description(&self) -> &str {
        "CI pipeline for building and pushing Docker images to registries"
    }
}

impl EditorPreset for DockerPreset {
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
        language_version: &str,
    ) -> Result<String> {
        let preset = Self::from_config(config, language_version);
        generate_for_platform(&preset, platform)
    }

    fn matches_project(&self, project_type: &ProjectType, working_dir: &Path) -> bool {
        // Docker preset matches if:
        // 1. Project type is DockerImage, OR
        // 2. Any project type with a Dockerfile present

        if matches!(project_type, ProjectType::DockerImage) {
            return true;
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let preset = DockerPreset::new(
            "myapp".to_string(),
            DockerRegistry::None,
            "./Dockerfile".to_string(),
            ".".to_string(),
            vec![],
            false,
            false,
        );

        assert_eq!(preset.image_name, "myapp");
        assert_eq!(preset.registry, DockerRegistry::None);
        assert_eq!(preset.dockerfile_path, "./Dockerfile");
        assert_eq!(preset.build_context, ".");
        assert!(!preset.enable_cache);
        assert!(!preset.push_on_tags_only);
    }

    #[test]
    fn test_builder_with_dockerhub() {
        let preset = DockerPreset::new(
            "myorg/myapp".to_string(),
            DockerRegistry::DockerHub,
            "./Dockerfile".to_string(),
            ".".to_string(),
            vec![],
            true,
            false,
        );

        assert_eq!(preset.registry, DockerRegistry::DockerHub);
        assert!(preset.enable_cache);
    }

    #[test]
    fn test_builder_with_github_registry() {
        let preset = DockerPreset::new(
            "myapp".to_string(),
            DockerRegistry::GitHubRegistry,
            "./Dockerfile".to_string(),
            ".".to_string(),
            vec![],
            false,
            true,
        );

        assert_eq!(preset.registry, DockerRegistry::GitHubRegistry);
        assert!(preset.push_on_tags_only);
    }

    #[test]
    fn test_builder_with_custom_paths() {
        let preset = DockerPreset::new(
            "myapp".to_string(),
            DockerRegistry::None,
            "./docker/Dockerfile".to_string(),
            "./app".to_string(),
            vec![
                ("VERSION".to_string(), "1.0.0".to_string()),
                ("BUILD_DATE".to_string(), "2024-01-01".to_string()),
            ],
            false,
            false,
        );

        assert_eq!(preset.dockerfile_path, "./docker/Dockerfile");
        assert_eq!(preset.build_context, "./app");
        assert_eq!(preset.build_args.len(), 2);
    }

    #[test]
    fn test_to_github_basic() {
        let preset = DockerPreset::new(
            "myapp".to_string(),
            DockerRegistry::None,
            "./Dockerfile".to_string(),
            ".".to_string(),
            vec![],
            false,
            false,
        );

        let workflow = preset.to_github().unwrap();

        assert_eq!(workflow.name, "Docker Build and Push");
        assert!(workflow.jobs.contains_key("docker/build"));

        let job = &workflow.jobs["docker/build"];
        assert_eq!(job.runs_on, "ubuntu-latest");

        // Should have checkout, buildx setup, metadata, and build steps
        assert!(job.steps.len() >= 4);
    }

    #[test]
    fn test_to_github_with_dockerhub() {
        let preset = DockerPreset::new(
            "myorg/myapp".to_string(),
            DockerRegistry::DockerHub,
            "./Dockerfile".to_string(),
            ".".to_string(),
            vec![],
            false,
            false,
        );

        let workflow = preset.to_github().unwrap();
        let job = &workflow.jobs["docker/build"];

        // Should include login step for DockerHub
        let has_dockerhub_login = job.steps.iter().any(|step| {
            step.name.as_ref().map(|n| n.contains("Docker Hub")).unwrap_or(false)
        });
        assert!(has_dockerhub_login);
    }

    #[test]
    fn test_to_github_with_github_registry() {
        let preset = DockerPreset::new(
            "myapp".to_string(),
            DockerRegistry::GitHubRegistry,
            "./Dockerfile".to_string(),
            ".".to_string(),
            vec![],
            false,
            false,
        );

        let workflow = preset.to_github().unwrap();
        let job = &workflow.jobs["docker/build"];

        // Should include login step for GHCR
        let has_ghcr_login = job.steps.iter().any(|step| {
            step.name.as_ref().map(|n| n.contains("GitHub Container Registry")).unwrap_or(false)
        });
        assert!(has_ghcr_login);
    }

    #[test]
    fn test_to_github_tags_only_trigger() {
        let preset = DockerPreset::new(
            "myapp".to_string(),
            DockerRegistry::DockerHub,
            "./Dockerfile".to_string(),
            ".".to_string(),
            vec![],
            false,
            true,
        );

        let workflow = preset.to_github().unwrap();

        // Verify trigger configuration
        if let GitHubTriggers::Detailed(triggers) = &workflow.on {
            if let Some(push_trigger) = triggers.get("push") {
                assert!(push_trigger.branches.is_none());
                assert!(push_trigger.tags.is_some());
            }
        }
    }

    #[test]
    fn test_to_gitlab_basic() {
        let preset = DockerPreset::new(
            "myapp".to_string(),
            DockerRegistry::None,
            "./Dockerfile".to_string(),
            ".".to_string(),
            vec![],
            false,
            false,
        );

        let config = preset.to_gitlab().unwrap();

        assert!(config.jobs.contains_key("docker/build"));
        assert_eq!(config.stages, Some(vec!["build".to_string()]));
    }

    #[test]
    fn test_to_circleci_basic() {
        let preset = DockerPreset::new(
            "myapp".to_string(),
            DockerRegistry::None,
            "./Dockerfile".to_string(),
            ".".to_string(),
            vec![],
            false,
            false,
        );

        let config = preset.to_circleci().unwrap();

        assert_eq!(config.version, "2.1");
        assert!(config.jobs.contains_key("docker/build"));
    }

    #[test]
    fn test_to_jenkins_basic() {
        let preset = DockerPreset::new(
            "myapp".to_string(),
            DockerRegistry::None,
            "./Dockerfile".to_string(),
            ".".to_string(),
            vec![],
            false,
            false,
        );

        let config = preset.to_jenkins().unwrap();

        assert_eq!(config.agent, "any");
        assert!(!config.stages.is_empty());
        assert_eq!(config.stages[0].name, "Docker Build");
    }

    #[test]
    fn test_preset_info() {
        let preset = DockerPreset::new(
            "myapp".to_string(),
            DockerRegistry::None,
            "./Dockerfile".to_string(),
            ".".to_string(),
            vec![],
            false,
            false,
        );

        assert_eq!(preset.name(), "docker");
        assert!(!preset.description().is_empty());
    }

    #[test]
    fn test_detectable_github() {
        let preset = DockerPreset::new(
            "myapp".to_string(),
            DockerRegistry::None,
            "./Dockerfile".to_string(),
            ".".to_string(),
            vec![],
            false,
            false,
        );

        // Create a workflow with docker build action
        let mut jobs = BTreeMap::new();
        jobs.insert(
            "build".to_string(),
            GitHubJob {
                runs_on: "ubuntu-latest".to_string(),
                steps: vec![GitHubStep {
                    name: Some("Build".to_string()),
                    uses: Some("docker/build-push-action@v5".to_string()),
                    run: None,
                    with: None,
                    env: None,
                }],
                needs: None,
                timeout_minutes: None,
                continue_on_error: None,
            },
        );

        let workflow = GitHubWorkflow {
            name: "Test".to_string(),
            on: GitHubTriggers::Simple(vec!["push".to_string()]),
            env: None,
            jobs,
        };

        assert!(preset.matches_github(&workflow));
    }

}
