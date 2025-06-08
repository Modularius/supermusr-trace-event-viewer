use std::io::Stdout;

use ratatui::{layout::Rect, prelude::CrosstermBackend, Frame};
use tracing::info;

use crate::{
    messages::Cache, tui::{ComponentStyle, FocusableComponent, TuiComponent, TuiComponentBuilder}, Component
};

pub(crate) struct Results {}

impl Results {
    pub(crate) fn new() -> TuiComponent<Self> {
        TuiComponentBuilder::new(ComponentStyle::selectable()).build(Self {})
    }

    pub(crate) fn push(&mut self, cache: Cache) {
        info!("{}", cache.iter_traces().len());
        for trace in cache.iter_traces() {
            info!("{:?}", trace.0);
        }
    }
}

impl FocusableComponent for Results {
    fn set_focus(&mut self, focus: bool) {
        self.propagate_parental_focus(focus);
    }

    fn propagate_parental_focus(&mut self, focus: bool) {
        //self.<children>.propagate_parental_focus(focus);
    }
}

impl Component for Results {
    fn handle_key_press(&mut self, key: crossterm::event::KeyEvent) {
        
    }

    fn update(&mut self) -> bool {
        todo!()
    }

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        
    }
}
