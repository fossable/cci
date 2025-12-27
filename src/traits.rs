use crate::error::Result;
use crate::platforms::circleci::models::CircleCIConfig;
use crate::platforms::gitea::models::GiteaWorkflow;
use crate::platforms::github::models::GitHubWorkflow;
use crate::platforms::gitlab::models::GitLabCI;
use crate::platforms::jenkins::models::JenkinsConfig;

/// Trait for converting a preset to GitHub Actions workflow
pub trait ToGitHub {
    fn to_github(&self) -> Result<GitHubWorkflow>;
}

/// Trait for converting a preset to Gitea Actions workflow
pub trait ToGitea {
    fn to_gitea(&self) -> Result<GiteaWorkflow>;
}

/// Trait for converting a preset to GitLab CI config
pub trait ToGitLab {
    fn to_gitlab(&self) -> Result<GitLabCI>;
}

/// Trait for converting a preset to CircleCI config
pub trait ToCircleCI {
    fn to_circleci(&self) -> Result<CircleCIConfig>;
}

/// Trait for converting a preset to Jenkins pipeline
pub trait ToJenkins {
    fn to_jenkins(&self) -> Result<JenkinsConfig>;
}

/// Trait for detecting if a preset matches an existing config
pub trait Detectable {
    /// Check if this preset matches a GitHub Actions workflow
    fn matches_github(&self, workflow: &GitHubWorkflow) -> bool;

    /// Check if this preset matches a Gitea Actions workflow
    fn matches_gitea(&self, workflow: &GiteaWorkflow) -> bool;

    /// Check if this preset matches a GitLab CI config
    fn matches_gitlab(&self, config: &GitLabCI) -> bool;

    /// Check if this preset matches a CircleCI config
    fn matches_circleci(&self, config: &CircleCIConfig) -> bool;

    /// Check if this preset matches a Jenkins pipeline
    fn matches_jenkins(&self, pipeline: &JenkinsConfig) -> bool;
}

/// Trait for preset identification
pub trait PresetInfo {
    /// Unique identifier for this preset
    fn name(&self) -> &str;

    /// Human-readable description of what this preset does
    fn description(&self) -> &str;
}

/// Combined trait for presets that can be detected and provide info
/// This allows presets to be used in trait objects
pub trait Preset: Detectable + PresetInfo {}

/// Automatically implement Preset for any type that implements both Detectable and PresetInfo
impl<T: Detectable + PresetInfo> Preset for T {}
