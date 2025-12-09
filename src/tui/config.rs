use crate::detection::ProjectType;
use crate::error::Result;
use crate::tui::state::Platform;
use std::collections::HashMap;

/// Represents a configurable option value
#[derive(Debug, Clone, PartialEq)]
pub enum OptionValue {
    Bool(bool),
    Enum {
        selected: String,
        variants: Vec<String>,
    },
    String(String),
    Int(i32),
}

impl OptionValue {
    pub fn as_bool(&self) -> bool {
        matches!(self, OptionValue::Bool(true))
    }

    pub fn toggle_bool(&mut self) {
        if let OptionValue::Bool(v) = self {
            *v = !*v;
        }
    }

    pub fn cycle_enum(&mut self) {
        if let OptionValue::Enum { selected, variants } = self {
            let idx = variants.iter().position(|v| v == selected).unwrap_or(0);
            let next_idx = (idx + 1) % variants.len();
            *selected = variants[next_idx].clone();
        }
    }
}

/// Metadata for a single option within a feature
#[derive(Debug, Clone)]
pub struct OptionMeta {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub default_value: OptionValue,
    pub depends_on: Option<String>, // ID of parent option this depends on
}

/// A feature groups related options
#[derive(Debug, Clone)]
pub struct FeatureMeta {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub options: Vec<OptionMeta>,
}

/// Runtime configuration state for a preset
#[derive(Debug, Clone)]
pub struct PresetConfig {
    /// Preset ID this config belongs to
    pub preset_id: String,
    /// Flat map of option_id -> value
    pub values: HashMap<String, OptionValue>,
}

impl PresetConfig {
    pub fn new(preset_id: String) -> Self {
        Self {
            preset_id,
            values: HashMap::new(),
        }
    }

    pub fn get(&self, option_id: &str) -> Option<&OptionValue> {
        self.values.get(option_id)
    }

    pub fn get_bool(&self, option_id: &str) -> bool {
        self.values
            .get(option_id)
            .map(|v| v.as_bool())
            .unwrap_or(false)
    }

    pub fn get_string(&self, option_id: &str) -> Option<String> {
        self.values.get(option_id).and_then(|v| {
            if let OptionValue::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        })
    }

    pub fn get_enum(&self, option_id: &str) -> Option<String> {
        self.values.get(option_id).and_then(|v| {
            if let OptionValue::Enum { selected, .. } = v {
                Some(selected.clone())
            } else {
                None
            }
        })
    }

    pub fn set(&mut self, option_id: String, value: OptionValue) {
        self.values.insert(option_id, value);
    }

    pub fn toggle(&mut self, option_id: &str) {
        if let Some(value) = self.values.get_mut(option_id) {
            match value {
                OptionValue::Bool(b) => *b = !*b,
                OptionValue::Enum { .. } => value.cycle_enum(),
                _ => {}
            }
        }
    }
}

/// Trait that all presets must implement for TUI integration
pub trait TuiPreset: Send + Sync {
    /// Unique identifier for this preset
    fn preset_id(&self) -> &'static str;

    /// Human-readable name
    fn preset_name(&self) -> &'static str;

    /// Description shown in UI
    fn preset_description(&self) -> &'static str;

    /// Define the feature hierarchy for this preset
    fn features(&self) -> Vec<FeatureMeta>;

    /// Build the preset with given configuration and generate output
    fn generate(&self, config: &PresetConfig, platform: Platform, language_version: &str)
        -> Result<String>;

    /// Check if this preset matches the detected project type
    fn matches_project(&self, project_type: &ProjectType, working_dir: &std::path::Path) -> bool;

    /// Get default configuration based on project detection
    fn default_config(&self, detected: bool) -> PresetConfig;
}
