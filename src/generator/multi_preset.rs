use crate::editor::config::PresetConfig;
use crate::editor::registry::PresetRegistry;
use crate::editor::state::Platform;
use crate::error::Result;
use std::path::PathBuf;
use std::sync::Arc;

/// Generates CI configurations for multiple presets
pub struct MultiPresetGenerator {
    preset_configs: Vec<(String, PresetConfig)>,
    registry: Arc<PresetRegistry>,
    platform: Platform,
    language_version: String,
}

impl MultiPresetGenerator {
    pub fn new(
        preset_configs: Vec<(String, PresetConfig)>,
        registry: Arc<PresetRegistry>,
        platform: Platform,
        language_version: String,
    ) -> Self {
        Self {
            preset_configs,
            registry,
            platform,
            language_version,
        }
    }

    /// Generate all preset configurations
    /// Returns a vector of (filename, content) tuples
    pub fn generate_all(&self) -> Result<Vec<(PathBuf, String)>> {
        let mut outputs = Vec::new();

        for (preset_id, config) in &self.preset_configs {
            if let Some(preset) = self.registry.get(preset_id) {
                let yaml = preset.generate(config, self.platform, &self.language_version)?;
                let filename = self.derive_filename(preset_id, self.platform);
                outputs.push((filename, yaml));
            }
        }

        Ok(outputs)
    }

    /// Derive the output filename based on preset ID and platform
    fn derive_filename(&self, preset_id: &str, platform: Platform) -> PathBuf {
        match platform {
            Platform::GitHub => {
                // For GitHub, create separate workflow files per preset
                PathBuf::from(format!(".github/workflows/{}.yml", preset_id))
            }
            Platform::GitLab => {
                // GitLab uses a single file - for multiple presets, we need to merge
                // For now, we'll use the preset ID as a suffix if there are multiple
                if self.preset_configs.len() > 1 {
                    PathBuf::from(format!(".gitlab-ci-{}.yml", preset_id))
                } else {
                    PathBuf::from(".gitlab-ci.yml")
                }
            }
            Platform::CircleCI => {
                // CircleCI uses a single file - for multiple presets, we need to merge
                // For now, we'll use the preset ID as a suffix if there are multiple
                if self.preset_configs.len() > 1 {
                    PathBuf::from(format!(".circleci/config-{}.yml", preset_id))
                } else {
                    PathBuf::from(".circleci/config.yml")
                }
            }
            Platform::Jenkins => {
                // Jenkins uses a single file - for multiple presets, we need to merge
                // For now, we'll use the preset ID as a suffix if there are multiple
                if self.preset_configs.len() > 1 {
                    PathBuf::from(format!("Jenkinsfile-{}", preset_id))
                } else {
                    PathBuf::from("Jenkinsfile")
                }
            }
        }
    }
}
