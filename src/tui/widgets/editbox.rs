use std::str::FromStr;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};

use crate::{
    tui::{
        ComponentStyle, FocusableComponent, InputComponent, ParentalFocusComponent, TuiComponent,
        TuiComponentBuilder,
    },
    Component,
};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

pub(crate) struct EditBox<D> {
    has_focus: bool,
    parent_has_focus: bool,
    data: D,
    input: Input,
    error: bool,
}

impl<D> EditBox<D>
where
    D: ToString + FromStr,
    <D as FromStr>::Err: std::fmt::Debug,
{
    pub(crate) fn new(data: D, name: Option<&'static str>) -> TuiComponent<Self> {
        let input = Input::new(data.to_string());
        let builder = TuiComponentBuilder::new(ComponentStyle::selectable()).is_in_block(true);

        if let Some(name) = name {
            builder.with_name(name)
        } else {
            builder
        }
        .build(Self {
            input,
            data,
            has_focus: false,
            parent_has_focus: false,
            error: false,
        })
    }

    pub(crate) fn set(&mut self, data: D) {
        self.data = data;
        self.input = Input::new(self.data.to_string());
    }
    pub(crate) fn get(&self) -> &D {
        &self.data
    }
}

impl<D> Component for EditBox<D>
where
    D: ToString + FromStr,
    <D as FromStr>::Err: std::fmt::Debug,
{
    fn render(&self, frame: &mut Frame, area: Rect) {
        let style =
            Style::new()
                .bg(Color::Black)
                .fg(if self.error { Color::Red } else { Color::Gray });

        let paragraph = Paragraph::new(self.input.value())
            .alignment(Alignment::Center)
            .style(style);
        frame.render_widget(paragraph, area);
    }
}

impl<D> InputComponent for EditBox<D>
where
    D: ToString + FromStr,
    <D as FromStr>::Err: std::fmt::Debug,
{
    fn handle_key_press(&mut self, key: KeyEvent) {
        if self.has_focus {
            if key == KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE) {
                if self.input.visual_cursor() != 0 {
                    self.input.handle_event(&Event::Key(key)).expect("");
                }
            } else if let KeyEvent {
                code: KeyCode::Char(_),
                modifiers: _,
                kind: _,
                state: _,
            } = key
            {
                self.input.handle_event(&Event::Key(key)).expect("");
            }

            self.error = false;
            match self.input.value().parse() {
                Ok(value) => self.data = value,
                Err(_) => {
                    self.error = true;
                }
            }
        }
    }
}

impl<D> FocusableComponent for EditBox<D>
where
    D: ToString + FromStr,
    <D as FromStr>::Err: std::fmt::Debug,
{
    fn set_focus(&mut self, focus: bool) {
        self.has_focus = focus;
    }
}

impl<D> ParentalFocusComponent for EditBox<D>
where
    D: ToString + FromStr,
    <D as FromStr>::Err: std::fmt::Debug,
{
    fn propagate_parental_focus(&mut self, focus: bool) {
        self.parent_has_focus = focus;
    }
}
