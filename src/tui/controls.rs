use std::io::Stdout;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    Frame,
};

use crate::tui::{components::TextBox, traits::Component, ComponentStyle, TuiComponent};

enum Focus {
    Blob,
}

pub(crate) struct Controls {
    is_changed: bool,
    focus: Focus,
    text: TuiComponent<TextBox>,
}

impl Controls {
    pub(crate) fn new() -> TuiComponent<Self> {
        TuiComponent::new(
            Controls {
                is_changed: true,
                text: TextBox::new(""),
                focus: Focus::Blob,
            },
            ComponentStyle::selectable(),
        )
    }

    pub(crate) fn set(&mut self, text: &str) {
        self.text.underlying_mut().set(text);
        self.is_changed = true;
    }
}

impl Component for Controls {
    fn changed(&self) -> bool {
        self.is_changed
    }

    fn acknowledge_change(&mut self) {
        self.is_changed = false;
    }
    
    fn give_focus(&mut self) {}
    
    fn acknowledge_focus(&mut self) {}

    fn handle_key_press(&mut self, key: KeyEvent) {}

    fn update(&mut self) {}

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        self.text.render(frame, area)
    }
}
