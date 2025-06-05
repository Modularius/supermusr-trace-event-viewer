use std::io::Stdout;

use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Alignment, Rect},
    prelude::CrosstermBackend,
    style::{Color, Style},
    text::Text,
    widgets::Paragraph,
    Frame,
};

use crate::tui::{traits::Component, ComponentStyle, TuiComponent};

pub(crate) struct TextBox {
    is_changed: bool,
    text: String,
}

impl TextBox {
    pub(crate) fn new(text: &str) -> TuiComponent<Self> {
        TuiComponent::new(
            Self {
                is_changed: true,
                text: text.to_owned(),
            },
            ComponentStyle::selectable(),
        )
    }

    pub(crate) fn set(&mut self, text: &str) {
        self.text = text.to_owned();
    }
}

impl Component for TextBox {
    fn changed(&self) -> bool {
        self.is_changed
    }

    fn acknowledge_change(&mut self) {
        self.is_changed = false;
    }

    fn give_focus(&mut self) {
        self.is_changed = true;
    }
    
    fn acknowledge_focus(&mut self) {}

    fn handle_key_press(&mut self, key: KeyEvent) {}

    fn update(&mut self) {}

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let style = Style::new().bg(Color::Black).fg(Color::Gray);
        let text = Text::styled(&self.text, style);
        let paragraph = Paragraph::new(text).alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }
}