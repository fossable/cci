pub mod app;
pub mod builder;
pub mod events;
pub mod stage;
pub mod state;
pub mod ui;

use crate::detection::DetectionResult;
use crate::error::Result;

/// Run the interactive TUI for configuring CI pipelines
pub fn run_tui(detection: DetectionResult, platform: Option<String>) -> Result<()> {
    app::TuiApp::new(detection, platform)?.run()
}
