use super::models::*;
use crate::error::Result;
use crate::models::*;
use crate::platforms::adapter::PlatformAdapter;
use std::path::PathBuf;

pub struct GitLabAdapter;

impl PlatformAdapter for GitLabAdapter {
    type IR = GitLabCI;

    fn transform(&self, pipeline: &Pipeline) -> Result<GitLabCI> {
        // Extract unique stages
        let stages: Vec<String> = pipeline
            .jobs
            .iter()
            .map(|job| self.job_to_stage(&job.name))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let jobs = pipeline
            .jobs
            .iter()
            .map(|job| {
                let mut script = Vec::new();
                for step in &job.steps {
                    script.extend(self.step_to_script(step));
                }

                (
                    job.name.clone(),
                    GitLabJob {
                        stage: self.job_to_stage(&job.name),
                        image: Some(self.runner_to_image(&job.runner)),
                        script,
                        before_script: None,
                        after_script: None,
                        needs: if job.needs.is_empty() {
                            None
                        } else {
                            Some(job.needs.clone())
                        },
                        cache: if pipeline.cache.enabled {
                            Some(GitLabCache {
                                key: pipeline.cache.key_prefix.clone(),
                                paths: pipeline.cache.paths.clone(),
                            })
                        } else {
                            None
                        },
                        artifacts: None,
                        only: None,
                        timeout: job.timeout_minutes.map(|m| format!("{}m", m)),
                    },
                )
            })
            .collect();

        Ok(GitLabCI {
            stages: Some(stages),
            variables: if pipeline.env.is_empty() {
                None
            } else {
                Some(pipeline.env.clone())
            },
            cache: if pipeline.cache.enabled {
                Some(GitLabCache {
                    key: pipeline.cache.key_prefix.clone(),
                    paths: pipeline.cache.paths.clone(),
                })
            } else {
                None
            },
            jobs,
        })
    }

    fn serialize(&self, ir: &GitLabCI) -> Result<String> {
        Ok(serde_yaml::to_string(ir)?)
    }

    fn output_path(&self) -> PathBuf {
        PathBuf::from(".gitlab-ci.yml")
    }
}

impl GitLabAdapter {
    fn job_to_stage(&self, job_name: &str) -> String {
        match job_name {
            "test" => "test".to_string(),
            "build" => "build".to_string(),
            "release" => "deploy".to_string(),
            _ => "test".to_string(),
        }
    }

    fn runner_to_image(&self, runner: &Runner) -> String {
        match runner {
            Runner::UbuntuLatest | Runner::Ubuntu2204 => "ubuntu:22.04".to_string(),
            Runner::Ubuntu2004 => "ubuntu:20.04".to_string(),
            _ => "ubuntu:latest".to_string(),
        }
    }

    fn step_to_script(&self, step: &Step) -> Vec<String> {
        match step {
            Step::Checkout => vec![],  // GitLab does this automatically

            Step::SetupToolchain { language, version } => match language {
                Language::Rust => vec![
                    format!("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain {}", version),
                    "source $HOME/.cargo/env".to_string(),
                ],
                Language::Python => vec![format!("apt-get update && apt-get install -y python{}", version)],
                Language::Go => vec![format!("wget -qO- https://go.dev/dl/go{}.linux-amd64.tar.gz | tar -C /usr/local -xz", version)],
            },

            Step::Cache { .. } => vec![],  // Handled at job level

            Step::RestoreCache { .. } => vec![],  // Handled at job level

            Step::RunCommand { command, .. } => vec![command.clone()],

            Step::InstallDependencies { language } => match language {
                Language::Rust => vec!["cargo fetch".to_string()],
                Language::Python => vec!["pip install -r requirements.txt".to_string()],
                Language::Go => vec!["go mod download".to_string()],
            },

            Step::RunTests { language, coverage } => match language {
                Language::Rust => {
                    if *coverage {
                        vec![
                            "cargo install cargo-tarpaulin".to_string(),
                            "cargo tarpaulin --all-features --workspace --out Xml".to_string(),
                        ]
                    } else {
                        vec!["cargo test --all-features".to_string()]
                    }
                }
                Language::Python => {
                    if *coverage {
                        vec!["pytest --cov".to_string()]
                    } else {
                        vec!["pytest".to_string()]
                    }
                }
                Language::Go => {
                    if *coverage {
                        vec!["go test -v -coverprofile=coverage.out ./...".to_string()]
                    } else {
                        vec!["go test -v ./...".to_string()]
                    }
                }
            },

            Step::RunLinter { language, tool } => match language {
                Language::Rust => vec![format!("cargo {}", tool)],
                Language::Python | Language::Go => vec![tool.clone()],
            },

            Step::SecurityScan { language, tool } => match language {
                Language::Rust => vec![
                    "cargo install cargo-audit".to_string(),
                    format!("cargo {}", tool),
                ],
                Language::Python | Language::Go => vec![tool.clone()],
            },

            Step::Build { language, .. } => match language {
                Language::Rust => vec!["cargo build --release".to_string()],
                Language::Python => vec!["python -m build".to_string()],
                Language::Go => vec!["go build -v ./...".to_string()],
            },

            Step::UploadArtifact { .. } => vec![],  // Handled at job level

            Step::UploadCoverage { .. } => vec![
                "bash <(curl -s https://codecov.io/bash)".to_string(),
            ],

            Step::PublishPackage { registry, .. } => match registry {
                Registry::CratesIo => vec!["cargo publish".to_string()],
                Registry::PyPI => vec!["python -m twine upload dist/*".to_string()],
                Registry::Npm => vec!["npm publish".to_string()],
            },

            Step::CreateRelease { .. } => vec![
                "echo 'Release creation handled by GitLab'".to_string(),
            ],
        }
    }
}
