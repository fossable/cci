pub mod docker;
pub mod go;
pub mod python;
pub mod rust;

// Export preset structs
pub use docker::{DockerPreset, DockerRegistry};
pub use go::GoAppPreset;
pub use python::{PythonAppPreset, PythonFormatterTool, PythonLinterTool};
pub use rust::RustPreset;
