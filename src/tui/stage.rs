use crate::detection::ProjectType;
use crate::models::step::{CoverageProvider, Language, Registry, Step};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Stage {
    Test,
    Lint,
    Security,
    Build,
    Release,
}

impl Stage {
    /// Get all stages in order
    pub fn all() -> Vec<Stage> {
        vec![
            Stage::Test,
            Stage::Lint,
            Stage::Security,
            Stage::Build,
            Stage::Release,
        ]
    }

    /// Get the name of this stage
    pub fn name(&self) -> &'static str {
        match self {
            Stage::Test => "Test",
            Stage::Lint => "Lint",
            Stage::Security => "Security",
            Stage::Build => "Build",
            Stage::Release => "Release",
        }
    }

    /// Get the description of this stage
    pub fn description(&self) -> &'static str {
        match self {
            Stage::Test => "Run tests with coverage",
            Stage::Lint => "Code quality checks",
            Stage::Security => "Security scans",
            Stage::Build => "Compilation and artifact creation",
            Stage::Release => "Publishing and releases",
        }
    }

    /// Check if this stage should be enabled by default for a project type
    pub fn default_enabled_for(&self, project_type: &ProjectType) -> bool {
        match (self, project_type) {
            (Stage::Test, _) => true,
            (Stage::Lint, _) => true,
            (Stage::Security, _) => true,
            (Stage::Build, ProjectType::RustLibrary) => false,
            (Stage::Build, ProjectType::PythonLibrary) => false,
            (Stage::Build, ProjectType::GoLibrary) => false,
            (Stage::Build, _) => true,
            (Stage::Release, _) => false,
        }
    }

    /// Get the job name for this stage
    pub fn job_name(&self) -> &'static str {
        match self {
            Stage::Test => "test",
            Stage::Lint => "lint",
            Stage::Security => "security",
            Stage::Build => "build",
            Stage::Release => "release",
        }
    }

    /// Get the steps for this stage based on language
    pub fn default_steps(&self, language: Language, version: &str) -> Vec<Step> {
        match self {
            Stage::Test => self.test_steps(language, version),
            Stage::Lint => self.lint_steps(language, version),
            Stage::Security => self.security_steps(language),
            Stage::Build => self.build_steps(language, version),
            Stage::Release => self.release_steps(language),
        }
    }

    fn test_steps(&self, language: Language, version: &str) -> Vec<Step> {
        vec![
            Step::Checkout,
            Step::SetupToolchain {
                language,
                version: version.to_string(),
            },
            Step::Cache {
                paths: Self::cache_paths(language),
                key: Self::cache_key(language),
            },
            Step::InstallDependencies { language },
            Step::RunTests {
                language,
                coverage: true,
            },
            Step::UploadCoverage {
                provider: CoverageProvider::Codecov,
            },
        ]
    }

    fn lint_steps(&self, language: Language, version: &str) -> Vec<Step> {
        let tool = match language {
            Language::Rust => "clippy",
            Language::Python => "flake8",
            Language::Go => "golangci-lint",
        };

        vec![
            Step::Checkout,
            Step::SetupToolchain {
                language,
                version: version.to_string(),
            },
            Step::Cache {
                paths: Self::cache_paths(language),
                key: Self::cache_key(language),
            },
            Step::RunLinter {
                language,
                tool: tool.to_string(),
            },
        ]
    }

    fn security_steps(&self, language: Language) -> Vec<Step> {
        let tool = match language {
            Language::Rust => "cargo-audit",
            Language::Python => "safety",
            Language::Go => "gosec",
        };

        vec![
            Step::Checkout,
            Step::SecurityScan {
                language,
                tool: tool.to_string(),
            },
        ]
    }

    fn build_steps(&self, language: Language, version: &str) -> Vec<Step> {
        let artifact_paths = match language {
            Language::Rust => vec!["target/release/*".to_string()],
            Language::Python => vec!["dist/*".to_string()],
            Language::Go => vec!["bin/*".to_string()],
        };

        vec![
            Step::Checkout,
            Step::SetupToolchain {
                language,
                version: version.to_string(),
            },
            Step::Cache {
                paths: Self::cache_paths(language),
                key: Self::cache_key(language),
            },
            Step::Build {
                language,
                artifact_paths: artifact_paths.clone(),
            },
            Step::UploadArtifact {
                name: "build-artifacts".to_string(),
                paths: artifact_paths,
            },
        ]
    }

    fn release_steps(&self, language: Language) -> Vec<Step> {
        let (registry, token_env) = match language {
            Language::Rust => (Registry::CratesIo, "CARGO_REGISTRY_TOKEN".to_string()),
            Language::Python => (Registry::PyPI, "PYPI_TOKEN".to_string()),
            Language::Go => (Registry::Npm, "NPM_TOKEN".to_string()), // Go uses module proxy, but for now use npm as placeholder
        };

        vec![
            Step::Checkout,
            Step::CreateRelease {
                tag_pattern: "v*".to_string(),
                artifacts: vec![],
            },
            Step::PublishPackage {
                registry,
                token_env,
            },
        ]
    }

    fn cache_paths(language: Language) -> Vec<String> {
        match language {
            Language::Rust => vec!["~/.cargo".to_string(), "target".to_string()],
            Language::Python => vec!["~/.cache/pip".to_string()],
            Language::Go => vec!["~/go/pkg/mod".to_string()],
        }
    }

    fn cache_key(language: Language) -> String {
        match language {
            Language::Rust => "cargo-${{ hashFiles('**/Cargo.lock') }}".to_string(),
            Language::Python => "pip-${{ hashFiles('**/requirements.txt') }}".to_string(),
            Language::Go => "go-${{ hashFiles('**/go.sum') }}".to_string(),
        }
    }
}

/// Helper to get language from project type
pub fn language_from_project(project_type: &ProjectType) -> Language {
    match project_type {
        ProjectType::RustLibrary | ProjectType::RustBinary | ProjectType::RustWorkspace => {
            Language::Rust
        }
        ProjectType::PythonApp | ProjectType::PythonLibrary => Language::Python,
        ProjectType::GoApp | ProjectType::GoLibrary => Language::Go,
    }
}

/// Helper to suggest default preset type from project type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetType {
    RustLibrary,
    RustBinary,
    PythonApp,
    GoApp,
}

impl PresetType {
    pub fn from_project_type(project_type: &ProjectType) -> Self {
        match project_type {
            ProjectType::RustLibrary => PresetType::RustLibrary,
            ProjectType::RustBinary | ProjectType::RustWorkspace => PresetType::RustBinary,
            ProjectType::PythonApp | ProjectType::PythonLibrary => PresetType::PythonApp,
            ProjectType::GoApp | ProjectType::GoLibrary => PresetType::GoApp,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            PresetType::RustLibrary => "Rust Library",
            PresetType::RustBinary => "Rust Binary",
            PresetType::PythonApp => "Python App",
            PresetType::GoApp => "Go App",
        }
    }

    pub fn all() -> Vec<PresetType> {
        vec![
            PresetType::RustLibrary,
            PresetType::RustBinary,
            PresetType::PythonApp,
            PresetType::GoApp,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_ordering() {
        let stages = Stage::all();
        assert_eq!(stages.len(), 5);
        assert_eq!(stages[0], Stage::Test);
        assert_eq!(stages[4], Stage::Release);
    }

    #[test]
    fn test_default_enabled() {
        assert!(Stage::Test.default_enabled_for(&ProjectType::RustLibrary));
        assert!(Stage::Lint.default_enabled_for(&ProjectType::RustBinary));
        assert!(!Stage::Build.default_enabled_for(&ProjectType::RustLibrary));
        assert!(Stage::Build.default_enabled_for(&ProjectType::RustBinary));
        assert!(!Stage::Release.default_enabled_for(&ProjectType::RustBinary));
    }

    #[test]
    fn test_language_from_project() {
        assert_eq!(
            language_from_project(&ProjectType::RustLibrary),
            Language::Rust
        );
        assert_eq!(
            language_from_project(&ProjectType::PythonApp),
            Language::Python
        );
        assert_eq!(language_from_project(&ProjectType::GoApp), Language::Go);
    }
}
