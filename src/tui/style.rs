use std::io::Stdout;

use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, prelude::CrosstermBackend, style::{Color, Style}, widgets::{Block, Borders}, Frame};

use crate::tui::{BlockExt, Component};



#[derive(Clone)]
pub(crate) struct ComponentStyle {
    pub(crate) main: Style,
    pub(crate) border: Style,
    pub(crate) selected_border: Option<Style>,
}

impl ComponentStyle {
    pub(crate) fn get_selected_border(&self) -> &Style {
        self.selected_border
            .as_ref()
            .unwrap_or(&self.border)
    }
    
    pub(crate) fn default() -> Self {
        Self {
            main: Style::new().fg(Color::Green).bg(Color::Black),
            border: Style::new(),
            selected_border: None,
        }
    }
    
    pub(crate) fn selectable() -> Self {
        Self {
            main: Style::new().fg(Color::Green).bg(Color::Black),
            border: Style::new().fg(Color::Green).bg(Color::Black),
            selected_border: Some(Style::new().fg(Color::Rgb(192, 224, 192)).bg(Color::Black)),
        }
    }
    
    pub(crate) fn dark() -> Self {
        Self {
            main: Style::new().fg(Color::DarkGray).bg(Color::Black),
            border: Style::new().fg(Color::DarkGray).bg(Color::Black),
            selected_border: None,
        }
    }
}
