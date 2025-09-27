// src/models/ui_state.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionState {
    #[serde(default)]
    pub selected_uuids: Vec<String>,
    #[serde(default)]
    pub cursor_position: usize,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            selected_uuids: Vec::new(),
            cursor_position: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportState {
    #[serde(default)]
    pub scroll_x: f64,
    #[serde(default)]
    pub scroll_y: f64,
    #[serde(default = "default_zoom")]
    pub zoom_level: f64,
}

fn default_zoom() -> f64 { 1.0 }

impl Default for ViewportState {
    fn default() -> Self {
        Self {
            scroll_x: 0.0,
            scroll_y: 0.0,
            zoom_level: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIState {
    #[serde(default)]
    pub selection: SelectionState,
    #[serde(default)]
    pub viewport: ViewportState,
    #[serde(default = "default_active_tab")]
    pub active_tab: String,
    #[serde(default)]
    pub clipboard_content: Option<String>,
}

fn default_active_tab() -> String { "editor_svg".to_string() }

impl Default for UIState {
    fn default() -> Self {
        Self {
            selection: SelectionState::default(),
            viewport: ViewportState::default(),
            active_tab: "editor_svg".to_string(),
            clipboard_content: None,
        }
    }
}
