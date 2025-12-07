use clap::Parser;
use cci::cli::{Cli, Commands};
use cci::error::Result;
use colored::Colorize;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            preset,
            platform,
            yes,
            directory,
        } => {
            use cci::cli::write_with_confirmation;
            use cci::detection::DetectorRegistry;
            use cci::platforms::{
                CircleCIAdapter, GitHubAdapter, GitLabAdapter, JenkinsAdapter, PlatformAdapter,
            };
            use cci::presets::{go_app_preset, python_app_preset, rust_binary_preset, rust_library_preset};
            use colored::Colorize;

            println!("{}", "🚀 Initializing CI configuration".cyan().bold());
            println!();

            // Determine preset to use
            let preset_name = if let Some(p) = preset {
                p
            } else if !yes {
                // Auto-detect project type
                let registry = DetectorRegistry::new();
                match registry.detect(&directory) {
                    Ok(result) => {
                        println!("{}", format!("🔍 Detected: {}", result.project_type).green());
                        println!("{}", format!("   Confidence: {:.0}%", result.confidence * 100.0));
                        println!();

                        // Suggest preset based on detection
                        let suggested = match result.project_type {
                            cci::detection::ProjectType::RustLibrary => "rust-library",
                            cci::detection::ProjectType::RustBinary => "rust-binary",
                            cci::detection::ProjectType::RustWorkspace => "rust-library",
                            cci::detection::ProjectType::PythonApp |
                            cci::detection::ProjectType::PythonLibrary => "python-app",
                            cci::detection::ProjectType::GoApp |
                            cci::detection::ProjectType::GoLibrary => "go-app",
                        };

                        let confirm = inquire::Confirm::new(&format!("Use '{}' preset?", suggested))
                            .with_default(true)
                            .prompt();

                        match confirm {
                            Ok(true) => suggested.to_string(),
                            Ok(false) => {
                                let presets = vec!["rust-library", "rust-binary"];
                                inquire::Select::new("Select a preset:", presets)
                                    .prompt()
                                    .unwrap_or_else(|_| std::process::exit(1))
                                    .to_string()
                            }
                            Err(_) => std::process::exit(1),
                        }
                    }
                    Err(_) => {
                        eprintln!("Error: Could not detect project type");
                        eprintln!("Please specify a preset with --preset");
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!("Error: --yes requires --preset to be specified");
                std::process::exit(1);
            };

            // Get language version
            let version = if !yes {
                inquire::Text::new("Rust toolchain version?")
                    .with_default("stable")
                    .prompt()
                    .unwrap_or_else(|_| std::process::exit(1))
            } else {
                "stable".to_string()
            };

            // Create pipeline from preset
            let pipeline = match preset_name.as_str() {
                "rust-library" => rust_library_preset(&version),
                "rust-binary" => rust_binary_preset(&version),
                "python-app" => python_app_preset(&version),
                "go-app" => go_app_preset(&version),
                _ => {
                    eprintln!("Error: Unknown preset '{}'", preset_name);
                    eprintln!("Available: rust-library, rust-binary, python-app, go-app");
                    std::process::exit(1);
                }
            };

            println!("{}", format!("📦 Using preset: {}", preset_name).cyan());
            println!();

            // Determine platforms
            let platforms = if !platform.is_empty() {
                platform
            } else if !yes {
                let choices = vec!["github", "gitlab", "circleci", "jenkins"];
                inquire::MultiSelect::new("Select target platforms:", choices)
                    .with_default(&[0])
                    .prompt()
                    .unwrap_or_else(|_| std::process::exit(1))
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            } else {
                eprintln!("Error: --yes requires --platform to be specified");
                std::process::exit(1);
            };

            // Generate configs for each platform
            for plat in &platforms {
                let (config, output_path) = match plat.as_str() {
                    "github" => {
                        let adapter = GitHubAdapter;
                        let config = adapter.generate(&pipeline)?;
                        (config, directory.join(adapter.output_path()))
                    }
                    "gitlab" => {
                        let adapter = GitLabAdapter;
                        let config = adapter.generate(&pipeline)?;
                        (config, directory.join(adapter.output_path()))
                    }
                    "circleci" => {
                        let adapter = CircleCIAdapter;
                        let config = adapter.generate(&pipeline)?;
                        (config, directory.join(adapter.output_path()))
                    }
                    "jenkins" => {
                        let adapter = JenkinsAdapter;
                        let config = adapter.generate(&pipeline)?;
                        (config, directory.join(adapter.output_path()))
                    }
                    _ => {
                        eprintln!("Error: Unknown platform '{}'", plat);
                        std::process::exit(1);
                    }
                };

                write_with_confirmation(&output_path, &config, yes)?;
            }

            println!();
            println!("{}", "✨ CI configuration initialized successfully!".green().bold());
        }
        Commands::Generate {
            config,
            platform,
            output,
        } => {
            println!("Generate command - config: {:?}, platforms: {:?}, output: {:?}",
                config, platform, output);
            println!("Not yet implemented");
        }
        Commands::Detect { directory } => {
            use cci::detection::DetectorRegistry;

            let registry = DetectorRegistry::new();

            match registry.detect(&directory) {
                Ok(result) => {
                    println!("🔍 Project Detection Results\n");
                    println!("  Project Type: {}", result.project_type);
                    println!("  Confidence:   {:.0}%", result.confidence * 100.0);

                    if let Some(version) = result.language_version {
                        println!("  Version:      {}", version);
                    }

                    if !result.metadata.is_empty() {
                        println!("\n  Metadata:");
                        for (key, value) in result.metadata.iter() {
                            println!("    {}: {}", key, value);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Presets => {
            println!("Available presets:");
            println!();
            println!("  {}:", "Rust".cyan().bold());
            println!("    rust-library    - Rust library with tests, clippy, coverage");
            println!("    rust-binary     - Rust binary with builds and releases");
            println!();
            println!("  {}:", "Python".cyan().bold());
            println!("    python-app      - Python application with pytest, mypy, black");
            println!();
            println!("  {}:", "Go".cyan().bold());
            println!("    go-app          - Go application with tests, linting, security");
        }
        Commands::Platforms => {
            println!("Supported platforms:");
            println!("  github    - GitHub Actions");
            println!("  gitlab    - GitLab CI");
            println!("  circleci  - CircleCI");
            println!("  jenkins   - Jenkins");
        }
    }

    Ok(())
}
