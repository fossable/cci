use crate::tui::state::{Platform, TreeItem, TuiState};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render_ui(f: &mut Frame, state: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Platform selector bar
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Footer
        ])
        .split(f.size());

    // Platform selector bar
    render_platform_bar(f, chunks[0], state);

    // Main content (two panels: tree + preview)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),  // Left panel (tree)
            Constraint::Percentage(60),  // Right panel (preview)
        ])
        .split(chunks[1]);

    render_presets_panel(f, main_chunks[0], state);
    render_preview_panel(f, main_chunks[1], state);

    // Footer
    render_footer(f, chunks[2], state);

    // Platform menu overlay (if open)
    if state.platform_menu_open {
        render_platform_menu(f, state);
    }
}

fn render_platform_bar(f: &mut Frame, area: Rect, state: &TuiState) {
    let text = format!("Platform: {} (press 'p' to change)", state.target_platform.name());

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(paragraph, area);
}

fn render_presets_panel(f: &mut Frame, area: Rect, state: &TuiState) {
    let mut items: Vec<ListItem> = Vec::new();

    for (i, item) in state.tree_items.iter().enumerate() {
        let is_selected = i == state.tree_cursor;
        let style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let list_item = match item {
            TreeItem::Platform => {
                // Platform is no longer in the tree
                continue;
            }
            TreeItem::Preset(preset) => {
                let is_expanded = state.expanded_presets.contains(preset);
                let is_enabled = state.enabled_presets.contains(preset);

                let expand_icon = if is_expanded { "▼" } else { "▶" };
                let checkbox = if is_enabled { "[✓]" } else { "[ ]" };

                ListItem::new(format!("{} {} {}", expand_icon, checkbox, preset.name()))
                    .style(if is_enabled {
                        style.fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else {
                        style
                    })
            }
            TreeItem::Option(preset, option_index) => {
                let option_name = preset.options()[*option_index];
                let is_enabled = state.get_option_value(*preset, *option_index);
                let checkbox = if is_enabled { "[✓]" } else { "[ ]" };

                ListItem::new(format!("    {} {}", checkbox, option_name))
                    .style(style)
            }
        };

        items.push(list_item);
    }

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Presets ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );

    f.render_widget(list, area);
}

fn render_preview_panel(f: &mut Frame, area: Rect, state: &TuiState) {
    let preview = if let Some(error) = &state.generation_error {
        Paragraph::new(format!("Error: {}", error))
            .style(Style::default().fg(Color::Red))
            .wrap(Wrap { trim: true })
    } else {
        // Apply syntax highlighting to YAML
        let lines = highlight_yaml(&state.yaml_preview);
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
    };

    let block = Block::default()
        .title(format!(" Preview - {} ", state.target_platform.name()))
        .borders(Borders::ALL);

    f.render_widget(preview.block(block), area);
}

fn render_platform_menu(f: &mut Frame, state: &TuiState) {
    let area = f.size();

    // Center the menu
    let menu_width = 40;
    let menu_height = 8;
    let x = (area.width.saturating_sub(menu_width)) / 2;
    let y = (area.height.saturating_sub(menu_height)) / 2;

    let menu_area = Rect {
        x,
        y,
        width: menu_width,
        height: menu_height,
    };

    // Clear the background
    f.render_widget(Clear, menu_area);

    // Render menu items
    let platforms = Platform::all();
    let items: Vec<ListItem> = platforms
        .iter()
        .enumerate()
        .map(|(i, platform)| {
            let is_selected = i == state.platform_menu_cursor;
            let is_current = *platform == state.target_platform;

            let style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let marker = if is_current { "● " } else { "  " };
            let prefix = if is_selected { "> " } else { "  " };

            ListItem::new(format!("{}{}{}", prefix, marker, platform.name()))
                .style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Select Platform ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
        );

    f.render_widget(list, menu_area);
}

fn highlight_yaml(yaml: &str) -> Vec<Line<'_>> {
    let mut lines = Vec::new();

    for line in yaml.lines() {
        let trimmed = line.trim_start();

        if trimmed.is_empty() {
            lines.push(Line::from(""));
            continue;
        }

        // Comment lines
        if trimmed.starts_with('#') {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(Color::DarkGray),
            )));
            continue;
        }

        // Parse the line into spans
        let mut spans = Vec::new();
        let indent = line.len() - trimmed.len();

        // Add indentation
        if indent > 0 {
            spans.push(Span::raw(" ".repeat(indent)));
        }

        // Key-value pairs
        if let Some(colon_pos) = trimmed.find(':') {
            let key = &trimmed[..colon_pos];
            let rest = &trimmed[colon_pos..];

            // Key (cyan)
            spans.push(Span::styled(
                key.to_string(),
                Style::default().fg(Color::Cyan),
            ));

            // Colon
            spans.push(Span::raw(":"));

            if rest.len() > 1 {
                let value = &rest[1..].trim_start();

                // Check for special values
                if value.starts_with('"') || value.starts_with('\'') {
                    // String value (green)
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(
                        value.to_string(),
                        Style::default().fg(Color::Green),
                    ));
                } else if *value == "true" || *value == "false" {
                    // Boolean (magenta)
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(
                        value.to_string(),
                        Style::default().fg(Color::Magenta),
                    ));
                } else if value.parse::<f64>().is_ok() {
                    // Number (yellow)
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(
                        value.to_string(),
                        Style::default().fg(Color::Yellow),
                    ));
                } else if !value.is_empty() {
                    // Other value
                    spans.push(Span::raw(" "));
                    spans.push(Span::raw(value.to_string()));
                }
            }
        } else if trimmed.starts_with("- ") {
            // List item
            spans.push(Span::styled(
                "- ".to_string(),
                Style::default().fg(Color::Yellow),
            ));
            spans.push(Span::raw(trimmed[2..].to_string()));
        } else {
            // Other lines
            spans.push(Span::raw(trimmed.to_string()));
        }

        lines.push(Line::from(spans));
    }

    lines
}

fn render_footer(f: &mut Frame, area: Rect, state: &TuiState) {
    let help_text = if state.platform_menu_open {
        vec![
            Span::styled("↑↓/jk", Style::default().fg(Color::Blue)),
            Span::raw(" navigate | "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(" select | "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(" close"),
        ]
    } else {
        vec![
            Span::styled("←→/hl", Style::default().fg(Color::Blue)),
            Span::raw(" expand/collapse | "),
            Span::styled("Space/Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" toggle | "),
            Span::styled("↑↓/jk", Style::default().fg(Color::Blue)),
            Span::raw(" navigate | "),
            Span::styled("p", Style::default().fg(Color::Cyan)),
            Span::raw(" platform | "),
            Span::styled("Ctrl+W", Style::default().fg(Color::Green)),
            Span::raw(" write | "),
            Span::styled("q", Style::default().fg(Color::Red)),
            Span::raw(" quit"),
        ]
    };

    let paragraph = Paragraph::new(Line::from(help_text))
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(paragraph, area);
}
