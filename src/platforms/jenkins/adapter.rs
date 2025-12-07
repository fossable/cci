use super::models::*;
use crate::error::Result;
use crate::models::*;
use crate::platforms::adapter::PlatformAdapter;
use std::path::PathBuf;

pub struct JenkinsAdapter;

impl PlatformAdapter for JenkinsAdapter {
    type IR = JenkinsConfig;

    fn transform(&self, pipeline: &Pipeline) -> Result<JenkinsConfig> {
        let stages = pipeline
            .jobs
            .iter()
            .map(|job| {
                let mut steps = Vec::new();
                for step in &job.steps {
                    steps.extend(self.step_to_command(step));
                }
                JenkinsStage {
                    name: job.name.clone(),
                    steps,
                }
            })
            .collect();

        Ok(JenkinsConfig {
            agent: "any".to_string(),
            environment: pipeline.env.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            stages,
        })
    }

    fn serialize(&self, ir: &JenkinsConfig) -> Result<String> {
        let mut output = String::new();
        output.push_str("pipeline {\n");
        output.push_str(&format!("    agent {}\n", ir.agent));

        if !ir.environment.is_empty() {
            output.push_str("    environment {\n");
            for (key, value) in &ir.environment {
                output.push_str(&format!("        {} = '{}'\n", key, value));
            }
            output.push_str("    }\n");
        }

        output.push_str("    stages {\n");
        for stage in &ir.stages {
            output.push_str(&format!("        stage('{}') {{\n", stage.name));
            output.push_str("            steps {\n");
            for step in &stage.steps {
                output.push_str(&format!("                {}\n", step));
            }
            output.push_str("            }\n");
            output.push_str("        }\n");
        }
        output.push_str("    }\n");
        output.push_str("}\n");

        Ok(output)
    }

    fn output_path(&self) -> PathBuf {
        PathBuf::from("Jenkinsfile")
    }
}

impl JenkinsAdapter {
    fn step_to_command(&self, step: &Step) -> Vec<String> {
        match step {
            Step::Checkout => vec!["checkout scm".to_string()],

            Step::SetupToolchain { language, version } => match language {
                Language::Rust => vec![format!("sh 'curl --proto \\'=https\\' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain {}'", version)],
                Language::Python => vec![format!("sh 'python{} --version'", version)],
                Language::Go => vec![format!("sh 'go version | grep {}'", version)],
            },

            Step::Cache { .. } => vec![],  // Jenkins uses different caching

            Step::RestoreCache { .. } => vec![],

            Step::RunCommand { command, .. } => vec![format!("sh '{}'", command)],

            Step::InstallDependencies { language } => {
                let cmd = match language {
                    Language::Rust => "cargo fetch",
                    Language::Python => "pip install -r requirements.txt",
                    Language::Go => "go mod download",
                };
                vec![format!("sh '{}'", cmd)]
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
                vec![format!("sh '{}'", cmd)]
            }

            Step::RunLinter { language, tool } => {
                let cmd = match language {
                    Language::Rust => format!("cargo {}", tool),
                    Language::Python | Language::Go => tool.clone(),
                };
                vec![format!("sh '{}'", cmd)]
            }

            Step::SecurityScan { language, tool } => {
                let cmd = match language {
                    Language::Rust => format!("cargo {}", tool),
                    Language::Python | Language::Go => tool.clone(),
                };
                vec![format!("sh '{}'", cmd)]
            }

            Step::Build { language, .. } => {
                let cmd = match language {
                    Language::Rust => "cargo build --release",
                    Language::Python => "python -m build",
                    Language::Go => "go build -v ./...",
                };
                vec![format!("sh '{}'", cmd)]
            }

            Step::UploadArtifact { name: _, paths } => vec![format!(
                "archiveArtifacts artifacts: '{}', fingerprint: true",
                paths.join(",")
            )],

            Step::UploadCoverage { .. } => vec!["sh 'bash <(curl -s https://codecov.io/bash)'".to_string()],

            Step::PublishPackage { registry, .. } => {
                let cmd = match registry {
                    Registry::CratesIo => "cargo publish",
                    Registry::PyPI => "python -m twine upload dist/*",
                    Registry::Npm => "npm publish",
                };
                vec![format!("sh '{}'", cmd)]
            }

            Step::CreateRelease { .. } => vec![],
        }
    }
}
