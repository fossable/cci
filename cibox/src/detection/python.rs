use super::{DetectionResult, ProjectDetector, ProjectType};
use crate::error::Result;
use std::collections::HashMap;
use std::path::Path;

pub struct PythonDetector;

impl ProjectDetector for PythonDetector {
    fn detect(&self, path: &Path) -> Result<Option<DetectionResult>> {
        let has_pyproject = path.join("pyproject.toml").exists();
        let has_setup = path.join("setup.py").exists();
        let has_requirements = path.join("requirements.txt").exists();

        if !has_pyproject && !has_setup && !has_requirements {
            return Ok(None);
        }

        let mut metadata = HashMap::new();

        if has_pyproject {
            metadata.insert("config".to_string(), "pyproject.toml".to_string());
        } else if has_setup {
            metadata.insert("config".to_string(), "setup.py".to_string());
        }

        // Simple heuristic: apps have main.py or __main__.py
        let is_app = path.join("main.py").exists() || path.join("__main__.py").exists();

        let project_type = if is_app {
            ProjectType::PythonApp
        } else {
            ProjectType::PythonLibrary
        };

        Ok(Some(DetectionResult {
            project_type,
            language_version: Some("3.11".to_string()),
            metadata,
        }))
    }

    fn name(&self) -> &str {
        "Python"
    }
}
