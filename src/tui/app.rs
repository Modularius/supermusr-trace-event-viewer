use ratatui::{buffer::Buffer, layout::{Constraint, Direction, Layout, Rect}, symbols, text, widgets::{Block, Borders, List, Widget}};
use rdkafka::consumer::BaseConsumer;

use crate::{tui::Component, Cache, Topics};

pub(crate) struct App<'a> {
    changed: bool,
    consumer: &'a BaseConsumer,
    topics: &'a Topics,
    cache: Cache,
}

impl<'a> App<'a> {
    pub(crate) fn new(consumer: &'a BaseConsumer, topics: &'a Topics) -> Self {
        App{ changed: true, consumer , topics, cache: Cache::default() }
    }

    pub(crate) fn changed(&self) -> bool {
        self.changed
    }
}

impl<'a> Component for App<'a> {
    fn handle_key_press(&mut self) {
    }

    fn update(&mut self) {
    }

    fn render(&self, area: Rect) {
        let (setup, display) = {
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(4), Constraint::Length(8)])
                .split(area);
                (chunk[0], chunk[1])
        };
        let (results, graph) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(4), Constraint::Length(8)])
                .split(display);
                (chunk[0], chunk[1])
        };

        {
            let block = Block::new()
                .title(text::Line::raw("TODO List").centered())
                .borders(Borders::TOP)
                .border_style(TODO_HEADER_STYLE)
                .style(NORMAL_ROW_BG);
            List::new(vec![])
                .block(block)
                .highlight_style(SELECTED_STYLE)
                .highlight_symbol(">");
        }
    }
}