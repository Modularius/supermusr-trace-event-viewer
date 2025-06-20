use std::str::FromStr;

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};

use crate::{
    tui::{ComponentStyle, ParentalFocusComponent, TuiComponent, TuiComponentBuilder},
    Component,
};

pub(crate) struct TextBox<D> {
    parent_has_focus: bool,
    data: D,
}

impl<D> TextBox<D>
where
    D: ToString + FromStr,
    <D as FromStr>::Err: std::fmt::Debug,
{
    pub(crate) fn new(data: D, name: Option<&'static str>) -> TuiComponent<Self> {
        let builder = TuiComponentBuilder::new(ComponentStyle::selectable()).is_in_block(true);

        if let Some(name) = name {
            builder.with_name(name)
        } else {
            builder
        }
        .build(Self {
            data,
            parent_has_focus: false,
        })
    }

    pub(crate) fn set(&mut self, data: D) {
        self.data = data;
    }

    pub(crate) fn get(&self) -> &D {
        &self.data
    }
}

impl<D> Component for TextBox<D>
where
    D: ToString,
{
    fn render(&self, frame: &mut Frame, area: Rect) {
        let style = Style::new().bg(Color::Black).fg(Color::Gray);

        let paragraph = Paragraph::new(self.data.to_string())
            .alignment(Alignment::Center)
            .style(style);
        frame.render_widget(paragraph, area);
    }
}

impl<D> ParentalFocusComponent for TextBox<D>
where
    D: ToString,
{
    fn propagate_parental_focus(&mut self, focus: bool) {
        self.parent_has_focus = focus;
    }
}
