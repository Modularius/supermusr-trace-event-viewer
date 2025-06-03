mod app;

use std::io::Stdout;

pub(crate) use app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{layout::Rect, prelude::CrosstermBackend, Frame};

pub(crate) trait Component {
    fn handle_key_press(&mut self, key: KeyEvent);

    fn update(&mut self) {

    }

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        
    }
}