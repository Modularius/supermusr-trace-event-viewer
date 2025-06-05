
use std::io::Stdout;

use crossterm::event::KeyEvent;
use ratatui::{layout::{Alignment, Rect}, prelude::CrosstermBackend, widgets::{block::Title, Block, BorderType}, Frame};

use crate::tui::TuiComponent;

pub(crate) trait Component {
    fn changed(&self) -> bool;

    fn acknowledge_change(&mut self);

    fn give_focus(&mut self);
    
    fn acknowledge_focus(&mut self);

    fn handle_key_press(&mut self, key: KeyEvent);

    fn update(&mut self);

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect);
    
    fn help(&self) -> &'static str { "" }
}

pub(crate) trait BlockExt {
    fn set_title<C : Component>(self, comp: &TuiComponent<C>) -> Self;
    fn set_border<C : Component>(self, comp: &TuiComponent<C>) -> Self;
}

impl BlockExt for Block<'_> {
    fn set_title<C : Component>(self, comp: &TuiComponent<C>) -> Self {
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
    }

    fn set_border<C : Component>(self, comp: &TuiComponent<C>) -> Self {
        if comp.has_focus {
            self.border_style(comp.style.get_selected_border().clone())
                .border_type(BorderType::Rounded)
        } else {
            self.border_style(comp.style.border)
        }
    }
}

pub(crate) trait ComponentContainer {

    fn focused_component(&self) -> &dyn Component;

    fn focused_component_mut(&mut self) -> &mut dyn Component;
}