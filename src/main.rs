use cci::cli::{Cli, Commands};
use cci::error::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Generate {
            config,
            platform,
            force,
        }) => cci::cli::commands::handle_generate(&config, platform, force),
        Some(Commands::Validate { config }) => cci::cli::commands::handle_validate(&config),
        Some(Commands::Editor { dir }) => cci::editor::run_with_args(&dir, None),
        Some(Commands::Detect { dir }) => cci::cli::commands::handle_detect(&dir),
        _ => {
            // Default to editor if no command specified
            cci::editor::run_with_args(".", None)
        }
    }
}
