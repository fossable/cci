pub mod app;
pub mod events;
pub mod state;
pub mod ui;

use crate::detection::DetectorRegistry;
use crate::error::Result;
use std::path::PathBuf;

/// Run the interactive TUI for configuring CI pipelines
pub fn run() -> Result<()> {
    // Auto-detect project
    let working_dir = PathBuf::from(".");
    let registry = DetectorRegistry::new();
    let detection = registry.detect(&working_dir)?;

    // Launch TUI
    app::TuiApp::new(detection, None)?.run()
}
