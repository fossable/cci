use super::{DetectionResult, ProjectDetector, ProjectType};
use crate::error::{cargo_toml_error, Result};
use cargo_toml::Manifest;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct RustDetector;

impl ProjectDetector for RustDetector {
    fn detect(&self, path: &Path) -> Result<Option<DetectionResult>> {
        let cargo_toml_path = path.join("Cargo.toml");

        if !cargo_toml_path.exists() {
            return Ok(None);
        }

        // Try to parse Cargo.toml
        let contents = fs::read_to_string(&cargo_toml_path)?;
        let manifest = Manifest::from_str(&contents)
            .map_err(|e| cargo_toml_error(e.to_string()))?;

        let mut metadata = HashMap::new();

        // Check if it's a workspace
        if let Some(workspace) = &manifest.workspace {
            metadata.insert("type".to_string(), "workspace".to_string());

            let members = &workspace.members;
            metadata.insert("members".to_string(), members.join(", "));
            metadata.insert("member_count".to_string(), members.len().to_string());

            return Ok(Some(DetectionResult {
                project_type: ProjectType::RustWorkspace,
                language_version: extract_rust_version(&manifest),
                metadata,
            }));
        }

        // Determine if library or binary
        let project_type = if has_library(&manifest) {
            metadata.insert("type".to_string(), "library".to_string());
            ProjectType::RustLibrary
        } else if has_binary(&manifest) {
            metadata.insert("type".to_string(), "binary".to_string());
            ProjectType::RustBinary
        } else {
            // Default to binary if unclear
            metadata.insert("type".to_string(), "binary (assumed)".to_string());
            ProjectType::RustBinary
        };

        // Extract package name
        if let Some(package) = &manifest.package {
            metadata.insert("name".to_string(), package.name().to_string());
            // For simplicity, skip edition extraction due to Inheritable complexity
        }

        Ok(Some(DetectionResult {
            project_type,
            language_version: extract_rust_version(&manifest),
            metadata,
        }))
    }

    fn name(&self) -> &str {
        "Rust"
    }
}

fn has_library(manifest: &Manifest) -> bool {
    manifest.lib.is_some()
}

fn has_binary(manifest: &Manifest) -> bool {
    !manifest.bin.is_empty() || manifest.package.as_ref()
        .and_then(|p| p.default_run.as_ref())
        .is_some()
}

fn extract_rust_version(_manifest: &Manifest) -> Option<String> {
    // Default to stable - rust_version extraction is complex with Inheritable
    Some("stable".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_rust_library() {
        let dir = tempdir().unwrap();
        let cargo_toml = dir.path().join("Cargo.toml");

        fs::write(&cargo_toml, r#"
[package]
name = "test-lib"
version = "0.1.0"
edition = "2021"

[lib]
name = "test_lib"
        "#).unwrap();

        let detector = RustDetector;
        let result = detector.detect(dir.path()).unwrap().unwrap();

        assert_eq!(result.project_type, ProjectType::RustLibrary);
        assert_eq!(result.metadata.get("name").unwrap(), "test-lib");
    }

    #[test]
    fn test_detect_rust_binary() {
        let dir = tempdir().unwrap();
        let cargo_toml = dir.path().join("Cargo.toml");

        fs::write(&cargo_toml, r#"
[package]
name = "test-bin"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "test-bin"
        "#).unwrap();

        let detector = RustDetector;
        let result = detector.detect(dir.path()).unwrap().unwrap();

        assert_eq!(result.project_type, ProjectType::RustBinary);
    }

    #[test]
    fn test_detect_rust_workspace() {
        let dir = tempdir().unwrap();
        let cargo_toml = dir.path().join("Cargo.toml");

        fs::write(&cargo_toml, r#"
[workspace]
members = ["crate1", "crate2"]
        "#).unwrap();

        let detector = RustDetector;
        let result = detector.detect(dir.path()).unwrap().unwrap();

        assert_eq!(result.project_type, ProjectType::RustWorkspace);
        assert_eq!(result.metadata.get("member_count").unwrap(), "2");
    }

    #[test]
    fn test_no_cargo_toml() {
        let dir = tempdir().unwrap();
        let detector = RustDetector;
        let result = detector.detect(dir.path()).unwrap();

        assert!(result.is_none());
    }
}
