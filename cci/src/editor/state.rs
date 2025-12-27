use crate::detection::{DetectionResult, ProjectType};
use crate::editor::config::{OptionValue, PresetConfig};
use crate::editor::registry::{build_registry, PresetRegistry};
use crate::error::Result;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    GitHub,
    Gitea,
    GitLab,
    CircleCI,
    Jenkins,
}

impl Platform {
    pub fn all() -> Vec<Platform> {
        vec![
            Platform::GitHub,
            Platform::Gitea,
            Platform::GitLab,
            Platform::CircleCI,
            Platform::Jenkins,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Platform::GitHub => "GitHub Actions",
            Platform::Gitea => "Gitea Actions",
            Platform::GitLab => "GitLab CI",
            Platform::CircleCI => "CircleCI",
            Platform::Jenkins => "Jenkins",
        }
    }

    pub fn output_path(&self) -> PathBuf {
        match self {
            Platform::GitHub => PathBuf::from(".github/workflows/ci.yml"),
            Platform::Gitea => PathBuf::from(".gitea/workflows/ci.yml"),
            Platform::GitLab => PathBuf::from(".gitlab-ci.yml"),
            Platform::CircleCI => PathBuf::from(".circleci/config.yml"),
            Platform::Jenkins => PathBuf::from("Jenkinsfile"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TreeItem {
    Preset(String),                        // preset_id
    Feature(String, String),               // preset_id, feature_id
    Option(String, String, String),        // preset_id, feature_id, option_id
}

pub struct EditorState {
    // Project context
    pub project_type: ProjectType,
    pub language_version: String,
    pub working_dir: PathBuf,

    // User selections
    pub target_platform: Platform,

    // Dynamic preset configuration (REPLACES all hardcoded fields!)
    pub registry: Arc<PresetRegistry>,
    pub preset_configs: HashMap<String, PresetConfig>,

    // UI state - tree structure
    pub expanded_presets: HashSet<String>,  // preset IDs
    pub expanded_features: HashSet<(String, String)>,  // (preset_id, feature_id)
    pub tree_items: Vec<TreeItem>,
    pub tree_cursor: usize,
    pub platform_menu_open: bool,
    pub platform_menu_cursor: usize,

    // Preview scroll state
    pub preview_scroll: u16,

    // Generated output
    pub yaml_preview: String,
    pub generation_error: Option<String>,

    // UI info
    pub current_item_description: String,

    // Exit flags
    pub should_quit: bool,
    pub should_write: bool,
}

impl EditorState {
    pub fn from_detection(
        detection: DetectionResult,
        platform: Option<String>,
        working_dir: PathBuf,
    ) -> Result<Self> {
        let project_type = detection.project_type.clone();
        let language_version = detection
            .language_version
            .clone()
            .unwrap_or_else(|| "stable".to_string());

        let target_platform = if let Some(p) = platform {
            match p.to_lowercase().as_str() {
                "github" => Platform::GitHub,
                "gitea" => Platform::Gitea,
                "gitlab" => Platform::GitLab,
                "circleci" => Platform::CircleCI,
                "jenkins" => Platform::Jenkins,
                _ => Platform::GitHub,
            }
        } else {
            Platform::GitHub
        };

        // Build the preset registry
        let registry = Arc::new(build_registry());

        // Initialize preset configs for all presets
        let mut preset_configs = HashMap::new();
        let mut expanded_presets = HashSet::new();

        for preset in registry.all() {
            let preset_id = preset.preset_id();
            let matches = preset.matches_project(&project_type, &working_dir);

            // Create default config based on whether it matches
            let config = preset.default_config(matches);
            preset_configs.insert(preset_id.to_string(), config);

            // Expand matching presets by default
            if matches {
                expanded_presets.insert(preset_id.to_string());
            }
        }

        let mut state = Self {
            project_type,
            language_version,
            working_dir,
            target_platform,
            registry,
            preset_configs,
            expanded_presets,
            expanded_features: HashSet::new(),
            tree_items: Vec::new(),
            tree_cursor: 0,
            platform_menu_open: false,
            platform_menu_cursor: Platform::all().iter().position(|&p| p == target_platform).unwrap_or(0),
            preview_scroll: 0,
            yaml_preview: String::new(),
            generation_error: None,
            current_item_description: String::new(),
            should_quit: false,
            should_write: false,
        };

        state.rebuild_tree();
        state.regenerate_yaml();
        state.update_current_item_description();
        Ok(state)
    }


    pub fn rebuild_tree(&mut self) {
        self.tree_items.clear();

        // Get all presets and sort: matching ones first, then others
        let mut all_presets: Vec<_> = self.registry.all().into_iter().collect();
        all_presets.sort_by_key(|preset| !preset.matches_project(&self.project_type, &self.working_dir));

        // Build three-level tree: Preset → Feature → Option
        for preset in all_presets {
            let preset_id = preset.preset_id().to_string();
            self.tree_items.push(TreeItem::Preset(preset_id.clone()));

            // Add features if preset is expanded
            if self.expanded_presets.contains(&preset_id) {
                for feature in preset.features() {
                    let feature_id = feature.id.clone();
                    self.tree_items.push(TreeItem::Feature(preset_id.clone(), feature_id.clone()));

                    // Add options if feature is expanded
                    if self.expanded_features.contains(&(preset_id.clone(), feature_id.clone())) {
                        for option in &feature.options {
                            self.tree_items.push(TreeItem::Option(
                                preset_id.clone(),
                                feature_id.clone(),
                                option.id.clone(),
                            ));
                        }
                    }
                }
            }
        }
    }

    pub fn toggle_preset_expand(&mut self, preset_id: &str) {
        if self.expanded_presets.contains(preset_id) {
            self.expanded_presets.remove(preset_id);
            // Also collapse all features of this preset
            self.expanded_features.retain(|(p, _)| p != preset_id);
        } else {
            self.expanded_presets.insert(preset_id.to_string());
        }
        self.rebuild_tree();
    }

    pub fn toggle_feature_expand(&mut self, preset_id: &str, feature_id: &str) {
        let key = (preset_id.to_string(), feature_id.to_string());
        if self.expanded_features.contains(&key) {
            self.expanded_features.remove(&key);
        } else {
            self.expanded_features.insert(key);
        }
        self.rebuild_tree();
    }

    pub fn current_item(&self) -> Option<&TreeItem> {
        self.tree_items.get(self.tree_cursor)
    }

    pub fn regenerate_yaml(&mut self) {
        // Reset scroll position when regenerating
        self.preview_scroll = 0;

        // Find the first preset that has any options enabled
        let active_preset = self.registry.all()
            .into_iter()
            .find(|preset| {
                if let Some(config) = self.preset_configs.get(preset.preset_id()) {
                    self.has_any_options_enabled(config)
                } else {
                    false
                }
            });

        let Some(preset) = active_preset else {
            self.yaml_preview = "# No preset options enabled\n# Enable at least one option to generate configuration".to_string();
            self.generation_error = None;
            return;
        };

        let preset_id = preset.preset_id();
        let config = match self.preset_configs.get(preset_id) {
            Some(c) => c,
            None => {
                self.generation_error = Some(format!("Config not found for preset: {}", preset_id));
                return;
            }
        };

        let result = preset.generate(config, self.target_platform, &self.language_version);

        match result {
            Ok(yaml) => {
                self.yaml_preview = yaml;
                self.generation_error = None;
            }
            Err(e) => {
                self.generation_error = Some(e.to_string());
            }
        }
    }

    fn has_any_options_enabled(&self, config: &PresetConfig) -> bool {
        config.values.values().any(|v| match v {
            OptionValue::Bool(b) => *b,
            _ => true, // Consider non-bool options as "enabled" if they have a value
        })
    }

    pub fn get_option_value(&self, preset_id: &str, option_id: &str) -> Option<&OptionValue> {
        self.preset_configs
            .get(preset_id)
            .and_then(|config| config.get(option_id))
    }

    pub fn set_option_value(&mut self, preset_id: &str, option_id: &str, value: OptionValue) {
        if let Some(config) = self.preset_configs.get_mut(preset_id) {
            config.set(option_id.to_string(), value);
        }
    }

    pub fn toggle_option(&mut self, preset_id: &str, option_id: &str) {
        if let Some(config) = self.preset_configs.get_mut(preset_id) {
            if let Some(value) = config.get(option_id) {
                let new_value = match value {
                    OptionValue::Bool(b) => OptionValue::Bool(!b),
                    OptionValue::Enum { selected, variants } => {
                        // Cycle to next variant
                        let current_index = variants.iter().position(|v| v == selected).unwrap_or(0);
                        let next_index = (current_index + 1) % variants.len();
                        OptionValue::Enum {
                            selected: variants[next_index].clone(),
                            variants: variants.clone(),
                        }
                    }
                    other => other.clone(),
                };
                config.set(option_id.to_string(), new_value);
            }
        }
        self.regenerate_yaml();
    }

    pub fn cycle_platform(&mut self) {
        let platforms = Platform::all();
        let current_index = platforms.iter().position(|&p| p == self.target_platform).unwrap_or(0);
        let next_index = (current_index + 1) % platforms.len();
        self.target_platform = platforms[next_index];
        self.regenerate_yaml();
    }

    pub fn toggle_preset(&mut self, preset_id: &str) {
        // Check if any options are currently enabled
        let config = match self.preset_configs.get(preset_id) {
            Some(c) => c,
            None => return,
        };

        let has_enabled = self.has_any_options_enabled(config);

        // Get preset to access all its options
        let preset = match self.registry.get(preset_id) {
            Some(p) => p,
            None => return,
        };

        // Toggle all boolean options for this preset
        for feature in preset.features() {
            for option in &feature.options {
                if matches!(option.default_value, OptionValue::Bool(_)) {
                    self.set_option_value(preset_id, &option.id, OptionValue::Bool(!has_enabled));
                }
            }
        }

        self.regenerate_yaml();
    }

    pub fn open_platform_menu(&mut self) {
        self.platform_menu_open = true;
    }

    pub fn close_platform_menu(&mut self) {
        self.platform_menu_open = false;
    }

    pub fn select_platform_from_menu(&mut self) {
        let platforms = Platform::all();
        if let Some(&platform) = platforms.get(self.platform_menu_cursor) {
            self.target_platform = platform;
            self.regenerate_yaml();
        }
        self.platform_menu_open = false;
    }

    pub fn update_current_item_description(&mut self) {
        self.current_item_description = match self.current_item() {
            Some(TreeItem::Preset(preset_id)) => {
                self.registry.get(preset_id)
                    .map(|p| p.preset_description().to_string())
                    .unwrap_or_default()
            }
            Some(TreeItem::Feature(preset_id, feature_id)) => {
                self.registry.get(preset_id)
                    .and_then(|preset| {
                        preset.features()
                            .into_iter()
                            .find(|f| &f.id == feature_id)
                            .map(|f| f.description.clone())
                    })
                    .unwrap_or_default()
            }
            Some(TreeItem::Option(preset_id, feature_id, option_id)) => {
                self.registry.get(preset_id)
                    .and_then(|preset| {
                        preset.features()
                            .into_iter()
                            .find(|f| &f.id == feature_id)
                            .and_then(|f| {
                                f.options.iter()
                                    .find(|o| &o.id == option_id)
                                    .map(|o| o.description.clone())
                            })
                    })
                    .unwrap_or_default()
            }
            None => String::new(),
        };
    }

    pub fn scroll_preview_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(1);
    }

    pub fn scroll_preview_down(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_add(1);
    }

    /// Load RON configuration into TUI state
    pub fn from_ron_file(path: &std::path::Path) -> Result<Self> {
        use crate::config::{preset_choice_to_config, CciConfig};
        use anyhow::Context;

        let ron_str = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read RON file: {}", path.display()))?;

        let ron_config: CciConfig = ron::from_str(&ron_str)
            .with_context(|| "Failed to parse RON configuration")?;

        let registry = Arc::new(build_registry());
        let mut preset_configs = HashMap::new();

        for preset_choice in ron_config.presets {
            let (preset_id, config) = preset_choice_to_config(&preset_choice);
            preset_configs.insert(preset_id, config);
        }

        let mut state = Self {
            project_type: ProjectType::PythonApp, // Default, doesn't affect RON-loaded config
            language_version: "stable".to_string(),
            working_dir: path.parent().unwrap_or(std::path::Path::new(".")).to_path_buf(),
            target_platform: Platform::GitHub, // Default platform
            registry,
            preset_configs,
            expanded_presets: HashSet::new(),
            expanded_features: HashSet::new(),
            tree_items: Vec::new(),
            tree_cursor: 0,
            platform_menu_open: false,
            platform_menu_cursor: 0,
            preview_scroll: 0,
            yaml_preview: String::new(),
            generation_error: None,
            current_item_description: String::new(),
            should_quit: false,
            should_write: false,
        };

        state.rebuild_tree();
        state.regenerate_yaml();
        state.update_current_item_description();
        Ok(state)
    }

    /// Export current TUI state to RON configuration
    pub fn export_to_ron(&self) -> Result<String> {
        use crate::config::{preset_config_to_choice, CciConfig};

        let mut presets = Vec::new();

        for (preset_id, config) in &self.preset_configs {
            if self.has_any_options_enabled(config) {
                let preset_choice = preset_config_to_choice(preset_id, config);
                presets.push(preset_choice);
            }
        }

        let ron_config = CciConfig { presets };

        let pretty_config = ron::ser::PrettyConfig::new()
            .depth_limit(4)
            .separate_tuple_members(true)
            .enumerate_arrays(false);

        let ron_str = ron::ser::to_string_pretty(&ron_config, pretty_config)
            .map_err(|e| anyhow::anyhow!("Failed to serialize to RON: {}", e))?;

        Ok(ron_str)
    }

    /// Save current state to a RON file
    pub fn save_to_ron_file(&self, path: &std::path::Path) -> Result<()> {
        use anyhow::Context;

        let ron_str = self.export_to_ron()?;

        std::fs::write(path, ron_str)
            .with_context(|| format!("Failed to write RON file: {}", path.display()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_rust_library_enables_only_matching_presets() {
        let dir = tempdir().unwrap();

        let detection = DetectionResult {
            project_type: ProjectType::RustLibrary,
            language_version: Some("stable".to_string()),
            metadata: HashMap::new(),
        };

        let state = EditorState::from_detection(detection, None, dir.path().to_path_buf()).unwrap();

        // All presets should be shown in the tree
        let preset_items: Vec<&str> = state.tree_items
            .iter()
            .filter_map(|item| match item {
                TreeItem::Preset(preset_id) => Some(preset_id.as_str()),
                _ => None,
            })
            .collect();

        assert_eq!(preset_items.len(), 4); // All 4 presets
        assert!(preset_items.contains(&"rust"));
        assert!(preset_items.contains(&"python-app"));
        assert!(preset_items.contains(&"go-app"));
        assert!(preset_items.contains(&"docker"));

        // But only Rust options should be enabled by default
        let rust_config = state.preset_configs.get("rust").unwrap();
        assert_eq!(rust_config.get_bool("enable_coverage"), true);
        assert_eq!(rust_config.get_bool("enable_linter"), true);

        let python_config = state.preset_configs.get("python-app").unwrap();
        assert_eq!(python_config.get_bool("enable_linter"), false);
    }

    #[test]
    fn test_rust_binary_enables_only_matching_presets() {
        let dir = tempdir().unwrap();

        let detection = DetectionResult {
            project_type: ProjectType::RustBinary,
            language_version: Some("stable".to_string()),
            metadata: HashMap::new(),
        };

        let state = EditorState::from_detection(detection, None, dir.path().to_path_buf()).unwrap();

        let rust_config = state.preset_configs.get("rust").unwrap();
        assert_eq!(rust_config.get_bool("enable_linter"), true);
        assert_eq!(rust_config.get_bool("build_release"), true);

        let python_config = state.preset_configs.get("python-app").unwrap();
        assert_eq!(python_config.get_bool("enable_linter"), false);
    }

    #[test]
    fn test_python_app_enables_only_matching_presets() {
        let dir = tempdir().unwrap();

        let detection = DetectionResult {
            project_type: ProjectType::PythonApp,
            language_version: Some("3.11".to_string()),
            metadata: HashMap::new(),
        };

        let state = EditorState::from_detection(detection, None, dir.path().to_path_buf()).unwrap();

        let rust_config = state.preset_configs.get("rust").unwrap();
        assert_eq!(rust_config.get_bool("enable_coverage"), false);

        let python_config = state.preset_configs.get("python-app").unwrap();
        assert_eq!(python_config.get_bool("enable_linter"), true);
        assert_eq!(python_config.get_bool("enable_formatter"), true);
    }

    #[test]
    fn test_go_app_enables_only_matching_presets() {
        let dir = tempdir().unwrap();

        let detection = DetectionResult {
            project_type: ProjectType::GoApp,
            language_version: Some("1.21".to_string()),
            metadata: HashMap::new(),
        };

        let state = EditorState::from_detection(detection, None, dir.path().to_path_buf()).unwrap();

        let go_config = state.preset_configs.get("go-app").unwrap();
        assert_eq!(go_config.get_bool("enable_linter"), true);
        assert_eq!(go_config.get_bool("enable_security"), true);
    }

    #[test]
    fn test_docker_preset_shown_for_all_project_types() {
        let project_types = vec![
            ProjectType::RustLibrary,
            ProjectType::RustBinary,
            ProjectType::PythonApp,
            ProjectType::GoApp,
        ];

        for project_type in project_types {
            let dir = tempdir().unwrap();

            let detection = DetectionResult {
                project_type: project_type.clone(),
                language_version: Some("stable".to_string()),
                    metadata: HashMap::new(),
            };

            let state = EditorState::from_detection(detection, None, dir.path().to_path_buf()).unwrap();

            let has_docker = state.tree_items
                .iter()
                .any(|item| matches!(item, TreeItem::Preset(id) if id == "docker"));

            assert!(has_docker, "Docker preset should be shown for {:?}", project_type);
        }
    }

    #[test]
    fn test_docker_enabled_with_docker_project_type() {
        let dir = tempdir().unwrap();

        let detection = DetectionResult {
            project_type: ProjectType::DockerImage,
            language_version: None,
            metadata: HashMap::new(),
        };

        let state = EditorState::from_detection(detection, None, dir.path().to_path_buf()).unwrap();

        let docker_config = state.preset_configs.get("docker").unwrap();
        assert_eq!(docker_config.get_bool("enable_cache"), true);
    }

    #[test]
    fn test_docker_disabled_for_non_docker_project() {
        let dir = tempdir().unwrap();

        let detection = DetectionResult {
            project_type: ProjectType::RustLibrary,
            language_version: Some("stable".to_string()),
            metadata: HashMap::new(),
        };

        let state = EditorState::from_detection(detection, None, dir.path().to_path_buf()).unwrap();

        let docker_config = state.preset_configs.get("docker").unwrap();
        // Docker preset is available but not enabled by default for non-Docker projects
        assert_eq!(docker_config.get_bool("enable_cache"), false);
    }

    #[test]
    fn test_docker_preset_available_for_all_projects() {
        // Docker preset should be available (but not enabled) for all project types
        // Users can manually enable it if they want to add Docker to their project
        let dir = tempdir().unwrap();

        let detection = DetectionResult {
            project_type: ProjectType::RustLibrary,
            language_version: Some("stable".to_string()),
            metadata: HashMap::new(),
        };

        let state = EditorState::from_detection(detection, None, dir.path().to_path_buf()).unwrap();

        // Docker config should exist
        let docker_config = state.preset_configs.get("docker");
        assert!(docker_config.is_some(), "Docker preset should be available");

        // But not enabled by default
        assert_eq!(docker_config.unwrap().get_bool("enable_cache"), false);
    }

    #[test]
    fn test_manually_enabling_non_detected_preset_generates_yaml() {
        let dir = tempdir().unwrap();

        let detection = DetectionResult {
            project_type: ProjectType::RustLibrary,
            language_version: Some("stable".to_string()),
            metadata: HashMap::new(),
        };

        let mut state = EditorState::from_detection(detection, None, dir.path().to_path_buf()).unwrap();

        // The YAML should be for Rust initially
        assert!(state.yaml_preview.contains("cargo"));

        // Now manually disable Rust and enable Python App
        use crate::editor::config::OptionValue;
        state.set_option_value("rust", "enable_coverage", OptionValue::Bool(false));
        state.set_option_value("rust", "enable_linter", OptionValue::Bool(false));
        state.set_option_value("rust", "enable_formatter", OptionValue::Bool(false));
        state.set_option_value("rust", "enable_security", OptionValue::Bool(false));
        state.set_option_value("rust", "build_release", OptionValue::Bool(false));

        state.set_option_value("python-app", "enable_linter", OptionValue::Bool(true));
        state.set_option_value("python-app", "enable_formatter", OptionValue::Bool(true));

        state.regenerate_yaml();

        // The YAML should now be for Python, even though it's not the detected type
        assert!(state.yaml_preview.contains("python") || state.yaml_preview.contains("pytest") || state.yaml_preview.contains("Setup Python"));
        assert!(!state.yaml_preview.contains("cargo"));
    }

    #[test]
    fn test_multiple_presets_first_enabled_wins() {
        let dir = tempdir().unwrap();

        let detection = DetectionResult {
            project_type: ProjectType::RustLibrary,
            language_version: Some("stable".to_string()),
            metadata: HashMap::new(),
        };

        let mut state = EditorState::from_detection(detection, None, dir.path().to_path_buf()).unwrap();

        // Enable both Rust and Python (unusual but allowed)
        use crate::editor::config::OptionValue;
        state.set_option_value("rust", "enable_linter", OptionValue::Bool(true));
        state.set_option_value("python-app", "enable_linter", OptionValue::Bool(true));

        state.regenerate_yaml();

        // The first preset with options enabled should be used (registry order)
        assert!(state.yaml_preview.contains("cargo"));
    }
}
