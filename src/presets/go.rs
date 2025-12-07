use crate::models::*;
use std::collections::HashMap;

pub fn go_app_preset(version: &str) -> Pipeline {
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
                    language: Language::Go,
                    version: version.to_string(),
                },
                Step::InstallDependencies {
                    language: Language::Go,
                },
                Step::RunTests {
                    language: Language::Go,
                    coverage: true,
                },
                Step::RunLinter {
                    language: Language::Go,
                    tool: "golangci-lint run".to_string(),
                },
                Step::SecurityScan {
                    language: Language::Go,
                    tool: "gosec ./...".to_string(),
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
            paths: vec!["~/go/pkg/mod".to_string()],
            key_prefix: "go".to_string(),
        },
    }
}
