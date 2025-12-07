use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::{Job, Trigger};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pipeline {
    pub name: String,
    pub triggers: Vec<Trigger>,
    pub jobs: Vec<Job>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub cache: CacheConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<String>,
    #[serde(default)]
    pub key_prefix: String,
}

fn default_cache_enabled() -> bool {
    true
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            paths: Vec::new(),
            key_prefix: String::new(),
        }
    }
}
