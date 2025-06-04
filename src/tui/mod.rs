mod app;
mod setup;
mod controls;
mod results;
mod graph;

use std::io::Stdout;

pub(crate) use app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{layout::Rect, prelude::CrosstermBackend, Frame};

pub(crate) trait Component {
    pub(crate) fn changed(&self) -> bool;
    pub(crate) fn acknowledge_change(&mut self);

    fn handle_key_press(&mut self, key: KeyEvent) -> bool {
        false
    }

    fn update(&mut self) {

    }

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        
    }
}