use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Trigger {
    Push {
        branches: Vec<String>,
    },
    PullRequest {
        branches: Vec<String>,
    },
    Tag {
        pattern: String,
    },
    Schedule {
        cron: String,
    },
    Manual,
}
