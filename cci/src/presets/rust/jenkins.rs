use crate::error::Result;
use crate::platforms::jenkins::models::JenkinsConfig;
use crate::traits::ToJenkins;

use super::RustPreset;

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
