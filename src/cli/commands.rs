use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cci")]
#[command(about = "Generate CI/CD configurations for multiple platforms", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize CI configuration for a project
    Init {
        /// Use a preset template (e.g., rust-library, python-app)
        #[arg(short, long)]
        preset: Option<String>,

        /// Target platform(s) to generate for (comma-separated)
        #[arg(short = 'P', long, value_delimiter = ',')]
        platform: Vec<String>,

        /// Skip interactive prompts and use defaults
        #[arg(short, long)]
        yes: bool,

        /// Project directory (defaults to current)
        #[arg(short, long, default_value = ".")]
        directory: PathBuf,
    },

    /// Generate CI configuration from existing config file
    Generate {
        /// Input configuration file
        #[arg(short, long, default_value = "ci-config.toml")]
        config: PathBuf,

        /// Target platform(s) (comma-separated)
        #[arg(short = 'P', long, value_delimiter = ',')]
        platform: Vec<String>,

        /// Output directory
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },

    /// Detect project type and show information
    Detect {
        /// Project directory
        #[arg(default_value = ".")]
        directory: PathBuf,
    },

    /// List available preset templates
    Presets,

    /// List supported CI platforms
    Platforms,
}
