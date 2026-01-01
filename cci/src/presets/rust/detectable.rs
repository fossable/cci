use crate::platforms::circleci::models::CircleCIConfig;
use crate::platforms::github::models::GitHubWorkflow;
use crate::platforms::gitlab::models::GitLabCI;
use crate::platforms::jenkins::models::JenkinsConfig;
use crate::traits::Detectable;

use super::RustPreset;

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
