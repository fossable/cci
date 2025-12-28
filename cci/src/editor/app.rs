use crate::detection::DetectionResult;
use crate::editor::events::handle_key_event;
use crate::editor::state::EditorState;
use crate::editor::ui::render_ui;
use crate::error::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

pub struct EditorApp {
    state: EditorState,
}

impl EditorApp {
    pub fn new(detection: DetectionResult, platform: Option<String>) -> Result<Self> {
        let working_dir = PathBuf::from(".");
        let state = EditorState::from_detection(detection, platform, working_dir)?;

        Ok(Self { state })
    }

    pub fn run(mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Run the event loop
        let result = self.event_loop(&mut terminal);

        // Cleanup
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    fn event_loop<B: ratatui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            // Render
            terminal.draw(|f| render_ui(f, &self.state))?;

            // Handle events
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    handle_key_event(&mut self.state, key);
                }
            }

            // Write CI config if requested
            if self.state.should_write {
                self.write_config()?;
                self.state.should_write = false; // Reset the flag so we don't keep writing
            }

            // Check for exit
            if self.state.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn write_config(&self) -> Result<()> {
        use std::fs;

        let output_path = self.state.working_dir.join(self.state.target_platform.output_path());

        // Create parent directories
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write file
        fs::write(&output_path, &self.state.yaml_preview)?;

        println!("✨ Generated: {}", output_path.display());

        Ok(())
    }
}
