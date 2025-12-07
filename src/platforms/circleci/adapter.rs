use super::models::*;
use crate::error::Result;
use crate::models::*;
use crate::platforms::adapter::PlatformAdapter;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct CircleCIAdapter;

impl PlatformAdapter for CircleCIAdapter {
    type IR = CircleCIConfig;

    fn transform(&self, pipeline: &Pipeline) -> Result<CircleCIConfig> {
        let jobs = pipeline
            .jobs
            .iter()
            .map(|job| {
                let mut steps = vec![CircleCIStep::Simple("checkout".to_string())];

                for step in &job.steps {
                    steps.extend(self.transform_step(step));
                }

                (
                    job.name.clone(),
                    CircleCIJob {
                        docker: vec![CircleCIDocker {
                            image: self.runner_to_image(&job.runner),
                        }],
                        steps,
                        environment: if pipeline.env.is_empty() {
                            None
                        } else {
                            Some(pipeline.env.clone())
                        },
                    },
                )
            })
            .collect();

        // Build workflow
        let workflow_jobs: Vec<CircleCIWorkflowJob> = pipeline
            .jobs
            .iter()
            .map(|job| {
                if job.needs.is_empty() {
                    CircleCIWorkflowJob::Simple(job.name.clone())
                } else {
                    CircleCIWorkflowJob::WithRequires {
                        job: HashMap::from([(
                            job.name.clone(),
                            CircleCIJobRequires {
                                requires: job.needs.clone(),
                            },
                        )]),
                    }
                }
            })
            .collect();

        Ok(CircleCIConfig {
            version: "2.1".to_string(),
            orbs: None,
            jobs,
            workflows: HashMap::from([(
                "main".to_string(),
                CircleCIWorkflow {
                    jobs: workflow_jobs,
                },
            )]),
        })
    }

    fn serialize(&self, ir: &CircleCIConfig) -> Result<String> {
        Ok(serde_yaml::to_string(ir)?)
    }

    fn output_path(&self) -> PathBuf {
        PathBuf::from(".circleci/config.yml")
    }
}

impl CircleCIAdapter {
    fn runner_to_image(&self, runner: &Runner) -> String {
        match runner {
            Runner::UbuntuLatest | Runner::Ubuntu2204 => "cimg/base:2022.04".to_string(),
            Runner::Ubuntu2004 => "cimg/base:2020.04".to_string(),
            _ => "cimg/base:current".to_string(),
        }
    }

    fn transform_step(&self, step: &Step) -> Vec<CircleCIStep> {
        match step {
            Step::Checkout => vec![],  // Already added

            Step::SetupToolchain { language, version } => match language {
                Language::Rust => vec![CircleCIStep::Command {
                    run: CircleCIRun::Detailed {
                        name: "Setup Rust".to_string(),
                        command: format!("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain {}", version),
                    },
                }],
                Language::Python => vec![CircleCIStep::Simple(format!("python/install:{}", version))],
                Language::Go => vec![CircleCIStep::Simple(format!("go/install:{}", version))],
            },

            Step::Cache { paths, key } => vec![
                CircleCIStep::Cache {
                    restore_cache: CircleCICache {
                        keys: vec![key.clone()],
                    },
                },
                CircleCIStep::SaveCache {
                    save_cache: CircleCICacheSave {
                        key: key.clone(),
                        paths: paths.clone(),
                    },
                },
            ],

            Step::RestoreCache { key } => vec![CircleCIStep::Cache {
                restore_cache: CircleCICache {
                    keys: vec![key.clone()],
                },
            }],

            Step::RunCommand { name, command, .. } => vec![CircleCIStep::Command {
                run: CircleCIRun::Detailed {
                    name: name.clone(),
                    command: command.clone(),
                },
            }],

            Step::InstallDependencies { language } => {
                let cmd = match language {
                    Language::Rust => "cargo fetch",
                    Language::Python => "pip install -r requirements.txt",
                    Language::Go => "go mod download",
                };
                vec![CircleCIStep::Command {
                    run: CircleCIRun::Simple(cmd.to_string()),
                }]
            }

            Step::RunTests { language, coverage } => {
                let cmd = match language {
                    Language::Rust => {
                        if *coverage {
                            "cargo tarpaulin --all-features --workspace"
                        } else {
                            "cargo test --all-features"
                        }
                    }
                    Language::Python => {
                        if *coverage {
                            "pytest --cov"
                        } else {
                            "pytest"
                        }
                    }
                    Language::Go => {
                        if *coverage {
                            "go test -v -coverprofile=coverage.out ./..."
                        } else {
                            "go test -v ./..."
                        }
                    }
                };
                vec![CircleCIStep::Command {
                    run: CircleCIRun::Simple(cmd.to_string()),
                }]
            }

            Step::RunLinter { language, tool } => {
                let cmd = match language {
                    Language::Rust => format!("cargo {}", tool),
                    Language::Python | Language::Go => tool.clone(),
                };
                vec![CircleCIStep::Command {
                    run: CircleCIRun::Simple(cmd),
                }]
            }

            Step::SecurityScan { language, tool } => {
                let cmd = match language {
                    Language::Rust => format!("cargo {}", tool),
                    Language::Python | Language::Go => tool.clone(),
                };
                vec![CircleCIStep::Command {
                    run: CircleCIRun::Simple(cmd),
                }]
            }

            Step::Build { language, .. } => {
                let cmd = match language {
                    Language::Rust => "cargo build --release",
                    Language::Python => "python -m build",
                    Language::Go => "go build -v ./...",
                };
                vec![CircleCIStep::Command {
                    run: CircleCIRun::Simple(cmd.to_string()),
                }]
            }

            Step::UploadArtifact { .. } => vec![],  // CircleCI handles this differently

            Step::UploadCoverage { .. } => vec![CircleCIStep::Command {
                run: CircleCIRun::Simple("bash <(curl -s https://codecov.io/bash)".to_string()),
            }],

            Step::PublishPackage { registry, .. } => {
                let cmd = match registry {
                    Registry::CratesIo => "cargo publish",
                    Registry::PyPI => "python -m twine upload dist/*",
                    Registry::Npm => "npm publish",
                };
                vec![CircleCIStep::Command {
                    run: CircleCIRun::Simple(cmd.to_string()),
                }]
            }

            Step::CreateRelease { .. } => vec![],
        }
    }
}
