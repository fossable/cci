use crate::error::Result;
use crate::platforms::circleci::models::CircleCIConfig;
use crate::platforms::github::models::{
    GitHubJob, GitHubStep, GitHubTriggerConfig, GitHubTriggers, GitHubWorkflow,
};
use crate::platforms::gitlab::models::GitLabCI;
use crate::platforms::jenkins::models::JenkinsConfig;
use crate::traits::{Detectable, PresetInfo, ToCircleCI, ToGitea, ToGitHub, ToGitLab, ToJenkins};
use cci_macros::Preset;
use std::collections::BTreeMap;

/// Unified preset for Rust projects (binaries, libraries, and workspaces)
#[derive(Debug, Clone, Preset)]
#[preset(
    id = "rust",
    name = "Rust",
    description = "CI pipeline for Rust projects (binaries, libraries, and workspaces)",
    matches = "RustBinary | RustLibrary | RustWorkspace"
)]
pub struct RustPreset {
    #[preset_field(
        ron_field = "version",
        default = "\"stable\".to_string()",
        hidden = true
    )]
    rust_version: String,

    #[preset_field(
        feature = "testing",
        feature_display = "Testing",
        feature_description = "Test coverage reporting",
        display = "Code Coverage",
        description = "Enable code coverage reporting with tarpaulin",
        default = "true"
    )]
    enable_coverage: bool,

    #[preset_field(
        feature = "linting",
        feature_display = "Linting",
        feature_description = "Code quality checks",
        display = "Clippy Linter",
        description = "Run Clippy linter for code quality",
        default = "true"
    )]
    enable_linter: bool,

    #[preset_field(
        id = "enable_security",
        feature = "security",
        feature_display = "Security",
        feature_description = "Security vulnerability scanning",
        display = "Security Scan",
        description = "Run cargo-audit for dependency vulnerabilities",
        default = "true"
    )]
    enable_security_scan: bool,

    #[preset_field(
        id = "enable_formatter",
        feature = "formatting",
        feature_display = "Formatting",
        feature_description = "Code formatting checks",
        display = "Rustfmt Check",
        description = "Check code formatting with rustfmt",
        default = "true"
    )]
    enable_format_check: bool,

    #[preset_field(
        feature = "building",
        feature_display = "Building",
        feature_description = "Release binary builds",
        display = "Build Release",
        description = "Build optimized release binary in CI",
        default = "true"
    )]
    build_release: bool,
}

impl Default for RustPreset {
    fn default() -> Self {
        Self {
            rust_version: "stable".to_string(),
            enable_coverage: false,
            enable_linter: false,
            enable_security_scan: false,
            enable_format_check: false,
            build_release: false,
        }
    }
}

impl ToGitHub for RustPreset {
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
                "rust/format".to_string(),
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
                "rust/security".to_string(),
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

impl ToGitea for RustPreset {
    fn to_gitea(&self) -> Result<crate::platforms::gitea::models::GiteaWorkflow> {
        // Gitea Actions uses the same workflow format as GitHub Actions
        self.to_github()
    }
}

impl ToGitLab for RustPreset {
    fn to_gitlab(&self) -> Result<GitLabCI> {
        use crate::platforms::gitlab::models::{GitLabArtifacts, GitLabCache, GitLabJob, GitLabOnly};

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

        if self.build_release {
            stages.push("build".to_string());
        }

        jobs.insert(
            "rust/test".to_string(),
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
                    Some(GitLabArtifacts {
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

        // Build job (optional)
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

        // Lint job (optional)
        if self.enable_linter {
            if !stages.contains(&"lint".to_string()) {
                stages.push("lint".to_string());
            }
            jobs.insert(
                "rust/lint".to_string(),
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
                "rust/format".to_string(),
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
                "rust/security".to_string(),
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

impl ToCircleCI for RustPreset {
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

        if self.build_release {
            test_steps.push(CircleCIStep::Command {
                run: CircleCIRun::Detailed {
                    name: "Build release".to_string(),
                    command: "cargo build --release".to_string(),
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
            "rust/test".to_string(),
            CircleCIJob {
                docker: vec![CircleCIDocker {
                    image: "rust:latest".to_string(),
                }],
                steps: test_steps,
                environment: None,
            },
        );

        workflow_jobs.push(CircleCIWorkflowJob::Simple("rust/test".to_string()));

        // Lint job (optional)
        if self.enable_linter {
            jobs.insert(
                "rust/lint".to_string(),
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
            workflow_jobs.push(CircleCIWorkflowJob::Simple("rust/lint".to_string()));
        }

        // Format check job (optional)
        if self.enable_format_check {
            jobs.insert(
                "rust/format".to_string(),
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
            workflow_jobs.push(CircleCIWorkflowJob::Simple("rust/format".to_string()));
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

impl ToJenkins for RustPreset {
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

        if self.build_release {
            test_steps.push("cargo build --release".to_string());
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

impl Detectable for RustPreset {
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

impl PresetInfo for RustPreset {
    fn name(&self) -> &str {
        "rust"
    }

    fn description(&self) -> &str {
        "CI pipeline for Rust projects (binaries, libraries, and workspaces)"
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let preset = RustPreset::default();
        assert_eq!(preset.rust_version, "stable");
        // Defaults changed - all features are now enabled by default
        assert!(preset.enable_coverage);
        assert!(preset.enable_linter);
        assert!(preset.enable_security_scan);
        assert!(preset.enable_format_check);
        assert!(preset.build_release);
    }

    #[test]
    fn test_with_options() {
        let preset = RustPreset {
            rust_version: "1.75.0".to_string(),
            enable_coverage: true,
            enable_linter: true,
            enable_security_scan: false,
            enable_format_check: false,
            build_release: true,
        };

        assert_eq!(preset.rust_version, "1.75.0");
        assert!(preset.enable_coverage);
        assert!(preset.enable_linter);
        assert!(preset.build_release);
    }

    #[test]
    fn test_to_github_basic() {
        let preset = RustPreset {
            rust_version: "stable".to_string(),
            enable_coverage: false,
            enable_linter: false,
            enable_security_scan: false,
            enable_format_check: false,
            build_release: false,
        };
        let workflow = preset.to_github().unwrap();

        assert_eq!(workflow.name, "CI");
        assert!(workflow.jobs.contains_key("rust/test"));
        assert!(!workflow.jobs.contains_key("rust/lint"));
    }

    #[test]
    fn test_to_github_with_lint() {
        let preset = RustPreset {
            rust_version: "stable".to_string(),
            enable_coverage: false,
            enable_linter: true,
            enable_security_scan: false,
            enable_format_check: false,
            build_release: false,
        };
        let workflow = preset.to_github().unwrap();

        assert!(workflow.jobs.contains_key("rust/test"));
        assert!(workflow.jobs.contains_key("rust/lint"));
    }

    #[test]
    fn test_preset_info() {
        let preset = RustPreset::default();
        assert_eq!(preset.name(), "rust");
        assert!(!preset.description().is_empty());
    }
}
