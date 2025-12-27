use crate::config::{preset_choice_to_config, CciConfig};
use crate::editor::registry::build_registry;
use crate::error::Result;
use crate::generator::MultiPresetGenerator;
use anyhow::{bail, Context};
use colored::Colorize;
use std::path::PathBuf;
use std::sync::Arc;

/// Handle the generate command
pub fn handle_generate(config_path: &str, platform_arg: Option<String>, force: bool) -> Result<()> {
    use crate::detection::DetectorRegistry;
    use crate::editor::state::Platform;

    // 1. Load and parse RON
    println!("{} {}", "Loading".cyan().bold(), config_path);
    let ron_str = std::fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path))?;

    // 2. Parse RON
    println!("{}", "Parsing RON configuration...".cyan().bold());
    let config: CciConfig = ron::from_str(&ron_str)
        .with_context(|| "Failed to parse RON config. Check syntax and structure.")?;

    if config.presets.is_empty() {
        bail!("No presets defined in configuration file");
    }

    // 3. Detect project and determine platform
    let working_dir = std::path::PathBuf::from(".");
    let detector_registry = DetectorRegistry::new();
    let detection = detector_registry.detect(&working_dir)?;

    let platform = if let Some(p) = platform_arg {
        match p.to_lowercase().as_str() {
            "github" => Platform::GitHub,
            "gitlab" => Platform::GitLab,
            "circleci" => Platform::CircleCI,
            "jenkins" => Platform::Jenkins,
            _ => Platform::GitHub,
        }
    } else {
        Platform::GitHub // Default platform
    };

    println!(
        "{} {} preset(s) for platform {}",
        "Found".green().bold(),
        config.presets.len(),
        format!("{:?}", platform).yellow()
    );

    // 4. Convert to preset configs
    let registry = Arc::new(build_registry());
    let mut preset_configs = Vec::new();

    for preset_choice in config.presets {
        let (preset_id, preset_config) = preset_choice_to_config(preset_choice)
            .with_context(|| "Failed to convert preset configuration")?;
        println!("  {} {}", "•".blue(), preset_id);
        preset_configs.push((preset_id, preset_config));
    }

    // 5. Generate outputs
    println!("\n{}", "Generating CI configurations...".cyan().bold());
    let language_version = detection
        .language_version
        .unwrap_or_else(|| "stable".to_string());

    let generator = MultiPresetGenerator::new(
        preset_configs,
        registry,
        platform,
        language_version,
    );

    let outputs = generator
        .generate_all()
        .with_context(|| "Failed to generate CI configurations")?;

    // 6. Write files
    let base_path = PathBuf::from(".");

    for (filename, content) in outputs {
        // Use the full path (includes subdirectories like .github/workflows)
        let output_path = base_path.join(&filename);

        // Check if file exists and force flag
        if output_path.exists() && !force {
            bail!(
                "File exists: {}. Use --force to overwrite",
                output_path.display()
            );
        }

        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create directory: {}", parent.display())
            })?;
        }

        // Write file
        std::fs::write(&output_path, content)
            .with_context(|| format!("Failed to write file: {}", output_path.display()))?;

        println!(
            "  {} {}",
            "✓".green().bold(),
            output_path.display().to_string().yellow()
        );
    }

    println!("\n{}", "Done!".green().bold());
    Ok(())
}

/// Handle the validate command
pub fn handle_validate(config_path: &str) -> Result<()> {
    println!("{} {}", "Validating".cyan().bold(), config_path);

    // Load file
    let ron_str = std::fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path))?;

    // Parse RON
    let config: CciConfig = ron::from_str(&ron_str).with_context(|| {
        "Failed to parse RON config. Check syntax and structure:\n\
         - Ensure all fields are properly formatted\n\
         - Check for missing commas\n\
         - Verify enum variants match expected values"
    })?;

    // Basic validation
    if config.presets.is_empty() {
        bail!("Validation failed: No presets defined in configuration");
    }

    println!("\n{}", "Configuration is valid!".green().bold());
    println!("  Presets: {}", config.presets.len());

    for (idx, preset) in config.presets.iter().enumerate() {
        let preset_name = match preset {
            crate::config::PresetChoice::Python(_) => "Python",
            crate::config::PresetChoice::RustLibrary(_) => "Rust Library",
            crate::config::PresetChoice::RustBinary(_) => "Rust Binary",
            crate::config::PresetChoice::GoApp(_) => "Go App",
            crate::config::PresetChoice::Docker(_) => "Docker",
        };
        println!("    {}. {}", idx + 1, preset_name);
    }

    Ok(())
}
