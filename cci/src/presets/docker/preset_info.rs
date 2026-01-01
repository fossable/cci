use crate::traits::PresetInfo;

use super::DockerPreset;

impl PresetInfo for DockerPreset {
    fn name(&self) -> &str {
        "docker"
    }

    fn description(&self) -> &str {
        "CI pipeline for building and pushing Docker images to registries"
    }
}
