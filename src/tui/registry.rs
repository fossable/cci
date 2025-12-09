use super::config::TuiPreset;
use std::collections::HashMap;
use std::sync::Arc;

/// Global registry of all presets
pub struct PresetRegistry {
    presets: HashMap<String, Arc<dyn TuiPreset>>,
    /// Ordered list of preset IDs for consistent iteration
    order: Vec<String>,
}

impl PresetRegistry {
    pub fn new() -> Self {
        Self {
            presets: HashMap::new(),
            order: Vec::new(),
        }
    }

    pub fn register(&mut self, preset: Arc<dyn TuiPreset>) {
        let id = preset.preset_id().to_string();
        self.presets.insert(id.clone(), preset);
        self.order.push(id);
    }

    pub fn get(&self, id: &str) -> Option<&Arc<dyn TuiPreset>> {
        self.presets.get(id)
    }

    pub fn all(&self) -> Vec<&Arc<dyn TuiPreset>> {
        self.order
            .iter()
            .filter_map(|id| self.presets.get(id))
            .collect()
    }
}

/// Build the global preset registry
pub fn build_registry() -> PresetRegistry {
    let mut registry = PresetRegistry::new();

    // Register all TUI preset implementations
    registry.register(Arc::new(crate::presets::rust::RustLibraryTuiPreset));
    registry.register(Arc::new(crate::presets::rust::RustBinaryTuiPreset));
    registry.register(Arc::new(crate::presets::python::PythonAppTuiPreset));
    registry.register(Arc::new(crate::presets::go::GoAppTuiPreset));
    registry.register(Arc::new(crate::presets::docker::DockerTuiPreset));

    registry
}
