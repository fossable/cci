use crate::tui::state::{Platform, TreeItem, TuiState};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key_event(state: &mut TuiState, key: KeyEvent) {
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

        // Write and quit
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.should_write = true;
            state.should_quit = true;
        }

        // Open platform menu with 'p'
        KeyCode::Char('p') => {
            state.open_platform_menu();
        }

        // Toggle preset/feature/option with Enter or Space
        KeyCode::Enter | KeyCode::Char(' ') => {
            if let Some(item) = state.current_item().cloned() {
                match item {
                    TreeItem::Preset(preset_id) => {
                        state.toggle_preset(&preset_id);
                    }
                    TreeItem::Feature(_preset_id, _feature_id) => {
                        // Features don't have a toggle action, just expand/collapse with arrow keys
                    }
                    TreeItem::Option(preset_id, _feature_id, option_id) => {
                        state.toggle_option(&preset_id, &option_id);
                    }
                }
            }
        }

        // Left - collapse preset/feature
        KeyCode::Left | KeyCode::Char('h') => {
            if let Some(item) = state.current_item().cloned() {
                match item {
                    TreeItem::Preset(preset_id) => {
                        if state.expanded_presets.contains(&preset_id) {
                            state.toggle_preset_expand(&preset_id);
                            state.update_current_item_description();
                        }
                    }
                    TreeItem::Feature(preset_id, feature_id) => {
                        let key = (preset_id.clone(), feature_id.clone());
                        if state.expanded_features.contains(&key) {
                            state.toggle_feature_expand(&preset_id, &feature_id);
                            state.update_current_item_description();
                        } else {
                            // Feature not expanded, collapse parent preset instead
                            if state.expanded_presets.contains(&preset_id) {
                                state.toggle_preset_expand(&preset_id);
                                // Move cursor to the preset
                                if let Some(pos) = state.tree_items.iter().position(|item| {
                                    matches!(item, TreeItem::Preset(p) if p == &preset_id)
                                }) {
                                    state.tree_cursor = pos;
                                    state.update_current_item_description();
                                }
                            }
                        }
                    }
                    TreeItem::Option(preset_id, feature_id, _option_id) => {
                        // If on an option, collapse its parent feature
                        let key = (preset_id.clone(), feature_id.clone());
                        if state.expanded_features.contains(&key) {
                            state.toggle_feature_expand(&preset_id, &feature_id);
                            // Move cursor to the feature
                            if let Some(pos) = state.tree_items.iter().position(|item| {
                                matches!(item, TreeItem::Feature(p, f) if p == &preset_id && f == &feature_id)
                            }) {
                                state.tree_cursor = pos;
                                state.update_current_item_description();
                            }
                        }
                    }
                }
            }
        }

        // Right - expand preset/feature
        KeyCode::Right | KeyCode::Char('l') => {
            if let Some(item) = state.current_item().cloned() {
                match item {
                    TreeItem::Preset(preset_id) => {
                        if !state.expanded_presets.contains(&preset_id) {
                            state.toggle_preset_expand(&preset_id);
                            state.update_current_item_description();
                        }
                    }
                    TreeItem::Feature(preset_id, feature_id) => {
                        let key = (preset_id.clone(), feature_id.clone());
                        if !state.expanded_features.contains(&key) {
                            state.toggle_feature_expand(&preset_id, &feature_id);
                            state.update_current_item_description();
                        }
                    }
                    TreeItem::Option(_, _, _) => {
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
