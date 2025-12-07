use super::models::*;
use crate::error::Result;
use crate::models::*;
use crate::platforms::adapter::PlatformAdapter;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct GitHubAdapter;

impl PlatformAdapter for GitHubAdapter {
    type IR = GitHubWorkflow;

    fn transform(&self, pipeline: &Pipeline) -> Result<GitHubWorkflow> {
        let jobs = pipeline
            .jobs
            .iter()
            .map(|job| {
                let steps = job.steps.iter().map(|step| self.transform_step(step)).collect();

                (
                    job.name.clone(),
                    GitHubJob {
                        runs_on: self.transform_runner(&job.runner),
                        steps,
                        needs: if job.needs.is_empty() {
                            None
                        } else {
                            Some(job.needs.clone())
                        },
                        timeout_minutes: job.timeout_minutes,
                        continue_on_error: if job.continue_on_error {
                            Some(true)
                        } else {
                            None
                        },
                    },
                )
            })
            .collect();

        Ok(GitHubWorkflow {
            name: pipeline.name.clone(),
            on: self.transform_triggers(&pipeline.triggers),
            env: if pipeline.env.is_empty() {
                None
            } else {
                Some(pipeline.env.clone())
            },
            jobs,
        })
    }

    fn serialize(&self, ir: &GitHubWorkflow) -> Result<String> {
        Ok(serde_yaml::to_string(ir)?)
    }

    fn output_path(&self) -> PathBuf {
        PathBuf::from(".github/workflows/ci.yml")
    }
}

impl GitHubAdapter {
    fn transform_step(&self, step: &Step) -> GitHubStep {
        match step {
            Step::Checkout => GitHubStep {
                name: Some("Checkout code".to_string()),
                uses: Some("actions/checkout@v4".to_string()),
                run: None,
                with: None,
                env: None,
            },

            Step::SetupToolchain { language, version } => match language {
                Language::Rust => GitHubStep {
                    name: Some("Setup Rust toolchain".to_string()),
                    uses: Some("dtolnay/rust-toolchain@stable".to_string()),
                    with: Some(HashMap::from([(
                        "toolchain".to_string(),
                        serde_yaml::Value::String(version.clone()),
                    )])),
                    run: None,
                    env: None,
                },
                Language::Python => GitHubStep {
                    name: Some("Setup Python".to_string()),
                    uses: Some("actions/setup-python@v5".to_string()),
                    with: Some(HashMap::from([(
                        "python-version".to_string(),
                        serde_yaml::Value::String(version.clone()),
                    )])),
                    run: None,
                    env: None,
                },
                Language::Go => GitHubStep {
                    name: Some("Setup Go".to_string()),
                    uses: Some("actions/setup-go@v5".to_string()),
                    with: Some(HashMap::from([(
                        "go-version".to_string(),
                        serde_yaml::Value::String(version.clone()),
                    )])),
                    run: None,
                    env: None,
                },
            },

            Step::Cache { paths, key } => GitHubStep {
                name: Some("Cache dependencies".to_string()),
                uses: Some("actions/cache@v4".to_string()),
                with: Some(HashMap::from([
                    (
                        "path".to_string(),
                        serde_yaml::Value::String(paths.join("\n")),
                    ),
                    ("key".to_string(), serde_yaml::Value::String(key.clone())),
                ])),
                run: None,
                env: None,
            },

            Step::RestoreCache { key } => GitHubStep {
                name: Some("Restore cache".to_string()),
                uses: Some("actions/cache/restore@v4".to_string()),
                with: Some(HashMap::from([(
                    "key".to_string(),
                    serde_yaml::Value::String(key.clone()),
                )])),
                run: None,
                env: None,
            },

            Step::RunCommand {
                name,
                command,
                working_dir,
            } => GitHubStep {
                name: Some(name.clone()),
                run: Some(command.clone()),
                uses: None,
                with: working_dir
                    .as_ref()
                    .map(|dir| {
                        HashMap::from([(
                            "working-directory".to_string(),
                            serde_yaml::Value::String(dir.clone()),
                        )])
                    }),
                env: None,
            },

            Step::InstallDependencies { language } => match language {
                Language::Rust => GitHubStep {
                    name: Some("Install dependencies".to_string()),
                    run: Some("cargo fetch".to_string()),
                    uses: None,
                    with: None,
                    env: None,
                },
                Language::Python => GitHubStep {
                    name: Some("Install dependencies".to_string()),
                    run: Some("pip install -r requirements.txt".to_string()),
                    uses: None,
                    with: None,
                    env: None,
                },
                Language::Go => GitHubStep {
                    name: Some("Install dependencies".to_string()),
                    run: Some("go mod download".to_string()),
                    uses: None,
                    with: None,
                    env: None,
                },
            },

            Step::RunTests { language, coverage } => match language {
                Language::Rust => {
                    if *coverage {
                        GitHubStep {
                            name: Some("Run tests with coverage".to_string()),
                            uses: Some("actions-rs/tarpaulin@v0.1".to_string()),
                            with: Some(HashMap::from([(
                                "args".to_string(),
                                serde_yaml::Value::String("--all-features --workspace".to_string()),
                            )])),
                            run: None,
                            env: None,
                        }
                    } else {
                        GitHubStep {
                            name: Some("Run tests".to_string()),
                            run: Some("cargo test --all-features".to_string()),
                            uses: None,
                            with: None,
                            env: None,
                        }
                    }
                }
                Language::Python => GitHubStep {
                    name: Some("Run tests".to_string()),
                    run: Some(if *coverage {
                        "pytest --cov".to_string()
                    } else {
                        "pytest".to_string()
                    }),
                    uses: None,
                    with: None,
                    env: None,
                },
                Language::Go => GitHubStep {
                    name: Some("Run tests".to_string()),
                    run: Some(if *coverage {
                        "go test -v -coverprofile=coverage.out ./...".to_string()
                    } else {
                        "go test -v ./...".to_string()
                    }),
                    uses: None,
                    with: None,
                    env: None,
                },
            },

            Step::RunLinter { language, tool } => match language {
                Language::Rust => GitHubStep {
                    name: Some(format!("Run {}", tool)),
                    run: Some(format!("cargo {}", tool)),
                    uses: None,
                    with: None,
                    env: None,
                },
                Language::Python => GitHubStep {
                    name: Some(format!("Run {}", tool)),
                    run: Some(format!("{} .", tool)),
                    uses: None,
                    with: None,
                    env: None,
                },
                Language::Go => GitHubStep {
                    name: Some(format!("Run {}", tool)),
                    run: Some(tool.clone()),
                    uses: None,
                    with: None,
                    env: None,
                },
            },

            Step::SecurityScan { language, tool } => match language {
                Language::Rust => GitHubStep {
                    name: Some(format!("Security scan ({})", tool)),
                    run: Some(format!("cargo {}", tool)),
                    uses: None,
                    with: None,
                    env: None,
                },
                Language::Python | Language::Go => GitHubStep {
                    name: Some(format!("Security scan ({})", tool)),
                    run: Some(tool.clone()),
                    uses: None,
                    with: None,
                    env: None,
                },
            },

            Step::Build {
                language,
                artifact_paths: _,
            } => match language {
                Language::Rust => GitHubStep {
                    name: Some("Build".to_string()),
                    run: Some("cargo build --release".to_string()),
                    uses: None,
                    with: None,
                    env: None,
                },
                Language::Python => GitHubStep {
                    name: Some("Build".to_string()),
                    run: Some("python -m build".to_string()),
                    uses: None,
                    with: None,
                    env: None,
                },
                Language::Go => GitHubStep {
                    name: Some("Build".to_string()),
                    run: Some("go build -v ./...".to_string()),
                    uses: None,
                    with: None,
                    env: None,
                },
            },

            Step::UploadArtifact { name, paths } => GitHubStep {
                name: Some(format!("Upload {}", name)),
                uses: Some("actions/upload-artifact@v4".to_string()),
                with: Some(HashMap::from([
                    ("name".to_string(), serde_yaml::Value::String(name.clone())),
                    (
                        "path".to_string(),
                        serde_yaml::Value::String(paths.join("\n")),
                    ),
                ])),
                run: None,
                env: None,
            },

            Step::UploadCoverage { provider } => match provider {
                CoverageProvider::Codecov => GitHubStep {
                    name: Some("Upload coverage to Codecov".to_string()),
                    uses: Some("codecov/codecov-action@v4".to_string()),
                    with: None,
                    run: None,
                    env: None,
                },
                CoverageProvider::Coveralls => GitHubStep {
                    name: Some("Upload coverage to Coveralls".to_string()),
                    uses: Some("coverallsapp/github-action@v2".to_string()),
                    with: None,
                    run: None,
                    env: None,
                },
                CoverageProvider::CodeClimate => GitHubStep {
                    name: Some("Upload coverage to Code Climate".to_string()),
                    uses: Some("paambaati/codeclimate-action@v5".to_string()),
                    with: None,
                    run: None,
                    env: None,
                },
            },

            Step::PublishPackage { registry, token_env } => match registry {
                Registry::CratesIo => GitHubStep {
                    name: Some("Publish to crates.io".to_string()),
                    run: Some("cargo publish".to_string()),
                    uses: None,
                    with: None,
                    env: Some(HashMap::from([(
                        "CARGO_REGISTRY_TOKEN".to_string(),
                        format!("${{{{ secrets.{} }}}}", token_env),
                    )])),
                },
                Registry::PyPI => GitHubStep {
                    name: Some("Publish to PyPI".to_string()),
                    run: Some("python -m twine upload dist/*".to_string()),
                    uses: None,
                    with: None,
                    env: Some(HashMap::from([(
                        "TWINE_PASSWORD".to_string(),
                        format!("${{{{ secrets.{} }}}}", token_env),
                    )])),
                },
                Registry::Npm => GitHubStep {
                    name: Some("Publish to npm".to_string()),
                    run: Some("npm publish".to_string()),
                    uses: None,
                    with: None,
                    env: Some(HashMap::from([(
                        "NODE_AUTH_TOKEN".to_string(),
                        format!("${{{{ secrets.{} }}}}", token_env),
                    )])),
                },
            },

            Step::CreateRelease {
                tag_pattern: _,
                artifacts,
            } => GitHubStep {
                name: Some("Create Release".to_string()),
                uses: Some("softprops/action-gh-release@v1".to_string()),
                with: Some(HashMap::from([(
                    "files".to_string(),
                    serde_yaml::Value::String(artifacts.join("\n")),
                )])),
                run: None,
                env: None,
            },
        }
    }

    fn transform_runner(&self, runner: &Runner) -> String {
        match runner {
            Runner::UbuntuLatest => "ubuntu-latest".to_string(),
            Runner::Ubuntu2204 => "ubuntu-22.04".to_string(),
            Runner::Ubuntu2004 => "ubuntu-20.04".to_string(),
            Runner::MacOSLatest => "macos-latest".to_string(),
            Runner::MacOS13 => "macos-13".to_string(),
            Runner::WindowsLatest => "windows-latest".to_string(),
            Runner::Windows2022 => "windows-2022".to_string(),
            Runner::Custom(s) => s.clone(),
        }
    }

    fn transform_triggers(&self, triggers: &[Trigger]) -> GitHubTriggers {
        let mut detailed = HashMap::new();

        for trigger in triggers {
            match trigger {
                Trigger::Push { branches } => {
                    detailed.insert(
                        "push".to_string(),
                        GitHubTriggerConfig {
                            branches: Some(branches.clone()),
                            tags: None,
                        },
                    );
                }
                Trigger::PullRequest { branches } => {
                    detailed.insert(
                        "pull_request".to_string(),
                        GitHubTriggerConfig {
                            branches: Some(branches.clone()),
                            tags: None,
                        },
                    );
                }
                Trigger::Tag { pattern } => {
                    detailed
                        .entry("push".to_string())
                        .or_insert_with(|| GitHubTriggerConfig {
                            branches: None,
                            tags: Some(vec![pattern.clone()]),
                        })
                        .tags
                        .get_or_insert_with(Vec::new)
                        .push(pattern.clone());
                }
                Trigger::Schedule { cron } => {
                    // Schedule triggers are more complex, simplified for now
                    detailed.insert(
                        "schedule".to_string(),
                        GitHubTriggerConfig {
                            branches: None,
                            tags: Some(vec![cron.clone()]),
                        },
                    );
                }
                Trigger::Manual => {
                    detailed.insert(
                        "workflow_dispatch".to_string(),
                        GitHubTriggerConfig {
                            branches: None,
                            tags: None,
                        },
                    );
                }
            }
        }

        GitHubTriggers::Detailed(detailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_simple_pipeline() {
        let adapter = GitHubAdapter;
        let pipeline = Pipeline {
            name: "Test".to_string(),
            triggers: vec![Trigger::Push {
                branches: vec!["main".to_string()],
            }],
            jobs: vec![Job {
                name: "test".to_string(),
                runner: Runner::UbuntuLatest,
                steps: vec![Step::Checkout],
                needs: vec![],
                timeout_minutes: None,
                continue_on_error: false,
            }],
            env: HashMap::new(),
            cache: CacheConfig::default(),
        };

        let workflow = adapter.transform(&pipeline).unwrap();
        assert_eq!(workflow.name, "Test");
        assert_eq!(workflow.jobs.len(), 1);
        assert!(workflow.jobs.contains_key("test"));
    }

    #[test]
    fn test_serialize_workflow() {
        let adapter = GitHubAdapter;
        let pipeline = Pipeline {
            name: "CI".to_string(),
            triggers: vec![Trigger::Push {
                branches: vec!["main".to_string()],
            }],
            jobs: vec![Job {
                name: "test".to_string(),
                runner: Runner::UbuntuLatest,
                steps: vec![Step::Checkout],
                needs: vec![],
                timeout_minutes: None,
                continue_on_error: false,
            }],
            env: HashMap::new(),
            cache: CacheConfig::default(),
        };

        let yaml = adapter.generate(&pipeline).unwrap();
        assert!(yaml.contains("name: CI"));
        assert!(yaml.contains("ubuntu-latest"));
        assert!(yaml.contains("actions/checkout"));
    }
}
