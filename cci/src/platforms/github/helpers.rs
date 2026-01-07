use super::models::GitHubStep;
use serde_yaml::Value;
use std::collections::BTreeMap;

/// Helper functions for creating common GitHub Actions steps
impl GitHubStep {
    /// Create a checkout step using actions/checkout@v4
    pub fn checkout() -> Self {
        Self {
            name: Some("Checkout code".to_string()),
            uses: Some("actions/checkout@v4".to_string()),
            run: None,
            with: None,
            env: None,
        }
    }

    /// Create a step that uses an action
    pub fn action(name: impl Into<String>, uses: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            uses: Some(uses.into()),
            run: None,
            with: None,
            env: None,
        }
    }

    /// Create a step that uses an action with configuration
    pub fn action_with_config(
        name: impl Into<String>,
        uses: impl Into<String>,
        with: BTreeMap<String, Value>,
    ) -> Self {
        Self {
            name: Some(name.into()),
            uses: Some(uses.into()),
            run: None,
            with: Some(with),
            env: None,
        }
    }

    /// Create a step that runs a shell command
    pub fn run(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            uses: None,
            run: Some(command.into()),
            with: None,
            env: None,
        }
    }

    /// Setup Rust toolchain
    pub fn setup_rust(version: impl Into<String>) -> Self {
        Self::action_with_config(
            "Setup Rust",
            "actions-rust-lang/setup-rust-toolchain@v1",
            BTreeMap::from([("toolchain".to_string(), Value::String(version.into()))]),
        )
    }

    /// Setup Python
    pub fn setup_python(version: impl Into<String>) -> Self {
        Self::action_with_config(
            "Setup Python",
            "actions/setup-python@v5",
            BTreeMap::from([("python-version".to_string(), Value::String(version.into()))]),
        )
    }

    /// Setup Go
    pub fn setup_go(version: impl Into<String>) -> Self {
        Self::action_with_config(
            "Setup Go",
            "actions/setup-go@v5",
            BTreeMap::from([("go-version".to_string(), Value::String(version.into()))]),
        )
    }
}
