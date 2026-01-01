use crate::traits::PresetInfo;

use super::GoAppPreset;

impl PresetInfo for GoAppPreset {
    fn name(&self) -> &str {
        "go-app"
    }

    fn description(&self) -> &str {
        "CI pipeline for Go applications with testing, linting, and security scanning"
    }
}
