# CLAUDE.md - Development Guide

This file contains instructions for AI assistants working on the common-ci
project.

## Project Overview

`cci` is a tool that generates CI/CD configurations for multiple platforms
(GitHub Actions, GitLab CI, CircleCI, Jenkins) from a common specification. It
uses Rust structs as "presets" that can be composed into a final template.

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

- Follow Rust idioms and conventions
- Use `clippy` for linting: `cargo clippy`
- Format code: `cargo fmt`
- Keep functions focused and testable
- Document public APIs with doc comments
- Prefer simple, readable code over clever optimizations
