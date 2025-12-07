use crate::error::Result;
use crate::platforms::circleci::models::CircleCIConfig;
use crate::platforms::github::models::{
    GitHubJob, GitHubStep, GitHubTriggerConfig, GitHubTriggers, GitHubWorkflow,
};
use crate::platforms::gitlab::models::GitLabCI;
use crate::platforms::jenkins::models::JenkinsConfig;
use crate::traits::{Detectable, PresetInfo, ToCircleCI, ToGitHub, ToGitLab, ToJenkins};
use std::collections::BTreeMap;

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
    /// Create a new builder for RustBinaryPreset
    pub fn builder() -> RustBinaryPresetBuilder {
        RustBinaryPresetBuilder::default()
    }
}

/// Builder for RustBinaryPreset
#[derive(Default)]
pub struct RustBinaryPresetBuilder {
    rust_version: Option<String>,
    enable_linter: bool,
    enable_security_scan: bool,
    enable_format_check: bool,
    build_release: bool,
}

impl RustBinaryPresetBuilder {
    /// Set the Rust toolchain version
    pub fn rust_version(mut self, version: impl Into<String>) -> Self {
        self.rust_version = Some(version.into());
        self
    }

    /// Enable or disable linting with clippy
    pub fn linter(mut self, enable: bool) -> Self {
        self.enable_linter = enable;
        self
    }

    /// Enable or disable security scanning
    pub fn security_scan(mut self, enable: bool) -> Self {
        self.enable_security_scan = enable;
        self
    }

    /// Enable or disable format checking with rustfmt
    pub fn format_check(mut self, enable: bool) -> Self {
        self.enable_format_check = enable;
        self
    }

    /// Enable or disable release builds
    pub fn build_release(mut self, enable: bool) -> Self {
        self.build_release = enable;
        self
    }

    /// Build the RustBinaryPreset
    pub fn build(self) -> RustBinaryPreset {
        RustBinaryPreset {
            rust_version: self.rust_version.unwrap_or_else(|| "stable".to_string()),
            enable_linter: self.enable_linter,
            enable_security_scan: self.enable_security_scan,
            enable_format_check: self.enable_format_check,
            build_release: self.build_release,
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
