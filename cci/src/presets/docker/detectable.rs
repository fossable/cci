use crate::platforms::circleci::models::CircleCIConfig;
use crate::platforms::github::models::GitHubWorkflow;
use crate::platforms::gitlab::models::GitLabCI;
use crate::platforms::jenkins::models::JenkinsConfig;
use crate::traits::Detectable;

use super::DockerPreset;

impl Detectable for DockerPreset {
    fn matches_github(&self, workflow: &GitHubWorkflow) -> bool {
        // Check for Docker build actions
        let has_docker_build = workflow.jobs.values().any(|job| {
            job.steps.iter().any(|step| {
                step.uses
                    .as_ref()
                    .map(|u| u.contains("docker/build-push-action"))
                    .unwrap_or(false)
            })
        });

        // Check for docker build commands
        let has_docker_commands = workflow.jobs.values().any(|job| {
            job.steps.iter().any(|step| {
                step.run
                    .as_ref()
                    .map(|r| r.contains("docker build"))
                    .unwrap_or(false)
            })
        });

        has_docker_build || has_docker_commands
    }

    fn matches_gitea(&self, workflow: &crate::platforms::gitea::models::GiteaWorkflow) -> bool {
        // Gitea Actions uses the same workflow format as GitHub Actions
        self.matches_github(workflow)
    }

    fn matches_gitlab(&self, config: &GitLabCI) -> bool {
        config.jobs.values().any(|job| {
            job.script
                .iter()
                .any(|cmd| cmd.contains("docker build") || cmd.contains("docker push"))
        })
    }

    fn matches_circleci(&self, config: &CircleCIConfig) -> bool {
        config.jobs.values().any(|job| {
            job.steps.iter().any(|step| {
                if let crate::platforms::circleci::models::CircleCIStep::Command { run } = step {
                    match run {
                        crate::platforms::circleci::models::CircleCIRun::Simple(cmd) => {
                            cmd.contains("docker build")
                        }
                        crate::platforms::circleci::models::CircleCIRun::Detailed {
                            command, ..
                        } => command.contains("docker build"),
                    }
                } else {
                    false
                }
            })
        })
    }

    fn matches_jenkins(&self, config: &JenkinsConfig) -> bool {
        config.stages.iter().any(|stage| {
            stage
                .steps
                .iter()
                .any(|step| step.contains("docker build"))
        })
    }
}
