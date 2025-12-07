use crate::detection::{DetectionResult, ProjectType};
use crate::error::Result;
use crate::platforms::jenkins::models::JenkinsConfig;
use crate::presets::*;
use crate::traits::{ToCircleCI, ToGitHub, ToGitLab, ToJenkins};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonFormatterTool {
    Black,
    Ruff,
}

impl PythonFormatterTool {
    pub fn name(&self) -> &'static str {
        match self {
            PythonFormatterTool::Black => "black",
            PythonFormatterTool::Ruff => "ruff",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            PythonFormatterTool::Black => PythonFormatterTool::Ruff,
            PythonFormatterTool::Ruff => PythonFormatterTool::Black,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonLinterTool {
    Flake8,
    Ruff,
}

impl PythonLinterTool {
    pub fn name(&self) -> &'static str {
        match self {
            PythonLinterTool::Flake8 => "flake8",
            PythonLinterTool::Ruff => "ruff",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            PythonLinterTool::Flake8 => PythonLinterTool::Ruff,
            PythonLinterTool::Ruff => PythonLinterTool::Flake8,
        }
    }
}

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
    Docker,
}

impl PresetChoice {
    pub fn all() -> Vec<PresetChoice> {
        vec![
            PresetChoice::RustLibrary,
            PresetChoice::RustBinary,
            PresetChoice::PythonApp,
            PresetChoice::GoApp,
            PresetChoice::Docker,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            PresetChoice::RustLibrary => "Rust Library",
            PresetChoice::RustBinary => "Rust Binary",
            PresetChoice::PythonApp => "Python App",
            PresetChoice::GoApp => "Go App",
            PresetChoice::Docker => "Docker",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            PresetChoice::RustLibrary => "CI pipeline for Rust library projects with testing, linting, and optional coverage",
            PresetChoice::RustBinary => "CI pipeline for Rust binary/application projects with building and testing",
            PresetChoice::PythonApp => "CI pipeline for Python applications with pytest, linting, and type checking",
            PresetChoice::GoApp => "CI pipeline for Go applications with testing and linting",
            PresetChoice::Docker => "CI pipeline for building and pushing Docker images to registries",
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
            PresetChoice::PythonApp => vec!["Linter", "  → Tool", "Type Check", "Formatter", "  → Tool", "Build Wheel"],
            PresetChoice::GoApp => vec!["Linter", "Security"],
            PresetChoice::Docker => vec!["DockerHub", "GitHub Registry", "Cache", "Tags Only", "Cross-build (QEMU)", "Multi-platform"],
        }
    }

    pub fn option_description(&self, option_index: usize) -> &'static str {
        match self {
            PresetChoice::RustLibrary => match option_index {
                0 => "Enable code coverage reporting with tarpaulin",
                1 => "Enable linting with clippy",
                2 => "Enable format checking with rustfmt",
                3 => "Enable security scanning with cargo-audit",
                _ => "",
            },
            PresetChoice::RustBinary => match option_index {
                0 => "Enable linting with clippy",
                1 => "Build release binary in CI",
                _ => "",
            },
            PresetChoice::PythonApp => match option_index {
                0 => "Enable linting",
                1 => "Choose linting tool (flake8 or ruff) - toggle to switch",
                2 => "Enable type checking with mypy",
                3 => "Enable format checking",
                4 => "Choose formatting tool (black or ruff) - toggle to switch",
                5 => "Build distributable wheel package",
                _ => "",
            },
            PresetChoice::GoApp => match option_index {
                0 => "Enable linting with golangci-lint",
                1 => "Enable security scanning with gosec",
                _ => "",
            },
            PresetChoice::Docker => match option_index {
                0 => "Push images to Docker Hub (requires DOCKER_USERNAME and DOCKER_PASSWORD secrets)",
                1 => "Push images to GitHub Container Registry (uses GITHUB_TOKEN)",
                2 => "Enable Docker layer caching for faster builds",
                3 => "Only push images on git tags (not on branch pushes)",
                4 => "Enable cross-architecture builds using QEMU emulation",
                5 => "Build for multiple platforms (linux/amd64, linux/arm64) using --platform flag",
                _ => "",
            },
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
    pub enable_build_wheel: bool,
    pub python_formatter_tool: PythonFormatterTool,
    pub python_linter_tool: PythonLinterTool,

    // Docker preset options
    pub docker_use_dockerhub: bool,
    pub docker_use_github_registry: bool,
    pub docker_enable_cache: bool,
    pub docker_tags_only: bool,
    pub docker_enable_qemu: bool,
    pub docker_multiplatform: bool,
    pub docker_image_name: String,

    // UI state - tree structure
    pub expanded_presets: HashSet<PresetChoice>,
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
            working_dir: working_dir.clone(),
            enabled_presets,
            target_platform,
            enable_coverage: true,
            enable_linter: true,
            enable_formatter: true,
            enable_security: true,
            enable_build_release: true,
            enable_type_check: true,
            enable_build_wheel: false,
            python_formatter_tool: PythonFormatterTool::Black,
            python_linter_tool: PythonLinterTool::Flake8,
            docker_use_dockerhub: false,
            docker_use_github_registry: false,
            docker_enable_cache: true,
            docker_tags_only: false,
            docker_enable_qemu: false,
            docker_multiplatform: false,
            docker_image_name: working_dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("myapp")
                .to_string(),
            expanded_presets,
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
        // Reset scroll position when regenerating
        self.preview_scroll = 0;

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
            PresetChoice::Docker => {
                use crate::presets::docker::DockerRegistry;

                let registry = if self.docker_use_dockerhub {
                    DockerRegistry::DockerHub
                } else if self.docker_use_github_registry {
                    DockerRegistry::GitHubRegistry
                } else {
                    DockerRegistry::None
                };

                let preset = DockerPreset::builder()
                    .image_name(&self.docker_image_name)
                    .registry(registry)
                    .cache(self.docker_enable_cache)
                    .push_on_tags_only(self.docker_tags_only)
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
                1 => false, // Sub-option for linter tool (not a checkbox)
                2 => self.enable_type_check,
                3 => self.enable_formatter,
                4 => false, // Sub-option for formatter tool (not a checkbox)
                5 => self.enable_build_wheel,
                _ => false,
            },
            PresetChoice::GoApp => match option_index {
                0 => self.enable_linter,
                1 => self.enable_security,
                _ => false,
            },
            PresetChoice::Docker => match option_index {
                0 => self.docker_use_dockerhub,
                1 => self.docker_use_github_registry,
                2 => self.docker_enable_cache,
                3 => self.docker_tags_only,
                4 => self.docker_enable_qemu,
                5 => self.docker_multiplatform,
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
                1 => self.python_linter_tool = self.python_linter_tool.toggle(),
                2 => self.enable_type_check = !self.enable_type_check,
                3 => self.enable_formatter = !self.enable_formatter,
                4 => self.python_formatter_tool = self.python_formatter_tool.toggle(),
                5 => self.enable_build_wheel = !self.enable_build_wheel,
                _ => {}
            },
            PresetChoice::GoApp => match option_index {
                0 => self.enable_linter = !self.enable_linter,
                1 => self.enable_security = !self.enable_security,
                _ => {}
            },
            PresetChoice::Docker => match option_index {
                0 => {
                    self.docker_use_dockerhub = !self.docker_use_dockerhub;
                    if self.docker_use_dockerhub {
                        self.docker_use_github_registry = false;
                    }
                }
                1 => {
                    self.docker_use_github_registry = !self.docker_use_github_registry;
                    if self.docker_use_github_registry {
                        self.docker_use_dockerhub = false;
                    }
                }
                2 => self.docker_enable_cache = !self.docker_enable_cache,
                3 => self.docker_tags_only = !self.docker_tags_only,
                4 => self.docker_enable_qemu = !self.docker_enable_qemu,
                5 => self.docker_multiplatform = !self.docker_multiplatform,
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

    pub fn update_current_item_description(&mut self) {
        self.current_item_description = match self.current_item() {
            Some(TreeItem::Preset(preset)) => preset.description().to_string(),
            Some(TreeItem::Option(preset, option_index)) => {
                preset.option_description(*option_index).to_string()
            }
            Some(TreeItem::Platform) | None => String::new(),
        };
    }

    pub fn has_any_options_enabled(&self, preset: PresetChoice) -> bool {
        let options = preset.options();
        for i in 0..options.len() {
            if self.get_option_value(preset, i) {
                return true;
            }
        }
        false
    }

    pub fn scroll_preview_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(1);
    }

    pub fn scroll_preview_down(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_add(1);
    }
}
