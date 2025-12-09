use crate::tui::config::OptionValue;
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
            Constraint::Length(4),  // Information message bar
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Footer
        ])
        .split(f.size());

    // Information message bar (where platform bar was)
    render_info_bar(f, chunks[0], state);

    // Main content (two panels: tree + preview)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),  // Left panel (tree)
            Constraint::Percentage(60),  // Right panel (preview + platform)
        ])
        .split(chunks[1]);

    render_presets_panel(f, main_chunks[0], state);

    // Right side: preview above platform selector
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),     // Preview
            Constraint::Length(3),  // Platform selector
        ])
        .split(main_chunks[1]);

    render_preview_panel(f, right_chunks[0], state);
    render_platform_bar(f, right_chunks[1], state);

    // Footer
    render_footer(f, chunks[2], state);

    // Platform menu overlay (if open)
    if state.platform_menu_open {
        render_platform_menu(f, state);
    }
}

fn render_info_bar(f: &mut Frame, area: Rect, state: &TuiState) {
    let text = if !state.current_item_description.is_empty() {
        state.current_item_description.clone()
    } else {
        "Navigate with ↑↓/jk, toggle with Space/Enter, expand/collapse with ←→/hl".to_string()
    };

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Gray))
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title(" Information "));

    f.render_widget(paragraph, area);
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

        let list_item = match item {
            TreeItem::Preset(preset_id) => {
                let preset = match state.registry.get(preset_id) {
                    Some(p) => p,
                    None => continue,
                };

                let config = state.preset_configs.get(preset_id.as_str());
                let is_expanded = state.expanded_presets.contains(preset_id);
                let has_options_enabled = config.map(|c| {
                    c.values.values().any(|v| matches!(v, OptionValue::Bool(true)))
                }).unwrap_or(false);
                let matches_project = preset.matches_project(&state.project_type, &state.working_dir);

                let expand_icon = if is_expanded { "▼" } else { "▶" };
                let circle_icon = if has_options_enabled { "●" } else { "○" };

                let circle_color = if has_options_enabled {
                    if matches_project { Color::Green } else { Color::DarkGray }
                } else {
                    Color::White
                };

                let text_color = if is_selected {
                    Color::Yellow
                } else if !matches_project {
                    Color::DarkGray
                } else {
                    Color::White
                };

                let line = Line::from(vec![
                    Span::styled(format!("{} ", expand_icon), Style::default().fg(text_color)),
                    Span::styled(circle_icon, Style::default().fg(circle_color)),
                    Span::styled(format!(" {}", preset.preset_name()), Style::default().fg(text_color)),
                ]);

                let item_style = if is_selected {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(line).style(item_style)
            }
            TreeItem::Feature(preset_id, feature_id) => {
                let preset = match state.registry.get(preset_id) {
                    Some(p) => p,
                    None => continue,
                };

                let feature = match preset.features().into_iter().find(|f| &f.id == feature_id) {
                    Some(f) => f,
                    None => continue,
                };

                let matches_project = preset.matches_project(&state.project_type, &state.working_dir);
                let is_expanded = state.expanded_features.contains(&(preset_id.clone(), feature_id.clone()));
                let expand_icon = if is_expanded { "▼" } else { "▶" };

                let text_color = if is_selected {
                    Color::Yellow
                } else if !matches_project {
                    Color::DarkGray
                } else {
                    Color::White
                };

                let line = Line::from(vec![
                    Span::styled(format!("  {} {}", expand_icon, feature.display_name), Style::default().fg(text_color)),
                ]);

                let item_style = if is_selected {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(line).style(item_style)
            }
            TreeItem::Option(preset_id, _feature_id, option_id) => {
                let preset = match state.registry.get(preset_id) {
                    Some(p) => p,
                    None => continue,
                };

                let config = match state.preset_configs.get(preset_id.as_str()) {
                    Some(c) => c,
                    None => continue,
                };

                let value = match config.get(option_id) {
                    Some(v) => v,
                    None => continue,
                };

                // Find the option metadata to get the display name
                let features = preset.features();
                let option_meta = features
                    .iter()
                    .flat_map(|f| &f.options)
                    .find(|o| &o.id == option_id);

                let display_name = option_meta.map(|o| o.display_name.as_str()).unwrap_or(option_id);

                let matches_project = preset.matches_project(&state.project_type, &state.working_dir);

                let display_text = match value {
                    OptionValue::Bool(b) => {
                        let checkbox = if *b { "[✓]" } else { "[ ]" };
                        format!("      {} {}", checkbox, display_name)
                    }
                    OptionValue::Enum { selected, .. } => {
                        format!("      {} ({})", display_name, selected)
                    }
                    OptionValue::String(s) => {
                        format!("      {}: {}", display_name, s)
                    }
                    OptionValue::Int(n) => {
                        format!("      {}: {}", display_name, n)
                    }
                };

                let text_color = if is_selected {
                    Color::Yellow
                } else if !matches_project {
                    Color::DarkGray
                } else {
                    Color::White
                };

                let item_style = if is_selected {
                    Style::default().fg(text_color).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(text_color)
                };

                ListItem::new(display_text).style(item_style)
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
            .scroll((state.preview_scroll, 0))
    } else {
        // Apply syntax highlighting to YAML
        let lines = highlight_yaml(&state.yaml_preview);
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((state.preview_scroll, 0))
    };

    let output_path = state.target_platform.output_path();
    let filename = output_path
        .to_str()
        .unwrap_or("config.yml");

    let block = Block::default()
        .title(format!(" Preview - {} (Shift+J/K to scroll) ", filename))
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
            Span::styled("JK", Style::default().fg(Color::Magenta)),
            Span::raw(" scroll preview | "),
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
