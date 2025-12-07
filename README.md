<p align="center">
	<img src="https://raw.githubusercontent.com/fossable/fossable/master/emblems/cci.svg" style="width:90%; height:auto;"/>
</p>

![License](https://img.shields.io/github/license/fossable/cci)
![Build](https://github.com/fossable/cci/actions/workflows/test.yml/badge.svg)
![GitHub repo size](https://img.shields.io/github/repo-size/fossable/cci)
![Stars](https://img.shields.io/github/stars/fossable/cci?style=social)

<hr>

**cci** (common-ci) is a tool that generates CI/CD configurations for popular
platforms like Github Actions and Gitlab CI. Imagine Terraform, but for CI
pipelines.

There are three main advantages to generating your CI workflows/pipelines:

- You can get started really quickly for projects in popular ecosystems
- You don't have to write any Yaml
- You're not locked into a single CI platform because

The downside is, of course, you don't get the full flexiblity of writing your
own configuration.

## Available Presets

### Rust

- **rust-library** - Comprehensive CI for Rust libraries
  - Tests with coverage (tarpaulin + codecov)
  - Linting (clippy)
  - Formatting checks (rustfmt)
  - Security scanning (cargo-audit)

- **rust-binary** - CI for Rust binaries
  - All features from rust-library
  - Build job with artifact upload
  - Automated releases on tags

### Python

- **python-app** - Python applications
  - Tests with coverage (pytest + codecov)
  - Type checking (mypy)
  - Code formatting (black)
  - Security scanning (safety)

### Go

- **go-app** - Go applications
  - Tests with coverage
  - Linting (golangci-lint)
  - Security scanning (gosec)
