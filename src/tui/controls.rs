use std::io::Stdout;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{layout::{Constraint, Direction, Layout, Rect}, prelude::CrosstermBackend, style::{Color, Style}, widgets::List, Frame};

use crate::{data::{DigitiserMetadata, DigitiserTrace}, Cache, Component};

enum Focus {
    Blob
}

pub(crate) struct Controls{
    changed: bool,
    focus: Focus
}

impl Controls {
    pub(crate) fn new() -> Self {
        Controls{
            changed: true,
            focus: Focus::Blob,
        }
    }
}

impl Component for Controls {
    fn changed(&self) -> bool {
        self.changed
    }

    fn acknowledge_change(&mut self) {
        self.changed = false;
    }

    fn handle_key_press(&mut self, key: KeyEvent) {
        if key == KeyEvent::new(KeyCode::Up, KeyModifiers::NONE) {

        } else if key == KeyEvent::new(KeyCode::Down, KeyModifiers::NONE) {

        } else if key == KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE) {
            self.changed = true;
        } else {
        }
    }

    fn update(&mut self) {
    }

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let (setup, display) = {
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(4), Constraint::Min(0)])
                .split(area);
                (chunk[0], chunk[1])
        };
    }
}