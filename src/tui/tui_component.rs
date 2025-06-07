use std::{io::Stdout, rc::Rc};

use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, prelude::CrosstermBackend, style::{Color, Style}, widgets::{Block, Borders}, Frame};

use crate::tui::{builder::TuiComponentBuilder, style::ComponentStyle, BlockExt, Component, FocusableComponent};


pub(crate) struct TuiComponent<C: Component + Sized> {
    pub(crate) is_changed: bool,
    pub(crate) has_focus: bool,
    comp: C,
    config: TuiComponentBuilder,
}

impl<C: Component> TuiComponent<C> {
    pub(crate) fn new(comp: C, config: TuiComponentBuilder) -> Self {
        Self {
            is_changed: true,
            has_focus: false,
            comp,
            config,
        }
    }

    pub(crate) fn underlying_mut(&mut self) -> &mut C {
        &mut self.comp
    }
}

impl<C: FocusableComponent> FocusableComponent for TuiComponent<C> {
    fn give_focus(&mut self) {
        self.has_focus = true;
        self.comp.give_focus();
    }
    
    fn remove_focus(&mut self) {
        self.has_focus = false;
        self.comp.remove_focus();
    }

}

impl<C: Component> Component for TuiComponent<C> {
    fn handle_key_press(&mut self, key: KeyEvent) {
        self.comp.handle_key_press(key)
    }

    fn update(&mut self) -> bool {
        self.comp.update()
    }

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let block = Block::new()
            .borders(Borders::ALL)
            .set_title(self)
            .set_border(self)
            .style(self.config.style.main);

        frame.render_widget(block.clone(), area);
        self.comp.render(frame, block.inner(area));
    }
}
