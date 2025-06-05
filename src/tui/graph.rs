use std::io::Stdout;

use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    prelude::CrosstermBackend,
    Frame,
};

use crate::{
    data::DigitiserTrace, tui::{traits::Component, ComponentStyle, TuiComponent}
};

pub(crate) struct Graph<'a> {
    changed: bool,
    trace: Option<&'a DigitiserTrace>,
}

impl<'a> Graph<'a> {
    pub(crate) fn new() -> TuiComponent<Self> {
        TuiComponent::new(
            Graph {
                changed: true,
                trace: None,
            },
            ComponentStyle::selectable(),
        )
        .with_name("graph")
    }
}

impl<'a> Component for Graph<'a> {
    fn changed(&self) -> bool {
        self.changed
    }

    fn acknowledge_change(&mut self) {
        self.changed = false;
    }
    
    fn give_focus(&mut self) {}
    
    fn acknowledge_focus(&mut self) {}


    fn handle_key_press(&mut self, key: KeyEvent) {}

    fn update(&mut self) {}

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {}
}
