use cci::platforms::{GitHubAdapter, PlatformAdapter};
use cci::presets::rust_library_preset;

fn main() {
    // Create a Rust library preset pipeline
    let pipeline = rust_library_preset("1.75");

    // Transform it to GitHub Actions
    let adapter = GitHubAdapter;
    let yaml = adapter.generate(&pipeline).expect("Failed to generate workflow");

    println!("Generated GitHub Actions Workflow:\n");
    println!("{}", yaml);
}
