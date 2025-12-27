use super::{DetectionResult, ProjectDetector, ProjectType};
use crate::error::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct GoDetector;

impl ProjectDetector for GoDetector {
    fn detect(&self, path: &Path) -> Result<Option<DetectionResult>> {
        let go_mod = path.join("go.mod");

        if !go_mod.exists() {
            return Ok(None);
        }

        let mut metadata = HashMap::new();

        // Try to parse go.mod for version
        if let Ok(contents) = fs::read_to_string(&go_mod) {
            if let Some(line) = contents.lines().find(|l| l.starts_with("go ")) {
                let version = line.strip_prefix("go ").unwrap_or("1.21").trim();
                metadata.insert("go_version".to_string(), version.to_string());
            }
        }

        // Simple heuristic: apps have main.go in root or cmd/
        let is_app = path.join("main.go").exists() || path.join("cmd").is_dir();

        let project_type = if is_app {
            ProjectType::GoApp
        } else {
            ProjectType::GoLibrary
        };

        Ok(Some(DetectionResult {
            project_type,
            language_version: Some("1.21".to_string()),
            metadata,
        }))
    }

    fn name(&self) -> &str {
        "Go"
    }
}
