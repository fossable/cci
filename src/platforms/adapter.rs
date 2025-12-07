use crate::error::Result;
use crate::models::Pipeline;
use std::path::PathBuf;

/// Trait for platform-specific CI configuration adapters
pub trait PlatformAdapter {
    /// Platform-specific intermediate representation type
    type IR;

    /// Transform a generic pipeline to platform-specific IR
    fn transform(&self, pipeline: &Pipeline) -> Result<Self::IR>;

    /// Serialize the IR to the final config format (YAML, Groovy, etc.)
    fn serialize(&self, ir: &Self::IR) -> Result<String>;

    /// Get the output file path for this platform
    fn output_path(&self) -> PathBuf;

    /// Validate the generated config (optional, default is no-op)
    fn validate(&self, _config: &str) -> Result<()> {
        Ok(())
    }

    /// Generate the complete config file content from a pipeline
    fn generate(&self, pipeline: &Pipeline) -> Result<String> {
        let ir = self.transform(pipeline)?;
        let config = self.serialize(&ir)?;
        self.validate(&config)?;
        Ok(config)
    }
}
