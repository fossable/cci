use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Runner {
    UbuntuLatest,
    Ubuntu2204,
    Ubuntu2004,
    MacOSLatest,
    MacOS13,
    WindowsLatest,
    Windows2022,
    Custom(String),
}

impl Default for Runner {
    fn default() -> Self {
        Runner::UbuntuLatest
    }
}
