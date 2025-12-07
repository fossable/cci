// Addon system for extending base presets
// Addons can modify workflows/configs created by base presets

// Re-export addon traits from main traits module
pub use crate::traits::{CircleCIAddon, GitHubAddon, GitLabAddon, JenkinsAddon};

// Addon implementations will be added here as they are developed
