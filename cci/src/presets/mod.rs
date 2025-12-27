pub mod docker {
    pub mod preset;
}
pub mod go {
    pub mod app;
}
pub mod python {
    pub mod app;
}
pub mod rust {
    pub mod preset;
}

// Export preset structs
pub use docker::preset::{DockerPreset, DockerRegistry};
pub use go::app::GoAppPreset;
pub use python::app::{PythonAppPreset, PythonFormatterTool, PythonLinterTool};
pub use rust::preset::RustPreset;
