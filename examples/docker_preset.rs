use cci::presets::docker::{DockerPreset, DockerRegistry};
use cci::traits::{ToGitHub, ToGitLab};

fn main() {
    println!("=== Docker Preset Examples ===\n");

    // Example 1: Basic Docker build (no push)
    println!("--- Example 1: Basic Docker Build (No Push) ---");
    let basic_preset = DockerPreset::builder()
        .image_name("myorg/myapp")
        .build();

    let github_workflow = basic_preset.to_github().expect("Failed to generate GitHub workflow");
    let github_yaml = serde_yaml::to_string(&github_workflow).expect("Failed to serialize");
    println!("{}\n", github_yaml);

    // Example 2: Docker Hub with caching
    println!("--- Example 2: Docker Hub Push with Caching ---");
    let dockerhub_preset = DockerPreset::builder()
        .image_name("myorg/myapp")
        .registry(DockerRegistry::DockerHub)
        .cache(true)
        .build();

    let github_workflow = dockerhub_preset.to_github().expect("Failed to generate GitHub workflow");
    let github_yaml = serde_yaml::to_string(&github_workflow).expect("Failed to serialize");
    println!("{}\n", github_yaml);

    // Example 3: GitHub Container Registry (tags only)
    println!("--- Example 3: GitHub Container Registry (Push on Tags Only) ---");
    let ghcr_preset = DockerPreset::builder()
        .image_name("myapp")
        .registry(DockerRegistry::GitHubRegistry)
        .push_on_tags_only(true)
        .cache(true)
        .build();

    let github_workflow = ghcr_preset.to_github().expect("Failed to generate GitHub workflow");
    let github_yaml = serde_yaml::to_string(&github_workflow).expect("Failed to serialize");
    println!("{}\n", github_yaml);

    // Example 4: Custom Dockerfile path with build args
    println!("--- Example 4: Custom Dockerfile with Build Args ---");
    let custom_preset = DockerPreset::builder()
        .image_name("myorg/custom-app")
        .registry(DockerRegistry::DockerHub)
        .dockerfile_path("./docker/Dockerfile.prod")
        .build_context("./app")
        .build_arg("VERSION", "1.2.3")
        .build_arg("BUILD_DATE", "2024-01-15")
        .cache(true)
        .build();

    let github_workflow = custom_preset.to_github().expect("Failed to generate GitHub workflow");
    let github_yaml = serde_yaml::to_string(&github_workflow).expect("Failed to serialize");
    println!("{}\n", github_yaml);

    // Example 5: GitLab CI
    println!("--- Example 5: GitLab CI Configuration ---");
    let gitlab_preset = DockerPreset::builder()
        .image_name("myorg/myapp")
        .registry(DockerRegistry::DockerHub)
        .build();

    let gitlab_config = gitlab_preset.to_gitlab().expect("Failed to generate GitLab config");
    let gitlab_yaml = serde_yaml::to_string(&gitlab_config).expect("Failed to serialize");
    println!("{}\n", gitlab_yaml);
}
