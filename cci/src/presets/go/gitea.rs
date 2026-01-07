use crate::error::Result;
use crate::traits::{ToGitHub, ToGitea};

use super::GoAppPreset;

impl ToGitea for GoAppPreset {
    fn to_gitea(&self) -> Result<crate::platforms::gitea::models::GiteaWorkflow> {
        // Gitea Actions uses the same workflow format as GitHub Actions
        self.to_github()
    }
}
