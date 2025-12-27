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

/// Preset for Rust binary/application projects
#[derive(Debug, Clone)]
pub struct RustBinaryPreset {
    rust_version: String,
    enable_linter: bool,
    enable_security_scan: bool,
    enable_format_check: bool,
    build_release: bool,
}

impl RustBinaryPreset {
    pub fn new(
        rust_version: String,
        enable_linter: bool,
        enable_security_scan: bool,
        enable_format_check: bool,
        build_release: bool,
    ) -> Self {
        Self {
            rust_version,
            enable_linter,
            enable_security_scan,
            enable_format_check,
            build_release,
        }
    }

    /// Create a new RustBinaryPreset from editor configuration
    pub fn from_config(config: &PresetConfig, version: &str) -> Self {
        Self::new(
            version.to_string(),
            config.get_bool("enable_linter"),
            false, // enable_security_scan (not exposed in TUI)
            false, // enable_format_check (not exposed in TUI)
            config.get_bool("build_release"),
        )
    }

    /// Constant default instance for registry initialization
    pub const DEFAULT: Self = Self {
        rust_version: String::new(),
        enable_linter: false,
        enable_security_scan: false,
        enable_format_check: false,
        build_release: false,
    };
}

impl Default for RustBinaryPreset {
    fn default() -> Self {
        Self {
            rust_version: "stable".to_string(),
            enable_linter: false,
            enable_security_scan: false,
            enable_format_check: false,
            build_release: false,
        }
    }
}

impl ToGitHub for RustBinaryPreset {
    fn to_github(&self) -> Result<GitHubWorkflow> {
        let mut jobs = BTreeMap::new();

        // Test job (always present)
        let mut test_steps = vec![
            GitHubStep {
                name: Some("Checkout code".to_string()),
                uses: Some("actions/checkout@v4".to_string()),
                run: None,
                with: None,
                env: None,
            },
            GitHubStep {
                name: Some("Setup Rust toolchain".to_string()),
                uses: Some("dtolnay/rust-toolchain@stable".to_string()),
                run: None,
                with: Some(BTreeMap::from([(
                    "toolchain".to_string(),
                    serde_yaml::Value::String(self.rust_version.clone()),
                )])),
                env: None,
            },
            GitHubStep {
                name: Some("Cache dependencies".to_string()),
                uses: Some("Swatinem/rust-cache@v2".to_string()),
                run: None,
                with: None,
                env: None,
            },
            GitHubStep {
                name: Some("Run tests".to_string()),
                uses: None,
                run: Some("cargo test".to_string()),
                with: None,
                env: None,
            },
        ];

        if self.build_release {
            test_steps.push(GitHubStep {
                name: Some("Build release binary".to_string()),
                uses: None,
                run: Some("cargo build --release".to_string()),
                with: None,
                env: None,
            });
        }

        jobs.insert(
            "rust/test".to_string(),
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
                "rust/lint".to_string(),
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
                            name: Some("Setup Rust toolchain".to_string()),
                            uses: Some("dtolnay/rust-toolchain@stable".to_string()),
                            run: None,
                            with: Some(BTreeMap::from([
                                (
                                    "toolchain".to_string(),
                                    serde_yaml::Value::String(self.rust_version.clone()),
                                ),
                                (
                                    "components".to_string(),
                                    serde_yaml::Value::String("clippy".to_string()),
                                ),
                            ])),
                            env: None,
                        },
                        GitHubStep {
                            name: Some("Run clippy".to_string()),
                            uses: None,
                            run: Some("cargo clippy -- -D warnings".to_string()),
                            with: None,
                            env: None,
                        },
                    ],
                    needs: None,
                    timeout_minutes: Some(15),
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

impl ToGitea for RustBinaryPreset {
    fn to_gitea(&self) -> Result<crate::platforms::gitea::models::GiteaWorkflow> {
        // Gitea Actions uses the same workflow format as GitHub Actions
        self.to_github()
    }
}

impl ToGitLab for RustBinaryPreset {
    fn to_gitlab(&self) -> Result<GitLabCI> {
        use crate::platforms::gitlab::models::*;
        use std::collections::BTreeMap;

        let mut jobs = BTreeMap::new();

        // Test job
        jobs.insert(
            "rust/test".to_string(),
            GitLabJob {
                stage: "test".to_string(),
                image: Some(format!("rust:{}", self.rust_version)),
                script: vec![
                    "cargo test --verbose".to_string(),
                ],
                before_script: None,
                after_script: None,
                needs: None,
                cache: Some(GitLabCache {
                    key: "rust-cache".to_string(),
                    paths: vec!["target/".to_string()],
                }),
                artifacts: None,
                only: None,
                timeout: None,
            },
        );

        // Build job
        if self.build_release {
            jobs.insert(
                "rust/build".to_string(),
                GitLabJob {
                    stage: "build".to_string(),
                    image: Some(format!("rust:{}", self.rust_version)),
                    script: vec![
                        "cargo build --release".to_string(),
                    ],
                    before_script: None,
                    after_script: None,
                    needs: None,
                    cache: Some(GitLabCache {
                        key: "rust-cache".to_string(),
                        paths: vec!["target/".to_string()],
                    }),
                    artifacts: Some(GitLabArtifacts {
                        paths: vec!["target/release/".to_string()],
                        name: None,
                    }),
                    only: None,
                    timeout: None,
                },
            );
        }

        Ok(GitLabCI {
            stages: Some(vec!["test".to_string(), "build".to_string()]),
            variables: None,
            cache: None,
            jobs,
        })
    }
}

impl ToCircleCI for RustBinaryPreset {
    fn to_circleci(&self) -> Result<CircleCIConfig> {
        use crate::platforms::circleci::models::*;
        use std::collections::BTreeMap;

        let mut jobs = BTreeMap::new();

        // Test job
        jobs.insert(
            "rust/test".to_string(),
            CircleCIJob {
                docker: vec![CircleCIDocker {
                    image: format!("rust:{}", self.rust_version),
                }],
                steps: vec![
                    CircleCIStep::Simple("checkout".to_string()),
                    CircleCIStep::Command {
                        run: CircleCIRun::Detailed {
                            name: "Run tests".to_string(),
                            command: "cargo test --verbose".to_string(),
                        },
                    },
                ],
                environment: None,
            },
        );

        // Build job
        if self.build_release {
            jobs.insert(
                "rust/build".to_string(),
                CircleCIJob {
                    docker: vec![CircleCIDocker {
                        image: format!("rust:{}", self.rust_version),
                    }],
                    steps: vec![
                        CircleCIStep::Simple("checkout".to_string()),
                        CircleCIStep::Command {
                            run: CircleCIRun::Detailed {
                                name: "Build release".to_string(),
                                command: "cargo build --release".to_string(),
                            },
                        },
                    ],
                    environment: None,
                },
            );
        }

        Ok(CircleCIConfig {
            version: "2.1".to_string(),
            orbs: None,
            jobs,
            workflows: BTreeMap::from([(
                "main".to_string(),
                CircleCIWorkflow {
                    jobs: if self.build_release {
                        vec![
                            CircleCIWorkflowJob::Simple("rust/test".to_string()),
                            CircleCIWorkflowJob::Simple("rust/build".to_string()),
                        ]
                    } else {
                        vec![CircleCIWorkflowJob::Simple("rust/test".to_string())]
                    },
                },
            )]),
        })
    }
}

impl ToJenkins for RustBinaryPreset {
    fn to_jenkins(&self) -> Result<JenkinsConfig> {
        use crate::platforms::jenkins::models::*;

        let mut stages = vec![JenkinsStage {
            name: "Test".to_string(),
            steps: vec!["sh 'cargo test --verbose'".to_string()],
        }];

        if self.build_release {
            stages.push(JenkinsStage {
                name: "Build".to_string(),
                steps: vec!["sh 'cargo build --release'".to_string()],
            });
        }

        Ok(JenkinsConfig {
            agent: "any".to_string(),
            environment: vec![],
            stages,
        })
    }
}

impl Detectable for RustBinaryPreset {
    fn matches_github(&self, workflow: &GitHubWorkflow) -> bool {
        // Check for Rust toolchain and cargo build (not just test)
        let has_rust_toolchain = workflow.jobs.values().any(|job| {
            job.steps.iter().any(|step| {
                step.uses
                    .as_ref()
                    .map(|u| u.contains("rust-toolchain"))
                    .unwrap_or(false)
            })
        });

        let has_cargo_build = workflow.jobs.values().any(|job| {
            job.steps.iter().any(|step| {
                step.run
                    .as_ref()
                    .map(|r| r.contains("cargo build"))
                    .unwrap_or(false)
            })
        });

        has_rust_toolchain && (has_cargo_build || workflow.jobs.contains_key("rust/build"))
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

impl PresetInfo for RustBinaryPreset {
    fn name(&self) -> &str {
        "rust-binary"
    }

    fn description(&self) -> &str {
        "CI pipeline for Rust binary/application projects with building and testing"
    }
}

impl EditorPreset for RustBinaryPreset {
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
        let preset = Self::from_config(config, language_version);
        generate_for_platform(&preset, platform)
    }

    fn matches_project(&self, project_type: &ProjectType, _working_dir: &Path) -> bool {
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
