use cci::presets::rust::RustLibraryPreset;
use cci::traits::ToGitHub;

fn main() {
    // Create a Rust library preset using the builder pattern
    let preset = RustLibraryPreset::builder()
        .rust_version("1.75.0")
        .coverage(true)
        .linter(true)
        .format_check(true)
        .security_scan(true)
        .build();

    // Generate GitHub Actions workflow directly
    let workflow = preset.to_github().expect("Failed to generate workflow");

    // Serialize to YAML
    let yaml = serde_yaml::to_string(&workflow).expect("Failed to serialize");

    println!("Generated GitHub Actions Workflow (New API):\n");
    println!("{}", yaml);
}
