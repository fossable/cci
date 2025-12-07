use crate::error::Result;
use crate::platforms::circleci::models::CircleCIConfig;
use crate::platforms::github::models::{
    GitHubJob, GitHubStep, GitHubTriggerConfig, GitHubTriggers, GitHubWorkflow,
};
use crate::platforms::gitlab::models::GitLabCI;
use crate::platforms::jenkins::models::JenkinsConfig;
use crate::traits::{Detectable, PresetInfo, ToCircleCI, ToGitHub, ToGitLab, ToJenkins};
use std::collections::BTreeMap;

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
    /// Create a new builder for DockerPreset
    pub fn builder() -> DockerPresetBuilder {
        DockerPresetBuilder::default()
    }
}

/// Builder for DockerPreset
#[derive(Default)]
pub struct DockerPresetBuilder {
    image_name: Option<String>,
    registry: Option<DockerRegistry>,
    dockerfile_path: Option<String>,
    build_context: Option<String>,
    build_args: Vec<(String, String)>,
    enable_cache: bool,
    push_on_tags_only: bool,
}

impl DockerPresetBuilder {
    /// Set the Docker image name (required)
    pub fn image_name(mut self, name: impl Into<String>) -> Self {
        self.image_name = Some(name.into());
        self
    }

    /// Set the registry to push to
    pub fn registry(mut self, registry: DockerRegistry) -> Self {
        self.registry = Some(registry);
        self
    }

    /// Set the Dockerfile path
    pub fn dockerfile_path(mut self, path: impl Into<String>) -> Self {
        self.dockerfile_path = Some(path.into());
        self
    }

    /// Set the Docker build context
    pub fn build_context(mut self, context: impl Into<String>) -> Self {
        self.build_context = Some(context.into());
        self
    }

    /// Add a build argument
    pub fn build_arg(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.build_args.push((key.into(), value.into()));
        self
    }

    /// Enable or disable Docker layer caching
    pub fn cache(mut self, enable: bool) -> Self {
        self.enable_cache = enable;
        self
    }

    /// Only push images on git tags (not on branch pushes)
    pub fn push_on_tags_only(mut self, enable: bool) -> Self {
        self.push_on_tags_only = enable;
        self
    }

    /// Build the DockerPreset
    pub fn build(self) -> DockerPreset {
        DockerPreset {
            image_name: self
                .image_name
                .expect("image_name is required for DockerPreset"),
            registry: self.registry.unwrap_or(DockerRegistry::None),
            dockerfile_path: self
                .dockerfile_path
                .unwrap_or_else(|| "./Dockerfile".to_string()),
            build_context: self.build_context.unwrap_or_else(|| ".".to_string()),
            build_args: self.build_args,
            enable_cache: self.enable_cache,
            push_on_tags_only: self.push_on_tags_only,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let preset = DockerPreset::builder()
            .image_name("myapp")
            .build();

        assert_eq!(preset.image_name, "myapp");
        assert_eq!(preset.registry, DockerRegistry::None);
        assert_eq!(preset.dockerfile_path, "./Dockerfile");
        assert_eq!(preset.build_context, ".");
        assert!(!preset.enable_cache);
        assert!(!preset.push_on_tags_only);
    }

    #[test]
    fn test_builder_with_dockerhub() {
        let preset = DockerPreset::builder()
            .image_name("myorg/myapp")
            .registry(DockerRegistry::DockerHub)
            .cache(true)
            .build();

        assert_eq!(preset.registry, DockerRegistry::DockerHub);
        assert!(preset.enable_cache);
    }

    #[test]
    fn test_builder_with_github_registry() {
        let preset = DockerPreset::builder()
            .image_name("myapp")
            .registry(DockerRegistry::GitHubRegistry)
            .push_on_tags_only(true)
            .build();

        assert_eq!(preset.registry, DockerRegistry::GitHubRegistry);
        assert!(preset.push_on_tags_only);
    }

    #[test]
    fn test_builder_with_custom_paths() {
        let preset = DockerPreset::builder()
            .image_name("myapp")
            .dockerfile_path("./docker/Dockerfile")
            .build_context("./app")
            .build_arg("VERSION", "1.0.0")
            .build_arg("BUILD_DATE", "2024-01-01")
            .build();

        assert_eq!(preset.dockerfile_path, "./docker/Dockerfile");
        assert_eq!(preset.build_context, "./app");
        assert_eq!(preset.build_args.len(), 2);
    }

    #[test]
    fn test_to_github_basic() {
        let preset = DockerPreset::builder()
            .image_name("myapp")
            .build();

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
        let preset = DockerPreset::builder()
            .image_name("myorg/myapp")
            .registry(DockerRegistry::DockerHub)
            .build();

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
        let preset = DockerPreset::builder()
            .image_name("myapp")
            .registry(DockerRegistry::GitHubRegistry)
            .build();

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
        let preset = DockerPreset::builder()
            .image_name("myapp")
            .registry(DockerRegistry::DockerHub)
            .push_on_tags_only(true)
            .build();

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
        let preset = DockerPreset::builder()
            .image_name("myapp")
            .build();

        let config = preset.to_gitlab().unwrap();

        assert!(config.jobs.contains_key("docker/build"));
        assert_eq!(config.stages, Some(vec!["build".to_string()]));
    }

    #[test]
    fn test_to_circleci_basic() {
        let preset = DockerPreset::builder()
            .image_name("myapp")
            .build();

        let config = preset.to_circleci().unwrap();

        assert_eq!(config.version, "2.1");
        assert!(config.jobs.contains_key("docker/build"));
    }

    #[test]
    fn test_to_jenkins_basic() {
        let preset = DockerPreset::builder()
            .image_name("myapp")
            .build();

        let config = preset.to_jenkins().unwrap();

        assert_eq!(config.agent, "any");
        assert!(!config.stages.is_empty());
        assert_eq!(config.stages[0].name, "Docker Build");
    }

    #[test]
    fn test_preset_info() {
        let preset = DockerPreset::builder()
            .image_name("myapp")
            .build();

        assert_eq!(preset.name(), "docker");
        assert!(!preset.description().is_empty());
    }

    #[test]
    fn test_detectable_github() {
        let preset = DockerPreset::builder()
            .image_name("myapp")
            .build();

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

    #[test]
    #[should_panic(expected = "image_name is required")]
    fn test_builder_without_image_name() {
        DockerPreset::builder().build();
    }
}
