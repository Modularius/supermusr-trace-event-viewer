use std::{io::Stdout, ops::{Deref, DerefMut}, rc::Rc};

use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    prelude::CrosstermBackend,
    style::{Color, Style, Styled},
    widgets::{Block, Borders},
    Frame,
};

use crate::tui::{
    builder::TuiComponentBuilder, style::ComponentStyle, BlockExt, Component, ComponentContainer, FocusableComponent, InputComponent, ParentalFocusComponent
};

pub(crate) struct TuiComponent<C: Component + Sized> {
    has_focus: bool,
    parent_has_focus: bool,
    comp: C,
    config: TuiComponentBuilder,
}

impl<C: Component> TuiComponent<C> {
    pub(crate) fn new(comp: C, config: TuiComponentBuilder) -> Self {
        Self {
            has_focus: false,
            parent_has_focus: false,
            comp,
            config,
        }
    }

    pub(crate) fn has_focus(&self) -> bool {
        self.has_focus
    }

    pub(crate) fn parent_has_focus(&self) -> bool {
        self.has_focus
    }

    pub(crate) fn get_builder(&self) -> &TuiComponentBuilder {
        &self.config
    }
}

impl<D> Deref for TuiComponent<D> where D: Component {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.comp
    }
}

impl<D> DerefMut for TuiComponent<D> where D: Component {    
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.comp
    }
}

trait FocusedComponentCycler {
    type Focus;
}

impl<C: ComponentContainer> ComponentContainer for TuiComponent<C> {
    /*fn focused_component(&self) -> &dyn FocusableComponent {
        self.comp.focused_component()
    }*/

    fn focused_component_mut(&mut self) -> &mut dyn FocusableComponent {
        self.comp.focused_component_mut()
    }
}

impl<C: Component> Component for TuiComponent<C> {
    fn render(&self, frame: &mut Frame, area: Rect) {
        if self.config.is_in_block {
            let block = Block::new()
                .borders(Borders::ALL)
                .set_title(self)
                .set_border(self);

            frame.render_widget(block.clone(), area);
            self.comp.render(frame, block.inner(area));
        } else {
            self.comp.render(frame, area);
        };
    }
}

impl<C: InputComponent> InputComponent for TuiComponent<C> {
    fn handle_key_press(&mut self, key: KeyEvent) {
        self.comp.handle_key_press(key)
    }
}

impl<C: FocusableComponent> FocusableComponent for TuiComponent<C> {
    fn set_focus(&mut self, focus: bool) {
        self.has_focus = focus;
        self.comp.set_focus(focus);
    }
}

impl<C: ParentalFocusComponent> ParentalFocusComponent for TuiComponent<C> {
    fn propagate_parental_focus(&mut self, focus: bool) {
        self.parent_has_focus = focus;
        self.comp.propagate_parental_focus(focus);
    }
}