use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircleCIConfig {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orbs: Option<HashMap<String, String>>,
    pub jobs: HashMap<String, CircleCIJob>,
    pub workflows: HashMap<String, CircleCIWorkflow>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircleCIJob {
    pub docker: Vec<CircleCIDocker>,
    pub steps: Vec<CircleCIStep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircleCIDocker {
    pub image: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CircleCIStep {
    Simple(String),
    Command {
        run: CircleCIRun,
    },
    Cache {
        #[serde(rename = "restore_cache")]
        restore_cache: CircleCICache,
    },
    SaveCache {
        #[serde(rename = "save_cache")]
        save_cache: CircleCICacheSave,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CircleCIRun {
    Simple(String),
    Detailed {
        name: String,
        command: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircleCICache {
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircleCICacheSave {
    pub key: String,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircleCIWorkflow {
    pub jobs: Vec<CircleCIWorkflowJob>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CircleCIWorkflowJob {
    Simple(String),
    WithRequires {
        #[serde(flatten)]
        job: HashMap<String, CircleCIJobRequires>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircleCIJobRequires {
    pub requires: Vec<String>,
}
