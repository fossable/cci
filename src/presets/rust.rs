use crate::models::*;
use std::collections::HashMap;

pub fn rust_library_preset(version: &str) -> Pipeline {
    Pipeline {
        name: "CI".to_string(),
        triggers: vec![
            Trigger::Push {
                branches: vec!["main".to_string()],
            },
            Trigger::PullRequest {
                branches: vec!["main".to_string()],
            },
        ],
        jobs: vec![
            Job {
                name: "test".to_string(),
                runner: Runner::UbuntuLatest,
                steps: vec![
                    Step::Checkout,
                    Step::SetupToolchain {
                        language: Language::Rust,
                        version: version.to_string(),
                    },
                    Step::Cache {
                        paths: vec!["~/.cargo".to_string(), "target".to_string()],
                        key: "cargo-${{ hashFiles('**/Cargo.lock') }}".to_string(),
                    },
                    Step::RunTests {
                        language: Language::Rust,
                        coverage: true,
                    },
                    Step::RunLinter {
                        language: Language::Rust,
                        tool: "clippy".to_string(),
                    },
                    Step::RunCommand {
                        name: "Check formatting".to_string(),
                        command: "cargo fmt -- --check".to_string(),
                        working_dir: None,
                    },
                    Step::SecurityScan {
                        language: Language::Rust,
                        tool: "cargo-audit".to_string(),
                    },
                    Step::UploadCoverage {
                        provider: CoverageProvider::Codecov,
                    },
                ],
                needs: vec![],
                timeout_minutes: Some(30),
                continue_on_error: false,
            },
        ],
        env: HashMap::from([("RUST_BACKTRACE".to_string(), "1".to_string())]),
        cache: CacheConfig {
            enabled: true,
            paths: vec!["~/.cargo".to_string(), "target".to_string()],
            key_prefix: "cargo".to_string(),
        },
    }
}

pub fn rust_binary_preset(version: &str) -> Pipeline {
    Pipeline {
        name: "CI".to_string(),
        triggers: vec![
            Trigger::Push {
                branches: vec!["main".to_string()],
            },
            Trigger::PullRequest {
                branches: vec!["main".to_string()],
            },
            Trigger::Tag {
                pattern: "v*".to_string(),
            },
        ],
        jobs: vec![
            Job {
                name: "test".to_string(),
                runner: Runner::UbuntuLatest,
                steps: vec![
                    Step::Checkout,
                    Step::SetupToolchain {
                        language: Language::Rust,
                        version: version.to_string(),
                    },
                    Step::Cache {
                        paths: vec!["~/.cargo".to_string(), "target".to_string()],
                        key: "cargo-${{ hashFiles('**/Cargo.lock') }}".to_string(),
                    },
                    Step::RunTests {
                        language: Language::Rust,
                        coverage: true,
                    },
                    Step::RunLinter {
                        language: Language::Rust,
                        tool: "clippy".to_string(),
                    },
                    Step::SecurityScan {
                        language: Language::Rust,
                        tool: "cargo-audit".to_string(),
                    },
                    Step::UploadCoverage {
                        provider: CoverageProvider::Codecov,
                    },
                ],
                needs: vec![],
                timeout_minutes: Some(30),
                continue_on_error: false,
            },
            Job {
                name: "build".to_string(),
                runner: Runner::UbuntuLatest,
                steps: vec![
                    Step::Checkout,
                    Step::SetupToolchain {
                        language: Language::Rust,
                        version: version.to_string(),
                    },
                    Step::Cache {
                        paths: vec!["~/.cargo".to_string(), "target".to_string()],
                        key: "cargo-${{ hashFiles('**/Cargo.lock') }}".to_string(),
                    },
                    Step::Build {
                        language: Language::Rust,
                        artifact_paths: vec!["target/release/*".to_string()],
                    },
                    Step::UploadArtifact {
                        name: "binaries".to_string(),
                        paths: vec!["target/release/*".to_string()],
                    },
                ],
                needs: vec!["test".to_string()],
                timeout_minutes: Some(30),
                continue_on_error: false,
            },
            Job {
                name: "release".to_string(),
                runner: Runner::UbuntuLatest,
                steps: vec![
                    Step::Checkout,
                    Step::SetupToolchain {
                        language: Language::Rust,
                        version: version.to_string(),
                    },
                    Step::CreateRelease {
                        tag_pattern: "v*".to_string(),
                        artifacts: vec!["target/release/*".to_string()],
                    },
                ],
                needs: vec!["build".to_string()],
                timeout_minutes: Some(15),
                continue_on_error: false,
            },
        ],
        env: HashMap::from([("RUST_BACKTRACE".to_string(), "1".to_string())]),
        cache: CacheConfig {
            enabled: true,
            paths: vec!["~/.cargo".to_string(), "target".to_string()],
            key_prefix: "cargo".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_library_preset() {
        let pipeline = rust_library_preset("1.75");
        assert_eq!(pipeline.name, "CI");
        assert_eq!(pipeline.triggers.len(), 2);
        assert_eq!(pipeline.jobs.len(), 1);
        assert_eq!(pipeline.jobs[0].name, "test");
        assert_eq!(pipeline.jobs[0].steps.len(), 8);
    }

    #[test]
    fn test_rust_binary_preset() {
        let pipeline = rust_binary_preset("1.75");
        assert_eq!(pipeline.name, "CI");
        assert_eq!(pipeline.triggers.len(), 3);
        assert_eq!(pipeline.jobs.len(), 3);
        assert_eq!(pipeline.jobs[0].name, "test");
        assert_eq!(pipeline.jobs[1].name, "build");
        assert_eq!(pipeline.jobs[2].name, "release");
    }
}
