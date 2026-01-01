use crate::error::Result;
use crate::traits::{ToGitea, ToGitHub};

use super::DockerPreset;

impl ToGitea for DockerPreset {
    fn to_gitea(&self) -> Result<crate::platforms::gitea::models::GiteaWorkflow> {
        // Gitea Actions uses the same workflow format as GitHub Actions
        self.to_github()
    }
}
