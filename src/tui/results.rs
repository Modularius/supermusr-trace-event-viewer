use std::io::Stdout;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{layout::Rect, prelude::CrosstermBackend, style::{Color, Style}, widgets::{List, ListItem}, Frame};

use crate::{data::{DigitiserMetadata, DigitiserTrace}, Cache, Component};


pub(crate) struct Results {
    changed: bool,
    index: Option<usize>,
    list: Vec<String>,
}

impl Results {
    pub(crate) fn new() -> Self {
        Results {
            changed: true,
            index: None,
            list: Default::default()
        }
    }
}

impl Component for Results {
    fn changed(&self) -> bool {
        self.changed
    }

    fn acknowledge_change(&mut self) {
        self.changed = false;
    }

    fn handle_key_press(&mut self, key: KeyEvent) {
        if key == KeyEvent::new(KeyCode::Up, KeyModifiers::NONE) {

            self.changed = true;
        } else if key == KeyEvent::new(KeyCode::Down, KeyModifiers::NONE) {

            self.changed = true;
        } else {
        }
    }

    fn update(&mut self) {
    }

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let list = //List::new(self.list.iter().map(|trace|trace.metadata().expect("").collect::<Vec<_>>()))
        List::new(self.list.iter().map(|trace|ListItem::new(trace.as_str())).collect::<Vec<_>>())
            .highlight_style(Style::new().bg(Color::Cyan))
            .highlight_symbol(">");

            frame.render_widget(list, area);
    }
}