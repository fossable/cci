use crate::traits::PresetInfo;

use super::RustPreset;

impl PresetInfo for RustPreset {
    fn name(&self) -> &str {
        "rust"
    }

    fn description(&self) -> &str {
        "CI pipeline for Rust projects (binaries, libraries, and workspaces)"
    }
}
