use serde::{Deserialize, Serialize};

use crate::presets::{Docker, GoApp, PythonApp, Rust};

/// Top-level cibox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiboxConfig {
    /// Configuration format version
    pub version: String,
    /// List of preset configurations
    pub presets: Vec<PresetChoice>,
}

impl CiboxConfig {
    pub fn is_empty(&self) -> bool {
        self.presets.is_empty()
    }

    pub fn len(&self) -> usize {
        self.presets.len()
    }
}

/// Preset choice enum - supports all available presets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PresetChoice {
    #[serde(rename = "Python")]
    PythonApp(PythonApp),
    Rust(Rust),
    GoApp(GoApp),
    Docker(Docker),
}

impl PresetChoice {
    /// Convert a PresetChoice to a PresetConfig using the generated conversion methods
    pub fn to_preset_config(&self) -> (String, crate::editor::config::PresetConfig) {
        match self {
            PresetChoice::Rust(preset) => ("Rust".to_string(), preset.to_preset_config()),
            PresetChoice::PythonApp(preset) => ("PythonApp".to_string(), preset.to_preset_config()),
            PresetChoice::GoApp(preset) => ("GoApp".to_string(), preset.to_preset_config()),
            PresetChoice::Docker(preset) => ("Docker".to_string(), preset.to_preset_config()),
        }
    }
}

/// Convert a PresetChoice to a (preset_id, PresetConfig) tuple
pub fn preset_choice_to_config(
    choice: &PresetChoice,
) -> (String, crate::editor::config::PresetConfig) {
    choice.to_preset_config()
}

/// Convert a (preset_id, PresetConfig) tuple to a PresetChoice
pub fn preset_config_to_choice(
    preset_id: &str,
    config: &crate::editor::config::PresetConfig,
) -> PresetChoice {
    match preset_id {
        "Rust" => PresetChoice::Rust(Rust::from_config(config, "")),
        "PythonApp" => PresetChoice::PythonApp(PythonApp::from_config(config, "")),
        "GoApp" => PresetChoice::GoApp(GoApp::from_config(config, "")),
        "Docker" => PresetChoice::Docker(Docker::from_config(config, "")),
        _ => panic!("Unknown preset ID: {}", preset_id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_paren_presets() {
        let config: CiboxConfig = crate::config::ron_options()
            .from_str(
                r#"(
                    version: "1",
                    presets: [
                        Rust(enable_linter: true),
                        Python(linter: ruff),
                        Docker(registry: githubregistry),
                    ],
                )"#,
            )
            .unwrap();
        assert_eq!(config.len(), 3);
    }

    #[test]
    fn test_serialize_uses_single_parens() {
        let config = CiboxConfig {
            version: "1".to_string(),
            presets: vec![PresetChoice::Rust(Rust::default())],
        };
        let ron_str = crate::config::ron_options()
            .to_string_pretty(&config, ron::ser::PrettyConfig::new())
            .unwrap();
        assert!(ron_str.contains("Rust("), "{ron_str}");
        assert!(!ron_str.contains("Rust(("), "{ron_str}");

        let parsed: CiboxConfig = crate::config::ron_options().from_str(&ron_str).unwrap();
        assert_eq!(parsed.len(), 1);
    }
}
