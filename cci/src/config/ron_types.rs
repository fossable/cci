use serde::{Deserialize, Serialize};

// Re-export the generated config types from presets
pub use crate::presets::{DockerConfig, GoAppConfig, PythonAppConfig, RustConfig};

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
        use crate::presets::{Docker, GoApp, PythonApp, Rust};

        match self {
            PresetChoice::Rust(config) => (
                "Rust".to_string(),
                Rust::ron_to_preset_config(config.clone()),
            ),
            PresetChoice::PythonApp(config) => (
                "PythonApp".to_string(),
                PythonApp::ron_to_preset_config(config.clone()),
            ),
            PresetChoice::GoApp(config) => (
                "GoApp".to_string(),
                GoApp::ron_to_preset_config(config.clone()),
            ),
            PresetChoice::Docker(config) => (
                "Docker".to_string(),
                Docker::ron_to_preset_config(config.clone()),
            ),
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
    use crate::presets::{Docker, GoApp, PythonApp, Rust};

    match preset_id {
        "Rust" => PresetChoice::Rust(Rust::preset_config_to_ron(config)),
        "PythonApp" => PresetChoice::PythonApp(PythonApp::preset_config_to_ron(config)),
        "GoApp" => PresetChoice::GoApp(GoApp::preset_config_to_ron(config)),
        "Docker" => PresetChoice::Docker(Docker::preset_config_to_ron(config)),
        _ => panic!("Unknown preset ID: {}", preset_id),
    }
}
