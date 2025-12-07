use crate::detection::{DetectionResult, ProjectType};
use crate::error::{Error, Result};
use crate::platforms::{CircleCIAdapter, GitHubAdapter, GitLabAdapter, JenkinsAdapter, PlatformAdapter};
use crate::tui::builder::PipelineBuilder;
use crate::tui::stage::{language_from_project, PresetType, Stage};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Presets,
    Stages,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    GitHub,
    GitLab,
    CircleCI,
    Jenkins,
}

impl Platform {
    pub fn all() -> Vec<Platform> {
        vec![
            Platform::GitHub,
            Platform::GitLab,
            Platform::CircleCI,
            Platform::Jenkins,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Platform::GitHub => "GitHub Actions",
            Platform::GitLab => "GitLab CI",
            Platform::CircleCI => "CircleCI",
            Platform::Jenkins => "Jenkins",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "github" => Ok(Platform::GitHub),
            "gitlab" => Ok(Platform::GitLab),
            "circleci" => Ok(Platform::CircleCI),
            "jenkins" => Ok(Platform::Jenkins),
            _ => Err(Error::InvalidInput(format!("Unknown platform: {}", s))),
        }
    }

    pub fn output_path(&self) -> PathBuf {
        match self {
            Platform::GitHub => PathBuf::from(".github/workflows/ci.yml"),
            Platform::GitLab => PathBuf::from(".gitlab-ci.yml"),
            Platform::CircleCI => PathBuf::from(".circleci/config.yml"),
            Platform::Jenkins => PathBuf::from("Jenkinsfile"),
        }
    }

    pub fn adapter(&self) -> Box<dyn PlatformAdapter> {
        match self {
            Platform::GitHub => Box::new(GitHubAdapter),
            Platform::GitLab => Box::new(GitLabAdapter),
            Platform::CircleCI => Box::new(CircleCIAdapter),
            Platform::Jenkins => Box::new(JenkinsAdapter),
        }
    }
}

pub struct TuiState {
    // Project context (from detection)
    pub project_type: ProjectType,
    pub language_version: String,
    pub working_dir: PathBuf,

    // User selections
    pub selected_preset: PresetType,
    pub target_platform: Platform,

    // UI state
    pub active_panel: Panel,
    pub active_tab: Tab,
    pub selected_stage_index: usize,
    pub selected_preset_index: usize,
    pub selected_platform_index: usize,
    pub scroll_offset: usize,

    // Generated output
    pub pipeline_builder: PipelineBuilder,
    pub yaml_preview: String,
    pub existing_yaml: Option<String>,
    pub generation_error: Option<String>,
    pub dirty: bool,

    // Exit flag
    pub should_quit: bool,
    pub should_write: bool,
}

impl TuiState {
    /// Initialize TUI state from detection result
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

        let selected_preset = PresetType::from_project_type(&project_type);

        let target_platform = if let Some(p) = platform {
            Platform::from_str(&p)?
        } else {
            Platform::GitHub
        };

        // Initialize pipeline builder from preset
        let pipeline_builder =
            PipelineBuilder::from_preset(selected_preset, &project_type, &language_version);

        // Try to load existing CI file
        let existing_yaml = Self::load_existing_file(&working_dir, target_platform);

        let mut state = Self {
            project_type,
            language_version,
            working_dir,
            selected_preset,
            target_platform,
            active_panel: Panel::Left,
            active_tab: Tab::Stages,
            selected_stage_index: 0,
            selected_preset_index: Self::preset_index(selected_preset),
            selected_platform_index: Self::platform_index(target_platform),
            scroll_offset: 0,
            pipeline_builder,
            yaml_preview: String::new(),
            existing_yaml,
            generation_error: None,
            dirty: true,
            should_quit: false,
            should_write: false,
        };

        // Generate initial preview
        state.regenerate_yaml();

        Ok(state)
    }

    fn preset_index(preset: PresetType) -> usize {
        PresetType::all().iter().position(|&p| p == preset).unwrap_or(0)
    }

    fn platform_index(platform: Platform) -> usize {
        Platform::all().iter().position(|&p| p == platform).unwrap_or(0)
    }

    fn load_existing_file(working_dir: &PathBuf, platform: Platform) -> Option<String> {
        let path = working_dir.join(platform.output_path());
        fs::read_to_string(path).ok()
    }

    /// Toggle a stage's enabled state
    pub fn toggle_stage(&mut self, stage: Stage) {
        self.pipeline_builder.toggle_stage(stage);
        self.dirty = true;
    }

    /// Toggle currently selected stage
    pub fn toggle_selected_stage(&mut self) {
        let stages: Vec<_> = Stage::all();
        if let Some(&stage) = stages.get(self.selected_stage_index) {
            self.toggle_stage(stage);
        }
    }

    /// Navigate up in the current list
    pub fn navigate_up(&mut self) {
        match self.active_tab {
            Tab::Stages => {
                if self.selected_stage_index > 0 {
                    self.selected_stage_index -= 1;
                }
            }
            Tab::Presets => {
                if self.selected_preset_index > 0 {
                    self.selected_preset_index -= 1;
                }
            }
        }
    }

    /// Navigate down in the current list
    pub fn navigate_down(&mut self) {
        match self.active_tab {
            Tab::Stages => {
                let max = Stage::all().len() - 1;
                if self.selected_stage_index < max {
                    self.selected_stage_index += 1;
                }
            }
            Tab::Presets => {
                let max = PresetType::all().len() - 1;
                if self.selected_preset_index < max {
                    self.selected_preset_index += 1;
                }
            }
        }
    }

    /// Switch active panel
    pub fn switch_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Left => Panel::Right,
            Panel::Right => Panel::Left,
        };
    }

    /// Switch to specific tab
    pub fn switch_tab(&mut self, tab: Tab) {
        self.active_tab = tab;
    }

    /// Change preset
    pub fn change_preset(&mut self, preset: PresetType) {
        self.selected_preset = preset;
        self.selected_preset_index = Self::preset_index(preset);
        self.pipeline_builder =
            PipelineBuilder::from_preset(preset, &self.project_type, &self.language_version);
        self.dirty = true;
    }

    /// Change selected preset by index
    pub fn change_preset_by_index(&mut self) {
        if let Some(&preset) = PresetType::all().get(self.selected_preset_index) {
            self.change_preset(preset);
        }
    }

    /// Change platform
    pub fn change_platform(&mut self, platform: Platform) {
        self.target_platform = platform;
        self.selected_platform_index = Self::platform_index(platform);
        self.existing_yaml = Self::load_existing_file(&self.working_dir, platform);
        self.dirty = true;
    }

    /// Regenerate YAML preview
    pub fn regenerate_yaml(&mut self) {
        if !self.dirty {
            return;
        }

        match self.pipeline_builder.build() {
            Ok(pipeline) => {
                let adapter = self.target_platform.adapter();
                match adapter.generate(&pipeline) {
                    Ok(yaml) => {
                        self.yaml_preview = yaml;
                        self.generation_error = None;
                    }
                    Err(e) => {
                        self.generation_error = Some(format!("Generation error: {}", e));
                    }
                }
            }
            Err(e) => {
                self.generation_error = Some(format!("Build error: {}", e));
            }
        }

        self.dirty = false;
    }

    /// Get enabled stages
    pub fn enabled_stages(&self) -> HashSet<Stage> {
        self.pipeline_builder
            .enabled_stages()
            .into_iter()
            .collect()
    }

    /// Check if a stage is enabled
    pub fn is_stage_enabled(&self, stage: Stage) -> bool {
        self.pipeline_builder.is_stage_enabled(stage)
    }

    /// Request to quit
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Request to write and quit
    pub fn write_and_quit(&mut self) {
        self.should_write = true;
        self.should_quit = true;
    }

    /// Get the output file path
    pub fn output_path(&self) -> PathBuf {
        self.working_dir.join(self.target_platform.output_path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_platform_from_str() {
        assert_eq!(Platform::from_str("github").unwrap(), Platform::GitHub);
        assert_eq!(Platform::from_str("gitlab").unwrap(), Platform::GitLab);
        assert_eq!(Platform::from_str("circleci").unwrap(), Platform::CircleCI);
        assert_eq!(Platform::from_str("jenkins").unwrap(), Platform::Jenkins);
        assert!(Platform::from_str("unknown").is_err());
    }

    #[test]
    fn test_state_initialization() {
        let detection = DetectionResult {
            project_type: ProjectType::RustLibrary,
            language_version: Some("stable".to_string()),
            confidence: 1.0,
            metadata: HashMap::new(),
        };

        let state =
            TuiState::from_detection(detection, None, PathBuf::from(".")).unwrap();

        assert_eq!(state.project_type, ProjectType::RustLibrary);
        assert_eq!(state.selected_preset, PresetType::RustLibrary);
        assert_eq!(state.target_platform, Platform::GitHub);
        assert_eq!(state.active_tab, Tab::Stages);
    }

    #[test]
    fn test_toggle_stage() {
        let detection = DetectionResult {
            project_type: ProjectType::RustLibrary,
            language_version: Some("stable".to_string()),
            confidence: 1.0,
            metadata: HashMap::new(),
        };

        let mut state =
            TuiState::from_detection(detection, None, PathBuf::from(".")).unwrap();

        let initially_enabled = state.is_stage_enabled(Stage::Test);
        state.toggle_stage(Stage::Test);
        assert_eq!(state.is_stage_enabled(Stage::Test), !initially_enabled);
        assert!(state.dirty);
    }

    #[test]
    fn test_navigate() {
        let detection = DetectionResult {
            project_type: ProjectType::RustLibrary,
            language_version: Some("stable".to_string()),
            confidence: 1.0,
            metadata: HashMap::new(),
        };

        let mut state =
            TuiState::from_detection(detection, None, PathBuf::from(".")).unwrap();

        assert_eq!(state.selected_stage_index, 0);
        state.navigate_down();
        assert_eq!(state.selected_stage_index, 1);
        state.navigate_up();
        assert_eq!(state.selected_stage_index, 0);
    }
}
