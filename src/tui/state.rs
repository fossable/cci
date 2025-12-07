use crate::detection::{DetectionResult, ProjectType};
use crate::error::Result;
use crate::platforms::jenkins::models::JenkinsConfig;
use crate::presets::*;
use crate::traits::{ToCircleCI, ToGitHub, ToGitLab, ToJenkins};
use std::collections::HashSet;
use std::path::PathBuf;

fn jenkins_to_string(config: &JenkinsConfig) -> String {
    let mut result = String::new();
    result.push_str(&format!("pipeline {{\n"));
    result.push_str(&format!("    agent {{\n"));
    result.push_str(&format!("        label '{}'\n", config.agent));
    result.push_str(&format!("    }}\n\n"));

    if !config.environment.is_empty() {
        result.push_str("    environment {\n");
        for (key, value) in &config.environment {
            result.push_str(&format!("        {} = '{}'\n", key, value));
        }
        result.push_str("    }\n\n");
    }

    result.push_str("    stages {\n");
    for stage in &config.stages {
        result.push_str(&format!("        stage('{}') {{\n", stage.name));
        result.push_str("            steps {\n");
        for step in &stage.steps {
            result.push_str(&format!("                {}\n", step));
        }
        result.push_str("            }\n");
        result.push_str("        }\n");
    }
    result.push_str("    }\n");
    result.push_str("}\n");
    result
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

    pub fn output_path(&self) -> PathBuf {
        match self {
            Platform::GitHub => PathBuf::from(".github/workflows/ci.yml"),
            Platform::GitLab => PathBuf::from(".gitlab-ci.yml"),
            Platform::CircleCI => PathBuf::from(".circleci/config.yml"),
            Platform::Jenkins => PathBuf::from("Jenkinsfile"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PresetChoice {
    RustLibrary,
    RustBinary,
    PythonApp,
    GoApp,
}

impl PresetChoice {
    pub fn all() -> Vec<PresetChoice> {
        vec![
            PresetChoice::RustLibrary,
            PresetChoice::RustBinary,
            PresetChoice::PythonApp,
            PresetChoice::GoApp,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            PresetChoice::RustLibrary => "Rust Library",
            PresetChoice::RustBinary => "Rust Binary",
            PresetChoice::PythonApp => "Python App",
            PresetChoice::GoApp => "Go App",
        }
    }

    pub fn from_project_type(project_type: &ProjectType) -> Self {
        match project_type {
            ProjectType::RustLibrary | ProjectType::RustWorkspace => PresetChoice::RustLibrary,
            ProjectType::RustBinary => PresetChoice::RustBinary,
            ProjectType::PythonApp | ProjectType::PythonLibrary => PresetChoice::PythonApp,
            ProjectType::GoApp | ProjectType::GoLibrary => PresetChoice::GoApp,
        }
    }

    pub fn options(&self) -> Vec<&'static str> {
        match self {
            PresetChoice::RustLibrary => vec!["Coverage", "Linter", "Formatter", "Security"],
            PresetChoice::RustBinary => vec!["Linter", "Build Release"],
            PresetChoice::PythonApp => vec!["Linter", "Type Check", "Formatter"],
            PresetChoice::GoApp => vec!["Linter", "Security"],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TreeItem {
    Preset(PresetChoice),
    Option(PresetChoice, usize),
    Platform,
}

pub struct TuiState {
    // Project context
    pub project_type: ProjectType,
    pub language_version: String,
    pub working_dir: PathBuf,

    // User selections
    pub enabled_presets: HashSet<PresetChoice>,
    pub target_platform: Platform,
    pub enable_coverage: bool,
    pub enable_linter: bool,
    pub enable_formatter: bool,
    pub enable_security: bool,
    pub enable_build_release: bool,
    pub enable_type_check: bool,

    // UI state - tree structure
    pub expanded_presets: HashSet<PresetChoice>,
    pub tree_items: Vec<TreeItem>,
    pub tree_cursor: usize,
    pub platform_menu_open: bool,
    pub platform_menu_cursor: usize,

    // Generated output
    pub yaml_preview: String,
    pub generation_error: Option<String>,

    // Exit flags
    pub should_quit: bool,
    pub should_write: bool,
}

impl TuiState {
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

        let selected_preset = PresetChoice::from_project_type(&project_type);

        let target_platform = if let Some(p) = platform {
            match p.to_lowercase().as_str() {
                "github" => Platform::GitHub,
                "gitlab" => Platform::GitLab,
                "circleci" => Platform::CircleCI,
                "jenkins" => Platform::Jenkins,
                _ => Platform::GitHub,
            }
        } else {
            Platform::GitHub
        };

        let mut expanded_presets = HashSet::new();
        expanded_presets.insert(selected_preset);

        let mut enabled_presets = HashSet::new();
        enabled_presets.insert(selected_preset);

        let mut state = Self {
            project_type,
            language_version,
            working_dir,
            enabled_presets,
            target_platform,
            enable_coverage: true,
            enable_linter: true,
            enable_formatter: true,
            enable_security: true,
            enable_build_release: true,
            enable_type_check: true,
            expanded_presets,
            tree_items: Vec::new(),
            tree_cursor: 0,
            platform_menu_open: false,
            platform_menu_cursor: Platform::all().iter().position(|&p| p == target_platform).unwrap_or(0),
            yaml_preview: String::new(),
            generation_error: None,
            should_quit: false,
            should_write: false,
        };

        state.rebuild_tree();
        state.regenerate_yaml();
        Ok(state)
    }

    pub fn rebuild_tree(&mut self) {
        self.tree_items.clear();

        // Add presets (no platform selector in tree anymore)
        for preset in PresetChoice::all() {
            self.tree_items.push(TreeItem::Preset(preset));

            // Add options if expanded
            if self.expanded_presets.contains(&preset) {
                for (i, _) in preset.options().iter().enumerate() {
                    self.tree_items.push(TreeItem::Option(preset, i));
                }
            }
        }
    }

    pub fn toggle_expand(&mut self, preset: PresetChoice) {
        if self.expanded_presets.contains(&preset) {
            self.expanded_presets.remove(&preset);
        } else {
            self.expanded_presets.insert(preset);
        }
        self.rebuild_tree();
    }

    pub fn current_item(&self) -> Option<&TreeItem> {
        self.tree_items.get(self.tree_cursor)
    }

    pub fn regenerate_yaml(&mut self) {
        // If no presets are enabled, show a message
        if self.enabled_presets.is_empty() {
            self.yaml_preview = "# No presets enabled\n# Enable at least one preset to generate configuration".to_string();
            self.generation_error = None;
            return;
        }

        // For now, just use the first enabled preset
        // TODO: In the future, we could merge multiple preset configurations
        let first_preset = self.enabled_presets.iter().next().copied();

        let result = match first_preset {
            None => {
                self.yaml_preview = "# No presets enabled".to_string();
                self.generation_error = None;
                return;
            }
            Some(selected_preset) => match selected_preset {
            PresetChoice::RustLibrary => {
                let preset = RustLibraryPreset::builder()
                    .rust_version(&self.language_version)
                    .coverage(self.enable_coverage)
                    .linter(self.enable_linter)
                    .format_check(self.enable_formatter)
                    .security_scan(self.enable_security)
                    .build();

                match self.target_platform {
                    Platform::GitHub => preset.to_github().and_then(|w| serde_yaml::to_string(&w).map_err(|e| e.into())),
                    Platform::GitLab => preset.to_gitlab().and_then(|w| serde_yaml::to_string(&w).map_err(|e| e.into())),
                    Platform::CircleCI => preset.to_circleci().and_then(|w| serde_yaml::to_string(&w).map_err(|e| e.into())),
                    Platform::Jenkins => preset.to_jenkins().map(|j| jenkins_to_string(&j)),
                }
            }
            PresetChoice::RustBinary => {
                let preset = RustBinaryPreset::builder()
                    .rust_version(&self.language_version)
                    .linter(self.enable_linter)
                    .build_release(self.enable_build_release)
                    .build();

                match self.target_platform {
                    Platform::GitHub => preset.to_github().and_then(|w| serde_yaml::to_string(&w).map_err(|e| e.into())),
                    Platform::GitLab => preset.to_gitlab().and_then(|w| serde_yaml::to_string(&w).map_err(|e| e.into())),
                    Platform::CircleCI => preset.to_circleci().and_then(|w| serde_yaml::to_string(&w).map_err(|e| e.into())),
                    Platform::Jenkins => preset.to_jenkins().map(|j| jenkins_to_string(&j)),
                }
            }
            PresetChoice::PythonApp => {
                let preset = PythonAppPreset::builder()
                    .python_version(&self.language_version)
                    .linter(self.enable_linter)
                    .type_check(self.enable_type_check)
                    .build();

                match self.target_platform {
                    Platform::GitHub => preset.to_github().and_then(|w| serde_yaml::to_string(&w).map_err(|e| e.into())),
                    Platform::GitLab => preset.to_gitlab().and_then(|w| serde_yaml::to_string(&w).map_err(|e| e.into())),
                    Platform::CircleCI => preset.to_circleci().and_then(|w| serde_yaml::to_string(&w).map_err(|e| e.into())),
                    Platform::Jenkins => preset.to_jenkins().map(|j| jenkins_to_string(&j)),
                }
            }
            PresetChoice::GoApp => {
                let preset = GoAppPreset::builder()
                    .go_version(&self.language_version)
                    .linter(self.enable_linter)
                    .security_scan(self.enable_security)
                    .build();

                match self.target_platform {
                    Platform::GitHub => preset.to_github().and_then(|w| serde_yaml::to_string(&w).map_err(|e| e.into())),
                    Platform::GitLab => preset.to_gitlab().and_then(|w| serde_yaml::to_string(&w).map_err(|e| e.into())),
                    Platform::CircleCI => preset.to_circleci().and_then(|w| serde_yaml::to_string(&w).map_err(|e| e.into())),
                    Platform::Jenkins => preset.to_jenkins().map(|j| jenkins_to_string(&j)),
                }
            }
            }
        };

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

    pub fn get_option_value(&self, preset: PresetChoice, option_index: usize) -> bool {
        match preset {
            PresetChoice::RustLibrary => match option_index {
                0 => self.enable_coverage,
                1 => self.enable_linter,
                2 => self.enable_formatter,
                3 => self.enable_security,
                _ => false,
            },
            PresetChoice::RustBinary => match option_index {
                0 => self.enable_linter,
                1 => self.enable_build_release,
                _ => false,
            },
            PresetChoice::PythonApp => match option_index {
                0 => self.enable_linter,
                1 => self.enable_type_check,
                2 => self.enable_formatter,
                _ => false,
            },
            PresetChoice::GoApp => match option_index {
                0 => self.enable_linter,
                1 => self.enable_security,
                _ => false,
            },
        }
    }

    pub fn toggle_option(&mut self, preset: PresetChoice, option_index: usize) {
        match preset {
            PresetChoice::RustLibrary => match option_index {
                0 => self.enable_coverage = !self.enable_coverage,
                1 => self.enable_linter = !self.enable_linter,
                2 => self.enable_formatter = !self.enable_formatter,
                3 => self.enable_security = !self.enable_security,
                _ => {}
            },
            PresetChoice::RustBinary => match option_index {
                0 => self.enable_linter = !self.enable_linter,
                1 => self.enable_build_release = !self.enable_build_release,
                _ => {}
            },
            PresetChoice::PythonApp => match option_index {
                0 => self.enable_linter = !self.enable_linter,
                1 => self.enable_type_check = !self.enable_type_check,
                2 => self.enable_formatter = !self.enable_formatter,
                _ => {}
            },
            PresetChoice::GoApp => match option_index {
                0 => self.enable_linter = !self.enable_linter,
                1 => self.enable_security = !self.enable_security,
                _ => {}
            },
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

    pub fn toggle_preset(&mut self, preset: PresetChoice) {
        if self.enabled_presets.contains(&preset) {
            self.enabled_presets.remove(&preset);
        } else {
            self.enabled_presets.insert(preset);
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
}
