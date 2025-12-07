use cci::presets::rust::RustLibraryPreset;
use cci::traits::{ToCircleCI, ToGitHub, ToGitLab, ToJenkins};

fn main() {
    // Create a fully-featured Rust library preset
    let preset = RustLibraryPreset::builder()
        .rust_version("1.75.0")
        .coverage(true)
        .linter(true)
        .format_check(true)
        .security_scan(true)
        .build();

    println!("=== GITHUB ACTIONS ===\n");
    let github_workflow = preset.to_github().expect("Failed to generate GitHub workflow");
    let github_yaml = serde_yaml::to_string(&github_workflow).expect("Failed to serialize");
    println!("{}\n", github_yaml);

    println!("=== GITLAB CI ===\n");
    let gitlab_config = preset.to_gitlab().expect("Failed to generate GitLab config");
    let gitlab_yaml = serde_yaml::to_string(&gitlab_config).expect("Failed to serialize");
    println!("{}\n", gitlab_yaml);

    println!("=== CIRCLECI ===\n");
    let circleci_config = preset.to_circleci().expect("Failed to generate CircleCI config");
    let circleci_yaml = serde_yaml::to_string(&circleci_config).expect("Failed to serialize");
    println!("{}\n", circleci_yaml);

    println!("=== JENKINS ===\n");
    let jenkins_config = preset.to_jenkins().expect("Failed to generate Jenkins config");
    println!("Agent: {}", jenkins_config.agent);
    println!("Stages:");
    for stage in &jenkins_config.stages {
        println!("  - {}", stage.name);
        for step in &stage.steps {
            println!("    * {}", step);
        }
    }
}
