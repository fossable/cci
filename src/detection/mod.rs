mod detector;
mod go;
mod python;
mod registry;
mod rust;

pub use detector::{DetectionResult, ProjectDetector, ProjectType};
pub use go::GoDetector;
pub use python::PythonDetector;
pub use registry::DetectorRegistry;
pub use rust::RustDetector;
