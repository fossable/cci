use super::config::EditorPreset;
use std::collections::HashMap;
use std::sync::Arc;

/// Global registry of all presets
pub struct PresetRegistry {
    presets: HashMap<String, Arc<dyn EditorPreset>>,
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

    pub fn register(&mut self, preset: Arc<dyn EditorPreset>) {
        let id = preset.preset_id().to_string();
        self.presets.insert(id.clone(), preset);
        self.order.push(id);
    }

    pub fn get(&self, id: &str) -> Option<&Arc<dyn EditorPreset>> {
        self.presets.get(id)
    }

    pub fn all(&self) -> Vec<&Arc<dyn EditorPreset>> {
        self.order
            .iter()
            .filter_map(|id| self.presets.get(id))
            .collect()
    }
}

/// Build the global preset registry
pub fn build_registry() -> PresetRegistry {
    let mut registry = PresetRegistry::new();

    // Register all editor preset implementations
    registry.register(Arc::new(crate::presets::rust::RustLibraryEditorPreset));
    registry.register(Arc::new(crate::presets::rust::RustBinaryEditorPreset));
    registry.register(Arc::new(crate::presets::python::PythonAppEditorPreset));
    registry.register(Arc::new(crate::presets::go::GoAppEditorPreset));
    registry.register(Arc::new(crate::presets::docker::DockerEditorPreset));

    registry
}
