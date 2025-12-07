/// Common constants used throughout the codebase

// GitHub Actions
pub const GITHUB_UBUNTU_LATEST: &str = "ubuntu-latest";
pub const ACTION_CHECKOUT: &str = "actions/checkout@v4";
pub const ACTION_SETUP_PYTHON: &str = "actions/setup-python@v5";
pub const ACTION_SETUP_GO: &str = "actions/setup-go@v5";
pub const ACTION_SETUP_RUST: &str = "actions-rust-lang/setup-rust-toolchain@v1";

// Default git branches
pub const DEFAULT_BRANCHES: &[&str] = &["main", "master"];

// Default language versions
pub const DEFAULT_RUST_VERSION: &str = "stable";
pub const DEFAULT_GO_VERSION: &str = "1.21";
pub const DEFAULT_PYTHON_VERSION: &str = "3.11";
