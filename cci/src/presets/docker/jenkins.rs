use crate::error::Result;
use crate::platforms::jenkins::models::JenkinsConfig;
use crate::traits::ToJenkins;

use super::{DockerPreset, DockerRegistry};

impl ToJenkins for DockerPreset {
    fn to_jenkins(&self) -> Result<JenkinsConfig> {
        use crate::platforms::jenkins::models::*;

        let mut steps = Vec::new();

        // Login command
        let login_cmd = match &self.registry {
            DockerRegistry::DockerHub => {
                "sh 'echo $DOCKER_PASSWORD | docker login -u $DOCKER_USERNAME --password-stdin'"
                    .to_string()
            }
            DockerRegistry::GitHubRegistry => {
                "sh 'echo $GITHUB_TOKEN | docker login ghcr.io -u $GITHUB_USERNAME --password-stdin'"
                    .to_string()
            }
            DockerRegistry::None => String::new(),
        };

        if !login_cmd.is_empty() {
            steps.push(login_cmd);
        }

        // Build command
        let mut build_cmd = format!(
            "sh 'docker build -t {} -f {}",
            self.image_name, self.dockerfile_path
        );

        // Build args removed for simplicity

        build_cmd.push_str(&format!(" {}'", self.build_context));
        steps.push(build_cmd);

        // Push command
        if self.registry != DockerRegistry::None {
            steps.push(format!("sh 'docker push {}'", self.image_name));
        }

        Ok(JenkinsConfig {
            agent: "any".to_string(),
            environment: vec![],
            stages: vec![JenkinsStage {
                name: "Docker Build".to_string(),
                steps,
            }],
        })
    }
}
