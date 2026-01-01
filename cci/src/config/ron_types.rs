use serde::{Deserialize, Serialize};

// Re-export the generated config types from presets
pub use crate::presets::{RustConfig, PythonAppConfig, GoAppConfig, DockerConfig};

/// Top-level CCI configuration - just an array of presets
pub type CciConfig = Vec<PresetChoice>;

/// Preset choice enum - supports all available presets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PresetChoice {
    #[serde(rename = "Python")]
    PythonApp(PythonAppConfig),
    Rust(RustConfig),
    GoApp(GoAppConfig),
    Docker(DockerConfig),
}

impl PresetChoice {
    /// Convert a PresetChoice to a PresetConfig using the generated conversion methods
    pub fn to_preset_config(&self) -> (String, crate::editor::config::PresetConfig) {
        use crate::presets::{RustPreset, PythonAppPreset, GoAppPreset, DockerPreset};

        match self {
            PresetChoice::Rust(config) => {
                ("rust".to_string(), RustPreset::ron_to_preset_config(config.clone()))
            }
            PresetChoice::PythonApp(config) => {
                ("python-app".to_string(), PythonAppPreset::ron_to_preset_config(config.clone()))
            }
            PresetChoice::GoApp(config) => {
                ("go-app".to_string(), GoAppPreset::ron_to_preset_config(config.clone()))
            }
            PresetChoice::Docker(config) => {
                ("docker".to_string(), DockerPreset::ron_to_preset_config(config.clone()))
            }
        }
    }
}

/// Convert a PresetChoice to a (preset_id, PresetConfig) tuple
pub fn preset_choice_to_config(choice: &PresetChoice) -> (String, crate::editor::config::PresetConfig) {
    choice.to_preset_config()
}

/// Convert a (preset_id, PresetConfig) tuple to a PresetChoice
pub fn preset_config_to_choice(preset_id: &str, config: &crate::editor::config::PresetConfig) -> PresetChoice {
    use crate::presets::{RustPreset, PythonAppPreset, GoAppPreset, DockerPreset};

    match preset_id {
        "rust" => PresetChoice::Rust(RustPreset::preset_config_to_ron(config)),
        "python-app" => PresetChoice::PythonApp(PythonAppPreset::preset_config_to_ron(config)),
        "go-app" => PresetChoice::GoApp(GoAppPreset::preset_config_to_ron(config)),
        "docker" => PresetChoice::Docker(DockerPreset::preset_config_to_ron(config)),
        _ => panic!("Unknown preset ID: {}", preset_id),
    }
}
