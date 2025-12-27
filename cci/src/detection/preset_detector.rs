use crate::error::Result;
use crate::platforms::github::models::GitHubWorkflow;
use crate::platforms::gitlab::models::GitLabCI;
use crate::traits::Preset;
use std::fs;
use std::path::Path;

/// Detects which presets match existing CI configurations
pub struct PresetDetector {
    presets: Vec<Box<dyn Preset>>,
}

impl PresetDetector {
    /// Create a new preset detector with all registered presets
    pub fn new() -> Self {
        Self {
            presets: vec![
                // Presets will be registered here as they're implemented
            ],
        }
    }

    /// Add a preset to the detector
    pub fn add_preset(&mut self, preset: Box<dyn Preset>) {
        self.presets.push(preset);
    }

    /// Detect presets in a GitHub Actions workflow
    pub fn detect_github(&self, workflow: &GitHubWorkflow) -> Vec<&str> {
        self.presets
            .iter()
            .filter(|p| p.matches_github(workflow))
            .map(|p| p.name())
            .collect()
    }

    /// Detect presets in a GitLab CI config
    pub fn detect_gitlab(&self, config: &GitLabCI) -> Vec<&str> {
        self.presets
            .iter()
            .filter(|p| p.matches_gitlab(config))
            .map(|p| p.name())
            .collect()
    }

    /// Detect presets from a CI configuration file
    pub fn detect_from_file(&self, path: &Path) -> Result<Vec<&str>> {
        let content = fs::read_to_string(path)?;

        // Determine platform from path
        if path.to_string_lossy().contains(".github/workflows") {
            let workflow: GitHubWorkflow = serde_yaml::from_str(&content)?;
            Ok(self.detect_github(&workflow))
        } else if path
            .file_name()
            .map(|n| n == ".gitlab-ci.yml")
            .unwrap_or(false)
        {
            let config: GitLabCI = serde_yaml::from_str(&content)?;
            Ok(self.detect_gitlab(&config))
        } else {
            // Unknown platform
            Ok(vec![])
        }
    }

    /// List all available presets
    pub fn list_presets(&self) -> Vec<(&str, &str)> {
        self.presets
            .iter()
            .map(|p| (p.name(), p.description()))
            .collect()
    }
}

impl Default for PresetDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = PresetDetector::new();
        assert_eq!(detector.presets.len(), 0);
    }
}
