use std::io::Stdout;

use ratatui::{layout::Rect, prelude::CrosstermBackend, Frame};

use crate::{tui::{ComponentStyle, FocusableComponent, TuiComponent, TuiComponentBuilder}, Component};

pub(crate) struct Results {
    
}

impl Results {
    pub(crate) fn new() -> TuiComponent<Self> {
        TuiComponentBuilder::new(ComponentStyle::default()).build(Self {})
    }
}

impl FocusableComponent for Results {
    fn give_focus(&mut self) {
        todo!()
    }

    fn remove_focus(&mut self) {
        todo!()
    }
}

impl Component for Results {
    fn handle_key_press(&mut self, key: crossterm::event::KeyEvent) {
        todo!()
    }

    fn update(&mut self) -> bool {
        todo!()
    }

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        todo!()
    }
}