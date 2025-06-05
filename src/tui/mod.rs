mod components;
mod app;
mod controls;
mod graph;
mod results;
mod setup;
mod traits;

use std::io::Stdout;

pub(crate) use app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{layout::{Alignment, Rect}, prelude::CrosstermBackend, style::{Color, Style}, widgets::{block::Title, Block, BorderType, Borders}, Frame};

pub(crate) use traits::Component;

use crate::tui::traits::BlockExt;

pub(crate) struct TuiComponent<C: Component + Sized> {
    changed: bool,
    has_focus: bool,
    name: Option<&'static str>,
    selected_name: Option<&'static str>,
    comp: C,
    style: ComponentStyle,
}

impl<C: Component> TuiComponent<C> {
    pub(crate) fn new(comp: C, style: ComponentStyle) -> Self {
        Self {
            changed: true,
            has_focus: false,
            name: None,
            selected_name: None,
            comp,
            style,
        }
    }

    pub(crate) fn with_name(self, name: &'static str) -> Self {
        Self {
            changed: self.changed,
            has_focus: self.has_focus,
            name: Some(name),
            selected_name: self.selected_name,
            comp: self.comp,
            style: self.style,
        }
    }

    pub(crate) fn with_selected_name(self, selected_name: &'static str) -> Self {
        Self {
            changed: self.changed,
            has_focus: self.has_focus,
            name: self.name,
            selected_name: Some(selected_name),
            comp: self.comp,
            style: self.style,
        }
    }

    pub(crate) fn underlying_mut(&mut self) -> &mut C {
        &mut self.comp
    }
}

impl<C: Component> Component for TuiComponent<C> {
    fn changed(&self) -> bool {
        self.comp.changed()
    }

    fn acknowledge_change(&mut self) {
        self.comp.acknowledge_change()
    }

    fn give_focus(&mut self) {
        self.has_focus = true;
    }
    
    fn acknowledge_focus(&mut self) {
        self.has_focus = false;
    }

    fn handle_key_press(&mut self, key: KeyEvent) {
        self.comp.handle_key_press(key)
    }

    fn update(&mut self) {
        self.comp.update()
    }

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
            let block = Block::new()
                .borders(Borders::ALL)
                .set_title(self)
                .set_border(self)
                .style(self.style.main);

        frame.render_widget(block.clone(), area);
        self.comp.render(frame, block.inner(area));
    }
}

#[derive(Clone)]
pub(crate) struct ComponentStyle {
    pub(crate) main: Style,
    pub(crate) border: Style,
    pub(crate) selected_border: Option<Style>,
}

impl ComponentStyle {
    pub(crate) fn get_selected_border(&self) -> &Style {
        self.selected_border
            .as_ref()
            .unwrap_or(&self.border)
    }
    
    fn default() -> Self {
        Self {
            main: Style::new().fg(Color::Green).bg(Color::Black),
            border: Style::new(),
            selected_border: None,
        }
    }
    
    fn selectable() -> Self {
        Self {
            main: Style::new().fg(Color::Green).bg(Color::Black),
            border: Style::new().fg(Color::Green).bg(Color::Black),
            selected_border: Some(Style::new().fg(Color::Rgb(192, 224, 192)).bg(Color::Black)),
        }
    }
    
    fn dark() -> Self {
        Self {
            main: Style::new().fg(Color::DarkGray).bg(Color::Black),
            border: Style::new().fg(Color::DarkGray).bg(Color::Black),
            selected_border: None,
        }
    }
}
