use std::io::Stdout;

use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    prelude::CrosstermBackend,
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame,
};

use crate::tui::{BlockExt, Component};

#[derive(Clone)]
pub(crate) struct ComponentStyle {
    pub(crate) main: Style,
    border: Style,
    focus_border: Option<Style>,
    parent_focus_border: Option<Style>,
}

impl ComponentStyle {
    pub(crate) fn get_border(&self) -> &Style {
        &self.border
    }

    pub(crate) fn get_focus_border(&self) -> &Style {
        self.focus_border.as_ref().unwrap_or(&self.border)
    }

    pub(crate) fn get_parent_focus_border(&self) -> &Style {
        self.parent_focus_border.as_ref().unwrap_or(&self.border)
    }

    pub(crate) fn default() -> Self {
        Self {
            main: Style::new().fg(Color::Green).bg(Color::Black),
            border: Style::new(),
            focus_border: None,
            parent_focus_border: Some(Style::new().fg(Color::LightGreen).bg(Color::Black)),
        }
    }

    pub(crate) fn selectable() -> Self {
        Self {
            main: Style::new().fg(Color::Green).bg(Color::Black),
            border: Style::new().fg(Color::Green).bg(Color::Black),
            focus_border: Some(Style::new().fg(Color::Rgb(192, 255, 192)).bg(Color::Black)),
            parent_focus_border: Some(Style::new().fg(Color::LightGreen).bg(Color::Black)),
        }
    }

    pub(crate) fn dark() -> Self {
        Self {
            main: Style::new().fg(Color::DarkGray).bg(Color::Black),
            border: Style::new().fg(Color::DarkGray).bg(Color::Black),
            focus_border: None,
            parent_focus_border: None,
        }
    }
}
