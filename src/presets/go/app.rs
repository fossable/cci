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

/// Preset for Go application projects
#[derive(Debug, Clone)]
pub struct GoAppPreset {
    go_version: String,
    enable_linter: bool,
    enable_security_scan: bool,
}

impl GoAppPreset {
    pub fn new(go_version: String, enable_linter: bool, enable_security_scan: bool) -> Self {
        Self {
            go_version,
            enable_linter,
            enable_security_scan,
        }
    }

    /// Create a new GoAppPreset from editor configuration
    pub fn from_config(config: &PresetConfig, version: &str) -> Self {
        Self::new(
            version.to_string(),
            config.get_bool("enable_linter"),
            config.get_bool("enable_security"),
        )
    }

    /// Constant default instance for registry initialization
    pub const DEFAULT: Self = Self {
        go_version: String::new(),
        enable_linter: false,
        enable_security_scan: false,
    };
}

impl Default for GoAppPreset {
    fn default() -> Self {
        Self {
            go_version: "1.21".to_string(),
            enable_linter: false,
            enable_security_scan: false,
        }
    }
}

impl ToGitHub for GoAppPreset {
    fn to_github(&self) -> Result<GitHubWorkflow> {
        let mut jobs = BTreeMap::new();

        // Test job (always present)
        let test_steps = vec![
            GitHubStep {
                name: Some("Checkout code".to_string()),
                uses: Some("actions/checkout@v4".to_string()),
                run: None,
                with: None,
                env: None,
            },
            GitHubStep {
                name: Some("Setup Go".to_string()),
                uses: Some("actions/setup-go@v5".to_string()),
                run: None,
                with: Some(BTreeMap::from([(
                    "go-version".to_string(),
                    serde_yaml::Value::String(self.go_version.clone()),
                )])),
                env: None,
            },
            GitHubStep {
                name: Some("Download dependencies".to_string()),
                uses: None,
                run: Some("go mod download".to_string()),
                with: None,
                env: None,
            },
            GitHubStep {
                name: Some("Run tests".to_string()),
                uses: None,
                run: Some("go test -v ./...".to_string()),
                with: None,
                env: None,
            },
            GitHubStep {
                name: Some("Build".to_string()),
                uses: None,
                run: Some("go build -v ./...".to_string()),
                with: None,
                env: None,
            },
        ];

        jobs.insert(
            "go/test".to_string(),
            GitHubJob {
                runs_on: "ubuntu-latest".to_string(),
                steps: test_steps,
                needs: None,
                timeout_minutes: Some(30),
                continue_on_error: None,
            },
        );

        // Lint job (optional)
        if self.enable_linter {
            jobs.insert(
                "go/lint".to_string(),
                GitHubJob {
                    runs_on: "ubuntu-latest".to_string(),
                    steps: vec![
                        GitHubStep {
                            name: Some("Checkout code".to_string()),
                            uses: Some("actions/checkout@v4".to_string()),
                            run: None,
                            with: None,
                            env: None,
                        },
                        GitHubStep {
                            name: Some("Setup Go".to_string()),
                            uses: Some("actions/setup-go@v5".to_string()),
                            run: None,
                            with: Some(BTreeMap::from([(
                                "go-version".to_string(),
                                serde_yaml::Value::String(self.go_version.clone()),
                            )])),
                            env: None,
                        },
                        GitHubStep {
                            name: Some("Run golangci-lint".to_string()),
                            uses: Some("golangci/golangci-lint-action@v3".to_string()),
                            run: None,
                            with: Some(BTreeMap::from([(
                                "version".to_string(),
                                serde_yaml::Value::String("latest".to_string()),
                            )])),
                            env: None,
                        },
                    ],
                    needs: None,
                    timeout_minutes: Some(15),
                    continue_on_error: None,
                },
            );
        }

        // Security scan job (optional)
        if self.enable_security_scan {
            jobs.insert(
                "go/security".to_string(),
                GitHubJob {
                    runs_on: "ubuntu-latest".to_string(),
                    steps: vec![
                        GitHubStep {
                            name: Some("Checkout code".to_string()),
                            uses: Some("actions/checkout@v4".to_string()),
                            run: None,
                            with: None,
                            env: None,
                        },
                        GitHubStep {
                            name: Some("Setup Go".to_string()),
                            uses: Some("actions/setup-go@v5".to_string()),
                            run: None,
                            with: Some(BTreeMap::from([(
                                "go-version".to_string(),
                                serde_yaml::Value::String(self.go_version.clone()),
                            )])),
                            env: None,
                        },
                        GitHubStep {
                            name: Some("Run gosec".to_string()),
                            uses: Some("securego/gosec@master".to_string()),
                            run: None,
                            with: Some(BTreeMap::from([(
                                "args".to_string(),
                                serde_yaml::Value::String("./...".to_string()),
                            )])),
                            env: None,
                        },
                    ],
                    needs: None,
                    timeout_minutes: Some(10),
                    continue_on_error: None,
                },
            );
        }

        Ok(GitHubWorkflow {
            name: "CI".to_string(),
            on: GitHubTriggers::Detailed(BTreeMap::from([
                (
                    "push".to_string(),
                    GitHubTriggerConfig {
                        branches: Some(vec!["main".to_string(), "master".to_string()]),
                        tags: None,
                    },
                ),
                (
                    "pull_request".to_string(),
                    GitHubTriggerConfig {
                        branches: Some(vec!["main".to_string(), "master".to_string()]),
                        tags: None,
                    },
                ),
            ])),
            env: None,
            jobs,
        })
    }
}

impl ToGitea for GoAppPreset {
    fn to_gitea(&self) -> Result<crate::platforms::gitea::models::GiteaWorkflow> {
        // Gitea Actions uses the same workflow format as GitHub Actions
        self.to_github()
    }
}

impl ToGitLab for GoAppPreset {
    fn to_gitlab(&self) -> Result<GitLabCI> {
        use crate::platforms::gitlab::models::*;
        use std::collections::BTreeMap;

        let mut jobs = BTreeMap::new();

        let mut script = vec!["go test -v ./...".to_string()];

        if self.enable_linter {
            script.insert(0, "golangci-lint run".to_string());
        }

        if self.enable_security_scan {
            script.insert(0, "gosec ./...".to_string());
        }

        jobs.insert(
            "go/test".to_string(),
            GitLabJob {
                stage: "test".to_string(),
                image: Some(format!("golang:{}", self.go_version)),
                script,
                before_script: None,
                after_script: None,
                needs: None,
                cache: Some(GitLabCache {
                    key: "go-cache".to_string(),
                    paths: vec!["~/go/pkg/mod".to_string()],
                }),
                artifacts: None,
                only: None,
                timeout: None,
            },
        );

        Ok(GitLabCI {
            stages: Some(vec!["test".to_string()]),
            variables: None,
            cache: None,
            jobs,
        })
    }
}

impl ToCircleCI for GoAppPreset {
    fn to_circleci(&self) -> Result<CircleCIConfig> {
        use crate::platforms::circleci::models::*;
        use std::collections::BTreeMap;

        let mut steps = vec![CircleCIStep::Simple("checkout".to_string())];

        if self.enable_security_scan {
            steps.push(CircleCIStep::Command {
                run: CircleCIRun::Detailed {
                    name: "Security scan".to_string(),
                    command: "gosec ./...".to_string(),
                },
            });
        }

        if self.enable_linter {
            steps.push(CircleCIStep::Command {
                run: CircleCIRun::Detailed {
                    name: "Lint".to_string(),
                    command: "golangci-lint run".to_string(),
                },
            });
        }

        steps.push(CircleCIStep::Command {
            run: CircleCIRun::Detailed {
                name: "Run tests".to_string(),
                command: "go test -v ./...".to_string(),
            },
        });

        let mut jobs = BTreeMap::new();
        jobs.insert(
            "go/test".to_string(),
            CircleCIJob {
                docker: vec![CircleCIDocker {
                    image: format!("golang:{}", self.go_version),
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
                    jobs: vec![CircleCIWorkflowJob::Simple("go/test".to_string())],
                },
            )]),
        })
    }
}

impl ToJenkins for GoAppPreset {
    fn to_jenkins(&self) -> Result<JenkinsConfig> {
        use crate::platforms::jenkins::models::*;

        let mut test_steps = Vec::new();

        if self.enable_security_scan {
            test_steps.push("sh 'gosec ./...'".to_string());
        }

        if self.enable_linter {
            test_steps.push("sh 'golangci-lint run'".to_string());
        }

        test_steps.push("sh 'go test -v ./...'".to_string());

        Ok(JenkinsConfig {
            agent: "any".to_string(),
            environment: vec![],
            stages: vec![JenkinsStage {
                name: "Test".to_string(),
                steps: test_steps,
            }],
        })
    }
}

impl Detectable for GoAppPreset {
    fn matches_github(&self, workflow: &GitHubWorkflow) -> bool {
        let has_go_setup = workflow.jobs.values().any(|job| {
            job.steps.iter().any(|step| {
                step.uses
                    .as_ref()
                    .map(|u| u.contains("setup-go"))
                    .unwrap_or(false)
            })
        });

        let has_go_test = workflow.jobs.values().any(|job| {
            job.steps.iter().any(|step| {
                step.run
                    .as_ref()
                    .map(|r| r.contains("go test"))
                    .unwrap_or(false)
            })
        });

        has_go_setup && has_go_test
    }

    fn matches_gitea(&self, workflow: &crate::platforms::gitea::models::GiteaWorkflow) -> bool {
        // Gitea Actions uses the same workflow format as GitHub Actions
        self.matches_github(workflow)
    }

    fn matches_gitlab(&self, _config: &GitLabCI) -> bool {
        false
    }

    fn matches_circleci(&self, _config: &CircleCIConfig) -> bool {
        false
    }

    fn matches_jenkins(&self, _config: &JenkinsConfig) -> bool {
        false
    }
}

impl PresetInfo for GoAppPreset {
    fn name(&self) -> &str {
        "go-app"
    }

    fn description(&self) -> &str {
        "CI pipeline for Go applications with testing, linting, and security scanning"
    }
}

impl EditorPreset for GoAppPreset {
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
        let preset = Self::from_config(config, language_version);
        generate_for_platform(&preset, platform)
    }

    fn matches_project(&self, project_type: &ProjectType, _working_dir: &Path) -> bool {
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
