use super::{DetectionResult, ProjectDetector, ProjectType};
use crate::error::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct DockerDetector;

impl ProjectDetector for DockerDetector {
    fn detect(&self, path: &Path) -> Result<Option<DetectionResult>> {
        // Look for Dockerfile or common variations
        let dockerfile_patterns = vec![
            "Dockerfile",
            "dockerfile",
            "Dockerfile.dev",
            "Dockerfile.prod",
            "Dockerfile.build",
        ];

        let mut found_dockerfiles = Vec::new();
        let mut metadata = HashMap::new();

        // Check for standard Dockerfiles in the root
        for pattern in &dockerfile_patterns {
            let dockerfile_path = path.join(pattern);
            if dockerfile_path.exists() {
                found_dockerfiles.push(pattern.to_string());
            }
        }

        // Also check for docker-compose files
        let compose_files = vec![
            "docker-compose.yml",
            "docker-compose.yaml",
            "compose.yml",
            "compose.yaml",
        ];
        let mut has_compose = false;
        for compose_file in &compose_files {
            let compose_path = path.join(compose_file);
            if compose_path.exists() {
                has_compose = true;
                metadata.insert("compose_file".to_string(), compose_file.to_string());
                break;
            }
        }

        if found_dockerfiles.is_empty() && !has_compose {
            return Ok(None);
        }

        // Extract information from the first Dockerfile found
        if !found_dockerfiles.is_empty() {
            let primary_dockerfile = &found_dockerfiles[0];
            metadata.insert("dockerfile".to_string(), primary_dockerfile.clone());

            if found_dockerfiles.len() > 1 {
                metadata.insert(
                    "dockerfile_count".to_string(),
                    found_dockerfiles.len().to_string(),
                );
                metadata.insert("dockerfiles".to_string(), found_dockerfiles.join(", "));
            }

            // Try to extract base image from the Dockerfile
            if let Ok(contents) = fs::read_to_string(path.join(primary_dockerfile)) {
                if let Some(base_image) = extract_base_image(&contents) {
                    metadata.insert("base_image".to_string(), base_image);
                }
            }
        }

        if has_compose {
            metadata.insert("has_compose".to_string(), "yes".to_string());
        }

        Ok(Some(DetectionResult {
            project_type: ProjectType::DockerImage,
            language_version: None,
            metadata,
        }))
    }

    fn name(&self) -> &str {
        "Docker"
    }
}

/// Extract the base image from a Dockerfile
fn extract_base_image(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        // Look for FROM instruction
        if trimmed.to_uppercase().starts_with("FROM ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                // Return the image name (second word)
                // Handle multi-stage builds with "AS" keyword
                let image = parts[1];
                return Some(image.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_dockerfile() {
        let dir = tempdir().unwrap();
        let dockerfile = dir.path().join("Dockerfile");

        fs::write(
            &dockerfile,
            r#"
FROM rust:latest
WORKDIR /app
COPY . .
RUN cargo build --release
        "#,
        )
        .unwrap();

        let detector = DockerDetector;
        let result = detector.detect(dir.path()).unwrap().unwrap();

        assert_eq!(result.project_type, ProjectType::DockerImage);
        assert_eq!(result.metadata.get("dockerfile").unwrap(), "Dockerfile");
        assert_eq!(result.metadata.get("base_image").unwrap(), "rust:latest");
    }

    #[test]
    fn test_detect_dockerfile_variations() {
        let dir = tempdir().unwrap();

        // Create multiple Dockerfiles
        fs::write(dir.path().join("Dockerfile.dev"), "FROM node:18").unwrap();
        fs::write(dir.path().join("Dockerfile.prod"), "FROM node:18-alpine").unwrap();

        let detector = DockerDetector;
        let result = detector.detect(dir.path()).unwrap().unwrap();

        assert_eq!(result.project_type, ProjectType::DockerImage);
        assert_eq!(result.metadata.get("dockerfile_count").unwrap(), "2");
    }

    #[test]
    fn test_detect_docker_compose() {
        let dir = tempdir().unwrap();
        let compose = dir.path().join("docker-compose.yml");

        fs::write(
            &compose,
            r#"
version: '3'
services:
  web:
    build: .
        "#,
        )
        .unwrap();

        let detector = DockerDetector;
        let result = detector.detect(dir.path()).unwrap().unwrap();

        assert_eq!(result.project_type, ProjectType::DockerImage);
        assert_eq!(result.metadata.get("has_compose").unwrap(), "yes");
        assert_eq!(
            result.metadata.get("compose_file").unwrap(),
            "docker-compose.yml"
        );
    }

    #[test]
    fn test_detect_dockerfile_and_compose() {
        let dir = tempdir().unwrap();

        fs::write(dir.path().join("Dockerfile"), "FROM python:3.11").unwrap();
        fs::write(dir.path().join("docker-compose.yml"), "version: '3'").unwrap();

        let detector = DockerDetector;
        let result = detector.detect(dir.path()).unwrap().unwrap();

        assert_eq!(result.project_type, ProjectType::DockerImage);
        assert_eq!(result.metadata.get("dockerfile").unwrap(), "Dockerfile");
        assert_eq!(result.metadata.get("has_compose").unwrap(), "yes");
    }

    #[test]
    fn test_no_dockerfile() {
        let dir = tempdir().unwrap();
        let detector = DockerDetector;
        let result = detector.detect(dir.path()).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_extract_base_image() {
        let dockerfile = r#"
# This is a comment
FROM ubuntu:22.04
RUN apt-get update
        "#;

        let base = extract_base_image(dockerfile);
        assert_eq!(base, Some("ubuntu:22.04".to_string()));
    }

    #[test]
    fn test_extract_base_image_multistage() {
        let dockerfile = r#"
FROM golang:1.21 AS builder
WORKDIR /app
COPY . .
RUN go build

FROM alpine:latest
COPY --from=builder /app/app /app
        "#;

        // Should extract the first FROM
        let base = extract_base_image(dockerfile);
        assert_eq!(base, Some("golang:1.21".to_string()));
    }
}
