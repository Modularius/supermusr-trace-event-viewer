use std::io::Stdout;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{layout::Rect, prelude::CrosstermBackend, style::{Color, Style}, widgets::List, Frame};

use crate::{data::{DigitiserMetadata, DigitiserTrace}, Cache, Component};


pub(crate) struct Graph<'a> {
    changed: bool,
    trace: Option<&'a DigitiserTrace>,
}

impl<'a> Graph<'a> {
    pub(crate) fn new() -> Self {
        Graph {
            changed: true,
            trace: None
        }
    }
}

impl<'a> Component for Graph<'a> {
    fn changed(&self) -> bool {
        self.changed
    }

    fn acknowledge_change(&mut self) {
        self.changed = false;
    }

    fn handle_key_press(&mut self, key: KeyEvent) {
        
    }

    fn update(&mut self) {
    }

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        
    }
}