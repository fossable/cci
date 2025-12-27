use super::{DetectionResult, ProjectDetector};
use super::docker::DockerDetector;
use super::go::GoDetector;
use super::python::PythonDetector;
use super::rust::RustDetector;
use crate::error::{detection_failed_error, Result};
use std::path::Path;

/// Registry for all project detectors
pub struct DetectorRegistry {
    detectors: Vec<Box<dyn ProjectDetector>>,
}

impl DetectorRegistry {
    /// Create a new registry with all built-in detectors
    pub fn new() -> Self {
        let mut registry = Self {
            detectors: Vec::new(),
        };

        // Register built-in detectors
        // Order matters: more specific detectors first
        registry.register(Box::new(RustDetector));
        registry.register(Box::new(PythonDetector));
        registry.register(Box::new(GoDetector));
        registry.register(Box::new(DockerDetector));

        registry
    }

    /// Register a new detector
    pub fn register(&mut self, detector: Box<dyn ProjectDetector>) {
        self.detectors.push(detector);
    }

    /// Detect project type in the given path
    /// Returns the first detection result found
    pub fn detect(&self, path: &Path) -> Result<DetectionResult> {
        for detector in &self.detectors {
            if let Some(result) = detector.detect(path)? {
                return Ok(result);
            }
        }

        Err(detection_failed_error())
    }

    /// Get all registered detector names
    pub fn detector_names(&self) -> Vec<&str> {
        self.detectors.iter().map(|d| d.name()).collect()
    }
}

impl Default for DetectorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_registry_rust_detection() {
        let dir = tempdir().unwrap();
        let cargo_toml = dir.path().join("Cargo.toml");

        fs::write(&cargo_toml, r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"

[lib]
        "#).unwrap();

        let registry = DetectorRegistry::new();
        let _result = registry.detect(dir.path()).unwrap();
    }

    #[test]
    fn test_registry_no_match() {
        let dir = tempdir().unwrap();
        let registry = DetectorRegistry::new();
        let result = registry.detect(dir.path());

        assert!(result.is_err());
    }
}
