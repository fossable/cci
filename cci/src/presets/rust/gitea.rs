use crate::error::Result;
use crate::traits::{ToGitHub, ToGitea};

use super::RustPreset;

impl ToGitea for RustPreset {
    fn to_gitea(&self) -> Result<crate::platforms::gitea::models::GiteaWorkflow> {
        // Gitea Actions uses the same workflow format as GitHub Actions
        self.to_github()
    }
}
