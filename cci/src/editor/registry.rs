use super::config::EditorPreset;
use std::sync::Arc;

/// Global registry of all presets
///
/// Uses a simple Vec for storage since we have a small number of presets (~5).
/// Linear search is acceptable for this scale and simplifies the implementation.
pub struct PresetRegistry {
    presets: Vec<Arc<dyn EditorPreset>>,
}

impl PresetRegistry {
    pub fn new() -> Self {
        Self {
            presets: Vec::new(),
        }
    }

    pub fn register(&mut self, preset: Arc<dyn EditorPreset>) {
        self.presets.push(preset);
    }

    pub fn get(&self, id: &str) -> Option<&Arc<dyn EditorPreset>> {
        self.presets.iter().find(|p| p.preset_id() == id)
    }

    pub fn all(&self) -> Vec<&Arc<dyn EditorPreset>> {
        self.presets.iter().collect()
    }
}

/// Build the global preset registry
pub fn build_registry() -> PresetRegistry {
    let mut registry = PresetRegistry::new();

    // Register all editor preset implementations
    registry.register(Arc::new(crate::presets::RustPreset::default()));
    registry.register(Arc::new(crate::presets::PythonAppPreset::default()));
    registry.register(Arc::new(crate::presets::GoAppPreset::default()));
    registry.register(Arc::new(crate::presets::DockerPreset::DEFAULT));

    registry
}
