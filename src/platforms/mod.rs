pub mod adapter;
pub mod circleci;
pub mod github;
pub mod gitlab;
pub mod jenkins;

pub use adapter::PlatformAdapter;
pub use circleci::CircleCIAdapter;
pub use github::GitHubAdapter;
pub use gitlab::GitLabAdapter;
pub use jenkins::JenkinsAdapter;
