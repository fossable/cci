pub mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cci")]
#[command(about = "Control your CI/CD configuration")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate CI config from RON file
    Generate {
        /// Path to cci.ron config file
        #[arg(default_value = "cci.ron")]
        config: String,

        /// Target platform
        #[arg(short, long)]
        platform: Option<String>,

        /// Force overwrite existing files
        #[arg(short, long)]
        force: bool,
    },

    /// Validate RON config syntax
    Validate {
        /// Path to cci.ron config file
        #[arg(default_value = "cci.ron")]
        config: String,
    },

    /// Run interactive editor (default)
    Editor {
        /// Project directory
        #[arg(short, long, default_value = ".")]
        dir: String,
    },

    /// Detect project type and matching presets
    Detect {
        /// Project directory
        #[arg(short, long, default_value = ".")]
        dir: String,
    },
}
