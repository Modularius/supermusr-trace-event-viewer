mod builder;
mod style;
mod widgets;
mod tui_component;

use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    widgets::{Block, BorderType},
    Frame,
};

pub(crate) use builder::TuiComponentBuilder;
pub(crate) use style::ComponentStyle;
pub(crate) use tui_component::TuiComponent;
pub(crate) use widgets::{TextBox, ListBox, Graph, GraphProperties, Channels, Statusbar, EditBox};

pub(crate) trait Component {
    fn render(&self, frame: &mut Frame, area: Rect);
}

pub(crate) trait ComponentContainer : Component {
    //fn focused_component(&self) -> &dyn FocusableComponent;

    fn focused_component_mut(&mut self) -> &mut dyn FocusableComponent;
}


pub(crate) trait InputComponent : Component {
    fn handle_key_press(&mut self, key: KeyEvent);
}

pub(crate) trait FocusableComponent: InputComponent {
    fn set_focus(&mut self, focus: bool);
}

pub(crate) trait ParentalFocusComponent: Component {
    fn propagate_parental_focus(&mut self, focus: bool);
}

pub(crate) trait BlockExt {
    fn set_title<C: Component>(self, comp: &TuiComponent<C>) -> Self;
    fn set_border<C: Component>(self, comp: &TuiComponent<C>) -> Self;
}

impl BlockExt for Block<'_> {
    fn set_title<C: Component>(self, comp: &TuiComponent<C>) -> Self {
        let name = if comp.has_focus() {
            comp.get_builder().selected_name.or(comp.get_builder().name)
        } else {
            comp.get_builder().name
        };
        if let Some(name) = name {
            self.title_top(name)
        } else {
            self
        }
    }

    fn set_border<C: Component>(self, comp: &TuiComponent<C>) -> Self {
        if comp.has_focus() {
            if comp.parent_has_focus() {
                self.border_style(comp.get_builder().style.full_focus)
                    .border_type(BorderType::Rounded)
            } else {
                self.border_style(comp.get_builder().style.only_self_focus)
                    .border_type(BorderType::Rounded)
            }
        } else {
            if comp.parent_has_focus() {
                self.border_style(comp.get_builder().style.only_parent_focus)
            } else {
                self.border_style(comp.get_builder().style.no_focus)
            }
        }
    }
}