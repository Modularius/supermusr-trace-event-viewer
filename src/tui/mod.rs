mod builder;
mod style;
mod tui_component;

use std::io::Stdout;

use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Alignment, Rect},
    prelude::CrosstermBackend,
    widgets::{block::Title, Block, BorderType},
    Frame,
};

pub(crate) use builder::TuiComponentBuilder;
pub(crate) use style::ComponentStyle;
pub(crate) use tui_component::TuiComponent;

pub(crate) trait Component {
    fn handle_key_press(&mut self, key: KeyEvent);

    fn update(&mut self) -> bool {
        false
    }

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect);

    fn help(&self) -> &'static str {
        ""
    }
}

pub(crate) trait FocusableComponent: Component {
    fn set_focus(&mut self, focus: bool);

    fn propagate_parental_focus(&mut self, focus: bool);
}

pub(crate) trait BlockExt {
    fn set_title<C: Component>(self, comp: &TuiComponent<C>) -> Self;
    fn set_border<C: Component>(self, comp: &TuiComponent<C>) -> Self;
}

impl BlockExt for Block<'_> {
    fn set_title<C: Component>(self, comp: &TuiComponent<C>) -> Self {
        /*
        let name = if comp.has_focus {
            comp.selected_name.or(comp.name)
        } else {
            comp.name
        };
        if let Some(name) = name {
            let title = Title::default()
                .alignment(Alignment::Center)
                .content(name);
            self.title(title)
        } else {
            self
        }
        */
        self
    }

    fn set_border<C: Component>(self, comp: &TuiComponent<C>) -> Self {
        if comp.has_focus() {
            //self.border_style(comp.style.get_selected_border().clone())
            //    .border_type(BorderType::Rounded)
        } else {
            //self.border_style(comp.style.border)
        }
        self
    }
}

pub(crate) trait ComponentContainer {
    fn focused_component(&self) -> &dyn FocusableComponent;

    fn focused_component_mut(&mut self) -> &mut dyn FocusableComponent;
}
