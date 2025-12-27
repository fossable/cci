use crate::config::ron_types::*;
use crate::editor::config::{OptionValue, PresetConfig};
use crate::error::Result;
use anyhow::anyhow;

/// Convert a PresetChoice to (preset_id, PresetConfig) for editor use
pub fn preset_choice_to_config(choice: PresetChoice) -> Result<(String, PresetConfig)> {
    match choice {
        PresetChoice::Python(config) => {
            let preset_config = python_config_to_preset_config(config)?;
            Ok(("python-app".to_string(), preset_config))
        }
        PresetChoice::RustLibrary(config) => {
            let preset_config = rust_library_config_to_preset_config(config)?;
            Ok(("rust-library".to_string(), preset_config))
        }
        PresetChoice::RustBinary(config) => {
            let preset_config = rust_binary_config_to_preset_config(config)?;
            Ok(("rust-binary".to_string(), preset_config))
        }
        PresetChoice::GoApp(config) => {
            let preset_config = go_app_config_to_preset_config(config)?;
            Ok(("go-app".to_string(), preset_config))
        }
        PresetChoice::Docker(config) => {
            let preset_config = docker_config_to_preset_config(config)?;
            Ok(("docker".to_string(), preset_config))
        }
    }
}

/// Convert a (preset_id, PresetConfig) back to PresetChoice for saving
pub fn preset_config_to_choice(preset_id: &str, config: &PresetConfig) -> Result<PresetChoice> {
    match preset_id {
        "python-app" => {
            let python_config = preset_config_to_python_config(config)?;
            Ok(PresetChoice::Python(python_config))
        }
        "rust-library" => {
            let rust_config = preset_config_to_rust_library_config(config)?;
            Ok(PresetChoice::RustLibrary(rust_config))
        }
        "rust-binary" => {
            let rust_config = preset_config_to_rust_binary_config(config)?;
            Ok(PresetChoice::RustBinary(rust_config))
        }
        "go-app" => {
            let go_config = preset_config_to_go_app_config(config)?;
            Ok(PresetChoice::GoApp(go_config))
        }
        "docker" => {
            let docker_config = preset_config_to_docker_config(config)?;
            Ok(PresetChoice::Docker(docker_config))
        }
        _ => Err(anyhow!("Unknown preset ID: {}", preset_id)),
    }
}

// =============================================================================
// Python Conversions
// =============================================================================

fn python_config_to_preset_config(config: PythonConfig) -> Result<PresetConfig> {
    let mut preset_config = PresetConfig::new("python-app".to_string());

    preset_config.set("python_version".to_string(), OptionValue::String(config.version));
    preset_config.set("enable_linter".to_string(), OptionValue::Bool(config.linter.enabled));
    preset_config.set(
        "linter_tool".to_string(),
        OptionValue::Enum {
            selected: config.linter.tool.as_str().to_string(),
            variants: vec!["flake8".to_string(), "ruff".to_string()],
        },
    );
    preset_config.set("enable_formatter".to_string(), OptionValue::Bool(config.formatter.enabled));
    preset_config.set(
        "formatter_tool".to_string(),
        OptionValue::Enum {
            selected: config.formatter.tool.as_str().to_string(),
            variants: vec!["black".to_string(), "ruff".to_string()],
        },
    );
    preset_config.set("type_check".to_string(), OptionValue::Bool(config.type_check));

    Ok(preset_config)
}

fn preset_config_to_python_config(config: &PresetConfig) -> Result<PythonConfig> {
    let version = config
        .get_string("python_version")
        .unwrap_or_else(|| "3.11".to_string());

    let linter_enabled = config.get_bool("enable_linter");
    let linter_tool = match config.get_enum("linter_tool").as_deref() {
        Some("ruff") => LinterTool::Ruff,
        _ => LinterTool::Flake8,
    };

    let formatter_enabled = config.get_bool("enable_formatter");
    let formatter_tool = match config.get_enum("formatter_tool").as_deref() {
        Some("ruff") => FormatterTool::Ruff,
        _ => FormatterTool::Black,
    };

    let type_check = config.get_bool("type_check");

    Ok(PythonConfig {
        version,
        linter: LinterConfig {
            enabled: linter_enabled,
            tool: linter_tool,
        },
        formatter: FormatterConfig {
            enabled: formatter_enabled,
            tool: formatter_tool,
        },
        type_check,
    })
}

// =============================================================================
// Rust Library Conversions
// =============================================================================

fn rust_library_config_to_preset_config(config: RustLibraryConfig) -> Result<PresetConfig> {
    let mut preset_config = PresetConfig::new("rust-library".to_string());

    preset_config.set("rust_version".to_string(), OptionValue::String(config.version));
    preset_config.set("enable_coverage".to_string(), OptionValue::Bool(config.coverage));
    preset_config.set("enable_linter".to_string(), OptionValue::Bool(config.linter));
    preset_config.set("enable_security".to_string(), OptionValue::Bool(config.security));
    preset_config.set("enable_formatter".to_string(), OptionValue::Bool(config.formatter));

    Ok(preset_config)
}

fn preset_config_to_rust_library_config(config: &PresetConfig) -> Result<RustLibraryConfig> {
    let version = config
        .get_string("rust_version")
        .unwrap_or_else(|| "stable".to_string());

    Ok(RustLibraryConfig {
        version,
        coverage: config.get_bool("enable_coverage"),
        linter: config.get_bool("enable_linter"),
        security: config.get_bool("enable_security"),
        formatter: config.get_bool("enable_formatter"),
    })
}

// =============================================================================
// Rust Binary Conversions
// =============================================================================

fn rust_binary_config_to_preset_config(config: RustBinaryConfig) -> Result<PresetConfig> {
    let mut preset_config = PresetConfig::new("rust-binary".to_string());

    preset_config.set("rust_version".to_string(), OptionValue::String(config.version));
    preset_config.set("enable_linter".to_string(), OptionValue::Bool(config.linter));
    preset_config.set("enable_security".to_string(), OptionValue::Bool(config.security));
    preset_config.set("enable_formatter".to_string(), OptionValue::Bool(config.formatter));
    preset_config.set("build_release".to_string(), OptionValue::Bool(config.build_release));

    Ok(preset_config)
}

fn preset_config_to_rust_binary_config(config: &PresetConfig) -> Result<RustBinaryConfig> {
    let version = config
        .get_string("rust_version")
        .unwrap_or_else(|| "stable".to_string());

    Ok(RustBinaryConfig {
        version,
        linter: config.get_bool("enable_linter"),
        security: config.get_bool("enable_security"),
        formatter: config.get_bool("enable_formatter"),
        build_release: config.get_bool("build_release"),
    })
}

// =============================================================================
// Go App Conversions
// =============================================================================

fn go_app_config_to_preset_config(config: GoAppConfig) -> Result<PresetConfig> {
    let mut preset_config = PresetConfig::new("go-app".to_string());

    preset_config.set("go_version".to_string(), OptionValue::String(config.version));
    preset_config.set("enable_linter".to_string(), OptionValue::Bool(config.linter));
    preset_config.set("enable_security".to_string(), OptionValue::Bool(config.security));

    Ok(preset_config)
}

fn preset_config_to_go_app_config(config: &PresetConfig) -> Result<GoAppConfig> {
    let version = config
        .get_string("go_version")
        .unwrap_or_else(|| "1.21".to_string());

    Ok(GoAppConfig {
        version,
        linter: config.get_bool("enable_linter"),
        security: config.get_bool("enable_security"),
    })
}

// =============================================================================
// Docker Conversions
// =============================================================================

fn docker_config_to_preset_config(config: DockerConfig) -> Result<PresetConfig> {
    let mut preset_config = PresetConfig::new("docker".to_string());

    preset_config.set("image_name".to_string(), OptionValue::String(config.image_name));
    preset_config.set(
        "registry".to_string(),
        OptionValue::Enum {
            selected: config.registry.as_str().to_string(),
            variants: vec!["none".to_string(), "dockerhub".to_string(), "github".to_string()],
        },
    );
    preset_config.set("dockerfile_path".to_string(), OptionValue::String(config.dockerfile_path));
    preset_config.set("build_context".to_string(), OptionValue::String(config.build_context));
    preset_config.set("enable_cache".to_string(), OptionValue::Bool(config.cache));
    preset_config.set("push_on_tags_only".to_string(), OptionValue::Bool(config.tags_only));

    Ok(preset_config)
}

fn preset_config_to_docker_config(config: &PresetConfig) -> Result<DockerConfig> {
    let image_name = config
        .get_string("image_name")
        .unwrap_or_else(|| "myapp".to_string());

    let registry = match config.get_enum("registry").as_deref() {
        Some("dockerhub") => DockerRegistryChoice::DockerHub,
        Some("github") => DockerRegistryChoice::GitHubRegistry,
        _ => DockerRegistryChoice::None,
    };

    let dockerfile_path = config
        .get_string("dockerfile_path")
        .unwrap_or_else(|| "./Dockerfile".to_string());

    let build_context = config
        .get_string("build_context")
        .unwrap_or_else(|| ".".to_string());

    Ok(DockerConfig {
        image_name,
        registry,
        dockerfile_path,
        build_context,
        build_args: vec![],
        cache: config.get_bool("enable_cache"),
        tags_only: config.get_bool("push_on_tags_only"),
    })
}
