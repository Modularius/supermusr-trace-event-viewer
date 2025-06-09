use std::io::Stdout;

use ratatui::{layout::{Alignment, Rect}, prelude::CrosstermBackend, style::{Color, Style}, widgets::Paragraph, Frame};

use crate::{
    messages::Cache, tui::{ComponentStyle, FocusableComponent, TuiComponent, TuiComponentBuilder}, Component
};

pub(crate) struct Results {
    cache: Option<Cache>
}

impl Results {
    pub(crate) fn new() -> TuiComponent<Self> {
        TuiComponentBuilder::new(ComponentStyle::selectable()).build(Self {
            cache: None,
        })
    }

    pub(crate) fn push(&mut self, cache: Cache) {
        self.cache = Some(cache);
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
        if let Some(cache) = self.cache.as_ref() {
            let number = Paragraph::new(format!("Number of traces/events: {}/{}", cache.iter_traces().len(),cache.iter_events().len()))
                .alignment(Alignment::Left)
                .style(Style::new().fg(Color::White).bg(Color::Black));
            
            frame.render_widget(number, area);
        }
    }
}
