use crate::error::Result;
use crate::platforms::circleci::models::CircleCIConfig;
use crate::platforms::github::models::{
    GitHubJob, GitHubStep, GitHubTriggerConfig, GitHubTriggers, GitHubWorkflow,
};
use crate::platforms::gitlab::models::GitLabCI;
use crate::platforms::jenkins::models::JenkinsConfig;
use crate::traits::{Detectable, PresetInfo, ToCircleCI, ToGitHub, ToGitLab, ToJenkins};
use std::collections::BTreeMap;

/// Preset for Rust library projects
#[derive(Debug, Clone)]
pub struct RustLibraryPreset {
    rust_version: String,
    enable_coverage: bool,
    enable_linter: bool,
    enable_security_scan: bool,
    enable_format_check: bool,
}

impl RustLibraryPreset {
    /// Create a new builder for RustLibraryPreset
    pub fn builder() -> RustLibraryPresetBuilder {
        RustLibraryPresetBuilder::default()
    }
}

/// Builder for RustLibraryPreset
#[derive(Default)]
pub struct RustLibraryPresetBuilder {
    rust_version: Option<String>,
    enable_coverage: bool,
    enable_linter: bool,
    enable_security_scan: bool,
    enable_format_check: bool,
}

impl RustLibraryPresetBuilder {
    /// Set the Rust toolchain version
    pub fn rust_version(mut self, version: impl Into<String>) -> Self {
        self.rust_version = Some(version.into());
        self
    }

    /// Enable or disable coverage reporting
    pub fn coverage(mut self, enable: bool) -> Self {
        self.enable_coverage = enable;
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

    /// Build the RustLibraryPreset
    pub fn build(self) -> RustLibraryPreset {
        RustLibraryPreset {
            rust_version: self.rust_version.unwrap_or_else(|| "stable".to_string()),
            enable_coverage: self.enable_coverage,
            enable_linter: self.enable_linter,
            enable_security_scan: self.enable_security_scan,
            enable_format_check: self.enable_format_check,
        }
    }
}

impl ToGitHub for RustLibraryPreset {
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
                run: Some("cargo test --all-features".to_string()),
                with: None,
                env: None,
            },
        ];

        if self.enable_coverage {
            test_steps.push(GitHubStep {
                name: Some("Install tarpaulin".to_string()),
                uses: None,
                run: Some("cargo install cargo-tarpaulin".to_string()),
                with: None,
                env: None,
            });
            test_steps.push(GitHubStep {
                name: Some("Generate coverage".to_string()),
                uses: None,
                run: Some("cargo tarpaulin --out Xml --all-features".to_string()),
                with: None,
                env: None,
            });
            test_steps.push(GitHubStep {
                name: Some("Upload coverage to Codecov".to_string()),
                uses: Some("codecov/codecov-action@v3".to_string()),
                run: None,
                with: None,
                env: None,
            });
        }

        jobs.insert(
            "test".to_string(),
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
                "lint".to_string(),
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
                            run: Some("cargo clippy --all-features -- -D warnings".to_string()),
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

        // Format check job (optional)
        if self.enable_format_check {
            jobs.insert(
                "format".to_string(),
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
                                    serde_yaml::Value::String("rustfmt".to_string()),
                                ),
                            ])),
                            env: None,
                        },
                        GitHubStep {
                            name: Some("Check formatting".to_string()),
                            uses: None,
                            run: Some("cargo fmt -- --check".to_string()),
                            with: None,
                            env: None,
                        },
                    ],
                    needs: None,
                    timeout_minutes: Some(10),
                    continue_on_error: None,
                },
            );
        }

        // Security scan job (optional)
        if self.enable_security_scan {
            jobs.insert(
                "security".to_string(),
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
                            name: Some("Run cargo audit".to_string()),
                            uses: Some("rustsec/audit-check@v1".to_string()),
                            run: None,
                            with: Some(BTreeMap::from([(
                                "token".to_string(),
                                serde_yaml::Value::String("${{ secrets.GITHUB_TOKEN }}".to_string()),
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

impl ToGitLab for RustLibraryPreset {
    fn to_gitlab(&self) -> Result<GitLabCI> {
        use crate::platforms::gitlab::models::{GitLabCache, GitLabJob, GitLabOnly};

        let mut jobs = BTreeMap::new();
        let mut stages = vec!["test".to_string()];

        // Test job (always present)
        let mut test_script = vec![
            format!("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain {}", self.rust_version),
            "source $HOME/.cargo/env".to_string(),
            "cargo test --all-features".to_string(),
        ];

        if self.enable_coverage {
            test_script.push("cargo install cargo-tarpaulin".to_string());
            test_script.push("cargo tarpaulin --out Xml --all-features".to_string());
        }

        jobs.insert(
            "test".to_string(),
            GitLabJob {
                stage: "test".to_string(),
                image: Some("rust:latest".to_string()),
                script: test_script,
                before_script: None,
                after_script: None,
                needs: None,
                cache: Some(GitLabCache {
                    key: "rust-cache".to_string(),
                    paths: vec![
                        "target/".to_string(),
                        ".cargo/".to_string(),
                    ],
                }),
                artifacts: if self.enable_coverage {
                    Some(crate::platforms::gitlab::models::GitLabArtifacts {
                        paths: vec!["cobertura.xml".to_string()],
                        name: Some("coverage".to_string()),
                    })
                } else {
                    None
                },
                only: Some(GitLabOnly {
                    refs: Some(vec!["main".to_string(), "master".to_string(), "merge_requests".to_string()]),
                }),
                timeout: Some("30m".to_string()),
            },
        );

        // Lint job (optional)
        if self.enable_linter {
            stages.push("lint".to_string());
            jobs.insert(
                "lint".to_string(),
                GitLabJob {
                    stage: "lint".to_string(),
                    image: Some("rust:latest".to_string()),
                    script: vec![
                        format!("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain {} --component clippy", self.rust_version),
                        "source $HOME/.cargo/env".to_string(),
                        "cargo clippy --all-features -- -D warnings".to_string(),
                    ],
                    before_script: None,
                    after_script: None,
                    needs: None,
                    cache: Some(GitLabCache {
                        key: "rust-cache".to_string(),
                        paths: vec!["target/".to_string(), ".cargo/".to_string()],
                    }),
                    artifacts: None,
                    only: Some(GitLabOnly {
                        refs: Some(vec!["main".to_string(), "master".to_string(), "merge_requests".to_string()]),
                    }),
                    timeout: Some("15m".to_string()),
                },
            );
        }

        // Format check job (optional)
        if self.enable_format_check {
            if !stages.contains(&"lint".to_string()) {
                stages.push("lint".to_string());
            }
            jobs.insert(
                "format".to_string(),
                GitLabJob {
                    stage: "lint".to_string(),
                    image: Some("rust:latest".to_string()),
                    script: vec![
                        format!("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain {} --component rustfmt", self.rust_version),
                        "source $HOME/.cargo/env".to_string(),
                        "cargo fmt -- --check".to_string(),
                    ],
                    before_script: None,
                    after_script: None,
                    needs: None,
                    cache: None,
                    artifacts: None,
                    only: Some(GitLabOnly {
                        refs: Some(vec!["main".to_string(), "master".to_string(), "merge_requests".to_string()]),
                    }),
                    timeout: Some("10m".to_string()),
                },
            );
        }

        // Security scan job (optional)
        if self.enable_security_scan {
            if !stages.contains(&"security".to_string()) {
                stages.push("security".to_string());
            }
            jobs.insert(
                "security".to_string(),
                GitLabJob {
                    stage: "security".to_string(),
                    image: Some("rust:latest".to_string()),
                    script: vec![
                        "cargo install cargo-audit".to_string(),
                        "cargo audit".to_string(),
                    ],
                    before_script: None,
                    after_script: None,
                    needs: None,
                    cache: Some(GitLabCache {
                        key: "cargo-audit-cache".to_string(),
                        paths: vec![".cargo/".to_string()],
                    }),
                    artifacts: None,
                    only: Some(GitLabOnly {
                        refs: Some(vec!["main".to_string(), "master".to_string(), "merge_requests".to_string()]),
                    }),
                    timeout: Some("10m".to_string()),
                },
            );
        }

        Ok(GitLabCI {
            stages: Some(stages),
            variables: None,
            cache: None,
            jobs,
        })
    }
}

impl ToCircleCI for RustLibraryPreset {
    fn to_circleci(&self) -> Result<CircleCIConfig> {
        use crate::platforms::circleci::models::*;

        let mut jobs = BTreeMap::new();
        let mut workflow_jobs = vec![];

        // Test job (always present)
        let mut test_steps = vec![
            CircleCIStep::Simple("checkout".to_string()),
            CircleCIStep::Cache {
                restore_cache: CircleCICache {
                    keys: vec!["v1-cargo-cache-{{ checksum \"Cargo.lock\" }}".to_string()],
                },
            },
            CircleCIStep::Command {
                run: CircleCIRun::Detailed {
                    name: "Install Rust".to_string(),
                    command: format!(
                        "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain {}",
                        self.rust_version
                    ),
                },
            },
            CircleCIStep::Command {
                run: CircleCIRun::Simple("source $HOME/.cargo/env".to_string()),
            },
            CircleCIStep::Command {
                run: CircleCIRun::Detailed {
                    name: "Run tests".to_string(),
                    command: "cargo test --all-features".to_string(),
                },
            },
        ];

        if self.enable_coverage {
            test_steps.push(CircleCIStep::Command {
                run: CircleCIRun::Detailed {
                    name: "Install tarpaulin".to_string(),
                    command: "cargo install cargo-tarpaulin".to_string(),
                },
            });
            test_steps.push(CircleCIStep::Command {
                run: CircleCIRun::Detailed {
                    name: "Generate coverage".to_string(),
                    command: "cargo tarpaulin --out Xml --all-features".to_string(),
                },
            });
        }

        test_steps.push(CircleCIStep::SaveCache {
            save_cache: CircleCICacheSave {
                key: "v1-cargo-cache-{{ checksum \"Cargo.lock\" }}".to_string(),
                paths: vec!["~/.cargo".to_string(), "./target".to_string()],
            },
        });

        jobs.insert(
            "test".to_string(),
            CircleCIJob {
                docker: vec![CircleCIDocker {
                    image: "rust:latest".to_string(),
                }],
                steps: test_steps,
                environment: None,
            },
        );

        workflow_jobs.push(CircleCIWorkflowJob::Simple("test".to_string()));

        // Lint job (optional)
        if self.enable_linter {
            jobs.insert(
                "lint".to_string(),
                CircleCIJob {
                    docker: vec![CircleCIDocker {
                        image: "rust:latest".to_string(),
                    }],
                    steps: vec![
                        CircleCIStep::Simple("checkout".to_string()),
                        CircleCIStep::Command {
                            run: CircleCIRun::Detailed {
                                name: "Install Rust with clippy".to_string(),
                                command: format!(
                                    "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain {} --component clippy",
                                    self.rust_version
                                ),
                            },
                        },
                        CircleCIStep::Command {
                            run: CircleCIRun::Simple("source $HOME/.cargo/env".to_string()),
                        },
                        CircleCIStep::Command {
                            run: CircleCIRun::Detailed {
                                name: "Run clippy".to_string(),
                                command: "cargo clippy --all-features -- -D warnings".to_string(),
                            },
                        },
                    ],
                    environment: None,
                },
            );
            workflow_jobs.push(CircleCIWorkflowJob::Simple("lint".to_string()));
        }

        // Format check job (optional)
        if self.enable_format_check {
            jobs.insert(
                "format".to_string(),
                CircleCIJob {
                    docker: vec![CircleCIDocker {
                        image: "rust:latest".to_string(),
                    }],
                    steps: vec![
                        CircleCIStep::Simple("checkout".to_string()),
                        CircleCIStep::Command {
                            run: CircleCIRun::Detailed {
                                name: "Install Rust with rustfmt".to_string(),
                                command: format!(
                                    "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain {} --component rustfmt",
                                    self.rust_version
                                ),
                            },
                        },
                        CircleCIStep::Command {
                            run: CircleCIRun::Simple("source $HOME/.cargo/env".to_string()),
                        },
                        CircleCIStep::Command {
                            run: CircleCIRun::Detailed {
                                name: "Check formatting".to_string(),
                                command: "cargo fmt -- --check".to_string(),
                            },
                        },
                    ],
                    environment: None,
                },
            );
            workflow_jobs.push(CircleCIWorkflowJob::Simple("format".to_string()));
        }

        Ok(CircleCIConfig {
            version: "2.1".to_string(),
            orbs: None,
            jobs,
            workflows: BTreeMap::from([(
                "ci".to_string(),
                CircleCIWorkflow {
                    jobs: workflow_jobs,
                },
            )]),
        })
    }
}

impl ToJenkins for RustLibraryPreset {
    fn to_jenkins(&self) -> Result<JenkinsConfig> {
        use crate::platforms::jenkins::models::JenkinsStage;

        let mut stages = vec![];

        // Test stage (always present)
        let mut test_steps = vec![
            format!(
                "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain {}",
                self.rust_version
            ),
            "source $HOME/.cargo/env".to_string(),
            "cargo test --all-features".to_string(),
        ];

        if self.enable_coverage {
            test_steps.push("cargo install cargo-tarpaulin".to_string());
            test_steps.push("cargo tarpaulin --out Xml --all-features".to_string());
        }

        stages.push(JenkinsStage {
            name: "Test".to_string(),
            steps: test_steps,
        });

        // Lint stage (optional)
        if self.enable_linter {
            stages.push(JenkinsStage {
                name: "Lint".to_string(),
                steps: vec![
                    format!(
                        "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain {} --component clippy",
                        self.rust_version
                    ),
                    "source $HOME/.cargo/env".to_string(),
                    "cargo clippy --all-features -- -D warnings".to_string(),
                ],
            });
        }

        // Format check stage (optional)
        if self.enable_format_check {
            stages.push(JenkinsStage {
                name: "Format Check".to_string(),
                steps: vec![
                    format!(
                        "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain {} --component rustfmt",
                        self.rust_version
                    ),
                    "source $HOME/.cargo/env".to_string(),
                    "cargo fmt -- --check".to_string(),
                ],
            });
        }

        // Security scan stage (optional)
        if self.enable_security_scan {
            stages.push(JenkinsStage {
                name: "Security Scan".to_string(),
                steps: vec![
                    "cargo install cargo-audit".to_string(),
                    "cargo audit".to_string(),
                ],
            });
        }

        Ok(JenkinsConfig {
            agent: "any".to_string(),
            environment: vec![],
            stages,
        })
    }
}

impl Detectable for RustLibraryPreset {
    fn matches_github(&self, workflow: &GitHubWorkflow) -> bool {
        // Check for Rust toolchain setup
        let has_rust_toolchain = workflow.jobs.values().any(|job| {
            job.steps.iter().any(|step| {
                step.uses
                    .as_ref()
                    .map(|u| u.contains("rust-toolchain"))
                    .unwrap_or(false)
            })
        });

        // Check for cargo test
        let has_cargo_test = workflow.jobs.values().any(|job| {
            job.steps.iter().any(|step| {
                step.run
                    .as_ref()
                    .map(|r| r.contains("cargo test"))
                    .unwrap_or(false)
            })
        });

        has_rust_toolchain && has_cargo_test
    }

    fn matches_gitlab(&self, _config: &GitLabCI) -> bool {
        // TODO: Implement GitLab detection
        false
    }

    fn matches_circleci(&self, _config: &CircleCIConfig) -> bool {
        // TODO: Implement CircleCI detection
        false
    }

    fn matches_jenkins(&self, _pipeline: &JenkinsConfig) -> bool {
        // TODO: Implement Jenkins detection
        false
    }
}

impl PresetInfo for RustLibraryPreset {
    fn name(&self) -> &str {
        "rust-library"
    }

    fn description(&self) -> &str {
        "CI pipeline for Rust library projects with testing, linting, and optional coverage"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let preset = RustLibraryPreset::builder().build();
        assert_eq!(preset.rust_version, "stable");
        assert!(!preset.enable_coverage);
        assert!(!preset.enable_linter);
        assert!(!preset.enable_security_scan);
        assert!(!preset.enable_format_check);
    }

    #[test]
    fn test_builder_with_options() {
        let preset = RustLibraryPreset::builder()
            .rust_version("1.75.0")
            .coverage(true)
            .linter(true)
            .build();

        assert_eq!(preset.rust_version, "1.75.0");
        assert!(preset.enable_coverage);
        assert!(preset.enable_linter);
    }

    #[test]
    fn test_to_github_basic() {
        let preset = RustLibraryPreset::builder().build();
        let workflow = preset.to_github().unwrap();

        assert_eq!(workflow.name, "CI");
        assert!(workflow.jobs.contains_key("test"));
        assert!(!workflow.jobs.contains_key("lint"));
    }

    #[test]
    fn test_to_github_with_lint() {
        let preset = RustLibraryPreset::builder().linter(true).build();
        let workflow = preset.to_github().unwrap();

        assert!(workflow.jobs.contains_key("test"));
        assert!(workflow.jobs.contains_key("lint"));
    }

    #[test]
    fn test_preset_info() {
        let preset = RustLibraryPreset::builder().build();
        assert_eq!(preset.name(), "rust-library");
        assert!(!preset.description().is_empty());
    }
}
