use cci::cli::{Cli, Commands};
use cci::error::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Generate {
            config,
            output,
            force,
        }) => cci::cli::commands::handle_generate(&config, output, force),
        Some(Commands::Validate { config }) => cci::cli::commands::handle_validate(&config),
        Some(Commands::Tui { dir, platform }) => {
            cci::tui::run_with_args(&dir, platform)
        }
        None => {
            // Default to TUI if no command specified
            cci::tui::run_with_args(".", None)
        }
    }
}
