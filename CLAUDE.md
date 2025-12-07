# CLAUDE.md - Development Guide

This file contains instructions for AI assistants working on the common-ci
project.

## Project Overview

`cci` is a tool that generates CI/CD configurations for multiple platforms
(GitHub Actions, GitLab CI, CircleCI, Jenkins) from a common specification. It
uses Rust structs as a single source of truth, transforming them via
platform-specific adapters into the appropriate configuration format.

**Architecture:**

```
Preset → Generic Pipeline → Platform Adapter → Platform IR → YAML/Config File
```

**No templates** - everything uses Rust structs + serde serialization.

## Maintenance Tasks

### Checking CI Platform Models for Upstream Changes

The platform models in `src/platforms/*/models.rs` need periodic updates to stay
synchronized with upstream CI platform specifications.

**Platforms to monitor:**

- GitHub Actions - `.github/workflows/*.yml` syntax
- GitLab CI - `.gitlab-ci.yml` syntax
- CircleCI - `.circleci/config.yml` syntax
- Jenkins - `Jenkinsfile` declarative pipeline syntax

**Update procedure:**

1. **Check official documentation for changes:**
   - [GitHub Actions workflow syntax](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions)
   - [GitLab CI YAML reference](https://docs.gitlab.com/ee/ci/yaml/)
   - [CircleCI configuration reference](https://circleci.com/docs/configuration-reference/)
   - [Jenkins pipeline syntax](https://www.jenkins.io/doc/book/pipeline/syntax/)

2. **Review platform models that need updates:**

3. **Common changes to watch for:**
   - **Action/feature version updates** (e.g., `actions/checkout@v4` → `@v5`)
   - **New workflow syntax** features or keywords
   - **Deprecated features** that need migration paths
   - **New built-in capabilities** (caching, artifacts, matrix builds)
   - **Security updates** to actions, images, or runners
   - **New step types** or job configuration options

4. **Update process:**
   - Update IR models in `models.rs` to support new features
   - Update transformation logic in `adapter.rs` to handle new Step variants
   - Add tests for new functionality
   - Update examples if needed
   - Run full test suite: `cargo test`
   - Test example generation: `cargo run --example generate_github`

5. **Recommended update frequency:**
   - **Quarterly** for routine checks
   - **Immediately** when major platform updates are announced
   - **As needed** when users report compatibility issues

## Development Guidelines

### Adding a New Platform

1. Create platform directory: `src/platforms/{platform}/`
2. Define IR models in `models.rs` with serde derives
3. Implement `PlatformAdapter` trait in `adapter.rs`
4. Add transformation logic for all `Step` enum variants
5. Write tests for transformation and serialization
6. Export in `src/platforms/mod.rs`

### Adding a New Preset

1. Create preset function in appropriate file: `src/presets/{language}.rs`
2. Return a `Pipeline` struct with appropriate jobs and steps
3. Add tests to verify the preset structure
4. Export in `src/presets/mod.rs`
5. Update CLI `presets` command to list it

### Adding a New Language

1. Add variant to `Language` enum in `src/models/step.rs`
2. Update all platform adapters to handle the new language in:
   - `SetupToolchain` step
   - `InstallDependencies` step
   - `RunTests` step
   - `RunLinter` step
   - `SecurityScan` step
   - `Build` step
3. Create detector in `src/detection/{language}.rs`
4. Register detector in `DetectorRegistry`
5. Add presets in `src/presets/{language}.rs`

## Testing Strategy

- **Unit tests**: Test individual components (models, transformations)
- **Integration tests**: Test full pipeline generation
- **Example programs**: Demonstrate end-to-end functionality
- **Fixture tests**: Use real project structures for detection testing

Run tests: `cargo test` Run examples: `cargo run --example generate_github`

## Code Quality

- Follow Rust idioms and conventions
- Use `clippy` for linting: `cargo clippy`
- Format code: `cargo fmt`
- Keep functions focused and testable
- Document public APIs with doc comments
- Prefer simple, readable code over clever optimizations
