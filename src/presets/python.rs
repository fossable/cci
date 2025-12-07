use crate::models::*;
use std::collections::HashMap;

pub fn python_app_preset(version: &str) -> Pipeline {
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
        jobs: vec![Job {
            name: "test".to_string(),
            runner: Runner::UbuntuLatest,
            steps: vec![
                Step::Checkout,
                Step::SetupToolchain {
                    language: Language::Python,
                    version: version.to_string(),
                },
                Step::InstallDependencies {
                    language: Language::Python,
                },
                Step::RunTests {
                    language: Language::Python,
                    coverage: true,
                },
                Step::RunLinter {
                    language: Language::Python,
                    tool: "mypy".to_string(),
                },
                Step::RunLinter {
                    language: Language::Python,
                    tool: "black --check".to_string(),
                },
                Step::SecurityScan {
                    language: Language::Python,
                    tool: "safety check".to_string(),
                },
                Step::UploadCoverage {
                    provider: CoverageProvider::Codecov,
                },
            ],
            needs: vec![],
            timeout_minutes: Some(30),
            continue_on_error: false,
        }],
        env: HashMap::new(),
        cache: CacheConfig {
            enabled: true,
            paths: vec!["~/.cache/pip".to_string()],
            key_prefix: "pip".to_string(),
        },
    }
}
