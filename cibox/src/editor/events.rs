use crate::editor::state::{EditorState, Platform, TreeItem};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_key_event(state: &mut EditorState, key: KeyEvent) {
    // If platform menu is open, handle menu navigation
    if state.platform_menu_open {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                state.close_platform_menu();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if state.platform_menu_cursor > 0 {
                    state.platform_menu_cursor -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let platforms = Platform::all();
                if state.platform_menu_cursor < platforms.len() - 1 {
                    state.platform_menu_cursor += 1;
                }
            }
            KeyCode::Enter => {
                state.select_platform_from_menu();
            }
            _ => {}
        }
        return;
    }

    // Normal tree navigation
    match key.code {
        // Quit
        KeyCode::Char('q') | KeyCode::Esc => {
            state.should_quit = true;
        }

        // Write CI pipeline YAML
        KeyCode::Char('w') | KeyCode::Char('W') => {
            state.should_write = true;
        }

        // Open platform menu with 'p'
        KeyCode::Char('p') => {
            state.open_platform_menu();
        }

        // Toggle with Enter or Space
        KeyCode::Enter | KeyCode::Char(' ') => {
            if let Some(item) = state.current_item().cloned() {
                match item {
                    TreeItem::Category(category) => {
                        state.toggle_category_expand(&category);
                    }
                    TreeItem::Preset(preset_id) => {
                        state.toggle_preset(&preset_id);
                    }
                    TreeItem::Field(preset_id, field_id) => {
                        state.toggle_option(&preset_id, &field_id);
                    }
                }
            }
        }

        // Left - collapse
        KeyCode::Left | KeyCode::Char('h') => {
            if let Some(item) = state.current_item().cloned() {
                match item {
                    TreeItem::Category(category) => {
                        if state.expanded_categories.contains(&category) {
                            state.toggle_category_expand(&category);
                            state.update_current_item_description();
                        }
                    }
                    TreeItem::Preset(preset_id) => {
                        if state.expanded_presets.contains(&preset_id) {
                            state.toggle_preset_expand(&preset_id);
                            state.update_current_item_description();
                        } else {
                            // Preset not expanded, collapse parent category
                            if let Some(category) = state.preset_category(&preset_id) {
                                if state.expanded_categories.contains(&category) {
                                    state.toggle_category_expand(&category);
                                    // Move cursor to the category
                                    if let Some(pos) = state.tree_items.iter().position(
                                        |item| matches!(item, TreeItem::Category(c) if c == &category),
                                    ) {
                                        state.tree_cursor = pos;
                                        state.update_current_item_description();
                                    }
                                }
                            }
                        }
                    }
                    TreeItem::Field(preset_id, _field_id) => {
                        // Collapse parent preset
                        if state.expanded_presets.contains(&preset_id) {
                            state.toggle_preset_expand(&preset_id);
                            // Move cursor to the preset
                            if let Some(pos) = state.tree_items.iter().position(
                                |item| matches!(item, TreeItem::Preset(p) if p == &preset_id),
                            ) {
                                state.tree_cursor = pos;
                                state.update_current_item_description();
                            }
                        }
                    }
                }
            }
        }

        // Right - expand
        KeyCode::Right | KeyCode::Char('l') => {
            if let Some(item) = state.current_item().cloned() {
                match item {
                    TreeItem::Category(category) => {
                        if !state.expanded_categories.contains(&category) {
                            state.toggle_category_expand(&category);
                            state.update_current_item_description();
                        }
                    }
                    TreeItem::Preset(preset_id) => {
                        if !state.expanded_presets.contains(&preset_id) {
                            state.toggle_preset_expand(&preset_id);
                            state.update_current_item_description();
                        }
                    }
                    TreeItem::Field(_, _) => {
                        // Already at leaf level, do nothing
                    }
                }
            }
        }

        // Navigation - J/K for preview scroll when Shift is held
        KeyCode::Char('K') => {
            state.scroll_preview_up();
        }

        KeyCode::Char('J') => {
            state.scroll_preview_down();
        }

        // Navigation - regular up/down and lowercase j/k for tree navigation
        KeyCode::Up | KeyCode::Char('k') => {
            if state.tree_cursor > 0 {
                state.tree_cursor -= 1;
                state.update_current_item_description();
            }
        }

        KeyCode::Down | KeyCode::Char('j') => {
            if state.tree_cursor < state.tree_items.len().saturating_sub(1) {
                state.tree_cursor += 1;
                state.update_current_item_description();
            }
        }

        // Tab to cycle platform (alternative to 'p' menu)
        KeyCode::Tab => {
            state.cycle_platform();
        }

        _ => {}
    }
}
