use roniker::RustAnalyzer;
use std::env;
use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut analyzer = RustAnalyzer::with_root_type("crate::config::ron_types::CiboxConfig");

    analyzer
        .add_file(&root.join("src/config/ron_types.rs"))
        .expect("Failed to parse ron_types.rs");

    for dir in ["rust", "python", "go", "docker"] {
        let path = root.join(format!("src/presets/{dir}/mod.rs"));
        analyzer
            .add_file(&path)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {e}", path.display()));
    }

    // The preset structs use container-level #[serde(default)] with manual
    // Default impls, which roniker's #[derive(Default)] detection misses —
    // mark them defaulted so the LSP doesn't flag omitted fields as missing.
    for name in [
        "crate::presets::rust::Rust",
        "crate::presets::python::PythonApp",
        "crate::presets::go::GoApp",
        "crate::presets::docker::Docker",
    ] {
        let mut info = analyzer
            .get_type_info(name)
            .unwrap_or_else(|| panic!("preset type {name} not found"))
            .clone();
        info.has_default = true;
        analyzer.add_type(info);
    }

    let dest = PathBuf::from(env::var("OUT_DIR").unwrap()).join("rust_analyzer.json");
    let json = serde_json::to_string(&analyzer).expect("Failed to serialize RustAnalyzer");
    std::fs::write(&dest, json).expect("Failed to write rust_analyzer.json");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/config/ron_types.rs");
    println!("cargo:rerun-if-changed=src/presets");
}
