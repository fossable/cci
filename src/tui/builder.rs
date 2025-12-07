use crate::detection::ProjectType;
use crate::error::Result;
use crate::models::{CacheConfig, Job, Pipeline, Runner, Step, Trigger};
use crate::models::step::Language;
use crate::tui::stage::{language_from_project, PresetType, Stage};
use std::collections::HashMap;

/// Configuration for a stage's job
#[derive(Debug, Clone)]
pub struct StageConfig {
    pub enabled: bool,
    pub job_name: String,
    pub runner: Runner,
    pub steps: Vec<Step>,
}

/// Builder for constructing Pipeline structs from TUI state
#[derive(Debug, Clone)]
pub struct PipelineBuilder {
    name: String,
    language: Language,
    version: String,
    triggers: Vec<Trigger>,
    env: HashMap<String, String>,
    cache: CacheConfig,
    stages: HashMap<Stage, StageConfig>,
}

impl PipelineBuilder {
    /// Create a new PipelineBuilder with basic defaults
    pub fn new(language: Language, version: &str) -> Self {
        Self {
            name: "CI".to_string(),
            language,
            version: version.to_string(),
            triggers: vec![
                Trigger::Push {
                    branches: vec!["main".to_string()],
                },
                Trigger::PullRequest {
                    branches: vec!["main".to_string()],
                },
            ],
            env: HashMap::new(),
            cache: CacheConfig::default(),
            stages: HashMap::new(),
        }
    }

    /// Initialize builder from a preset type
    pub fn from_preset(preset: PresetType, project_type: &ProjectType, version: &str) -> Self {
        let language = language_from_project(project_type);
        let mut builder = Self::new(language, version);

        // Initialize all stages with default enabled states
        for stage in Stage::all() {
            let enabled = stage.default_enabled_for(project_type);
            builder.add_stage_config(stage, enabled);
        }

        // Add tag trigger for release stage
        if builder.is_stage_enabled(Stage::Release) {
            builder.triggers.push(Trigger::Tag {
                pattern: "v*".to_string(),
            });
        }

        builder
    }

    /// Add stage configuration
    fn add_stage_config(&mut self, stage: Stage, enabled: bool) {
        let config = StageConfig {
            enabled,
            job_name: stage.job_name().to_string(),
            runner: Runner::UbuntuLatest,
            steps: stage.default_steps(self.language, &self.version),
        };
        self.stages.insert(stage, config);
    }

    /// Enable a stage
    pub fn enable_stage(&mut self, stage: Stage) {
        if let Some(config) = self.stages.get_mut(&stage) {
            config.enabled = true;
        } else {
            self.add_stage_config(stage, true);
        }

        // Add tag trigger if enabling release stage
        if stage == Stage::Release {
            let has_tag_trigger = self.triggers.iter().any(|t| matches!(t, Trigger::Tag { .. }));
            if !has_tag_trigger {
                self.triggers.push(Trigger::Tag {
                    pattern: "v*".to_string(),
                });
            }
        }
    }

    /// Disable a stage
    pub fn disable_stage(&mut self, stage: Stage) {
        if let Some(config) = self.stages.get_mut(&stage) {
            config.enabled = false;
        }

        // Remove tag trigger if disabling release stage
        if stage == Stage::Release {
            self.triggers.retain(|t| !matches!(t, Trigger::Tag { .. }));
        }
    }

    /// Toggle a stage's enabled state
    pub fn toggle_stage(&mut self, stage: Stage) {
        if self.is_stage_enabled(stage) {
            self.disable_stage(stage);
        } else {
            self.enable_stage(stage);
        }
    }

    /// Check if a stage is enabled
    pub fn is_stage_enabled(&self, stage: Stage) -> bool {
        self.stages
            .get(&stage)
            .map(|config| config.enabled)
            .unwrap_or(false)
    }

    /// Get all enabled stages in order
    pub fn enabled_stages(&self) -> Vec<Stage> {
        Stage::all()
            .into_iter()
            .filter(|stage| self.is_stage_enabled(*stage))
            .collect()
    }

    /// Build the final Pipeline
    pub fn build(&self) -> Result<Pipeline> {
        let mut jobs = Vec::new();
        let mut previous_job: Option<String> = None;

        // Process stages in order, creating jobs for enabled stages
        for stage in Stage::all() {
            if let Some(config) = self.stages.get(&stage) {
                if config.enabled {
                    let needs = if stage == Stage::Test {
                        // Test job has no dependencies
                        Vec::new()
                    } else {
                        // Other jobs depend on the previous job
                        previous_job.clone().into_iter().collect()
                    };

                    let job = Job {
                        name: config.job_name.clone(),
                        runner: config.runner.clone(),
                        steps: config.steps.clone(),
                        needs,
                        timeout_minutes: None,
                        continue_on_error: false,
                    };

                    jobs.push(job);
                    previous_job = Some(config.job_name.clone());
                }
            }
        }

        Ok(Pipeline {
            name: self.name.clone(),
            triggers: self.triggers.clone(),
            jobs,
            env: self.env.clone(),
            cache: self.cache.clone(),
        })
    }

    /// Get the language
    pub fn language(&self) -> Language {
        self.language
    }

    /// Get the version
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Set the version
    pub fn set_version(&mut self, version: String) {
        self.version = version.clone();

        // Update version in all stage configurations
        for config in self.stages.values_mut() {
            for step in &mut config.steps {
                if let Step::SetupToolchain { version: v, .. } = step {
                    *v = version.clone();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_builder() {
        let builder = PipelineBuilder::new(Language::Rust, "stable");
        assert_eq!(builder.language(), Language::Rust);
        assert_eq!(builder.version(), "stable");
        assert_eq!(builder.name, "CI");
    }

    #[test]
    fn test_from_preset_rust_library() {
        let builder = PipelineBuilder::from_preset(
            PresetType::RustLibrary,
            &ProjectType::RustLibrary,
            "stable",
        );

        assert!(builder.is_stage_enabled(Stage::Test));
        assert!(builder.is_stage_enabled(Stage::Lint));
        assert!(builder.is_stage_enabled(Stage::Security));
        assert!(!builder.is_stage_enabled(Stage::Build));
        assert!(!builder.is_stage_enabled(Stage::Release));
    }

    #[test]
    fn test_from_preset_rust_binary() {
        let builder = PipelineBuilder::from_preset(
            PresetType::RustBinary,
            &ProjectType::RustBinary,
            "stable",
        );

        assert!(builder.is_stage_enabled(Stage::Test));
        assert!(builder.is_stage_enabled(Stage::Lint));
        assert!(builder.is_stage_enabled(Stage::Security));
        assert!(builder.is_stage_enabled(Stage::Build));
        assert!(!builder.is_stage_enabled(Stage::Release));
    }

    #[test]
    fn test_enable_disable_stage() {
        let mut builder = PipelineBuilder::new(Language::Rust, "stable");

        builder.enable_stage(Stage::Test);
        assert!(builder.is_stage_enabled(Stage::Test));

        builder.disable_stage(Stage::Test);
        assert!(!builder.is_stage_enabled(Stage::Test));
    }

    #[test]
    fn test_toggle_stage() {
        let mut builder = PipelineBuilder::new(Language::Rust, "stable");

        builder.enable_stage(Stage::Test);
        assert!(builder.is_stage_enabled(Stage::Test));

        builder.toggle_stage(Stage::Test);
        assert!(!builder.is_stage_enabled(Stage::Test));

        builder.toggle_stage(Stage::Test);
        assert!(builder.is_stage_enabled(Stage::Test));
    }

    #[test]
    fn test_build_creates_jobs() {
        let mut builder = PipelineBuilder::new(Language::Rust, "stable");
        builder.enable_stage(Stage::Test);
        builder.enable_stage(Stage::Lint);

        let pipeline = builder.build().unwrap();
        assert_eq!(pipeline.jobs.len(), 2);
        assert_eq!(pipeline.jobs[0].name, "test");
        assert_eq!(pipeline.jobs[1].name, "lint");

        // Lint should depend on test
        assert_eq!(pipeline.jobs[1].needs, vec!["test"]);
    }

    #[test]
    fn test_release_stage_adds_tag_trigger() {
        let mut builder = PipelineBuilder::new(Language::Rust, "stable");

        // Initially no tag trigger
        assert!(!builder.triggers.iter().any(|t| matches!(t, Trigger::Tag { .. })));

        builder.enable_stage(Stage::Release);

        // Should now have tag trigger
        assert!(builder.triggers.iter().any(|t| matches!(t, Trigger::Tag { .. })));

        builder.disable_stage(Stage::Release);

        // Tag trigger should be removed
        assert!(!builder.triggers.iter().any(|t| matches!(t, Trigger::Tag { .. })));
    }
}
