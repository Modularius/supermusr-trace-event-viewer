use std::{io::Stdout, str::FromStr};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Color, Style},
    widgets::{List, ListItem, Paragraph},
    Frame,
};
use supermusr_common::Channel;

use crate::{tui::{ComponentStyle, FocusableComponent, TuiComponent, TuiComponentBuilder}, Component};

#[derive(Clone)]
pub(crate) struct Channels {
    has_focus: bool,
    parent_has_focus: bool,
    channels: Vec<Channel>,
    channel_index : usize,
}

impl Channels {
    pub(crate) fn new(name: Option<&'static str>) -> TuiComponent<Self> {
        let builder = TuiComponentBuilder::new(ComponentStyle::selectable())
            .is_in_block(true);

        if let Some(name) = name {
            builder.with_name(name)
        } else {
            builder
        }.build(Self {
            channels: Default::default(),
            has_focus: false,
            parent_has_focus: false,
            channel_index: 0,
        })
    }

    pub(crate) fn set(&mut self, channels: Vec<Channel>) {
        self.channels = channels;
        self.channel_index = 0;
    }

    pub(crate) fn get(&self) -> Option<Channel> {
        if self.channels.is_empty() {
            None
        } else {
            Some(self.channels[self.channel_index])
        }
    }
}

impl FocusableComponent for Channels {
    fn set_focus(&mut self, focus: bool) {
        self.has_focus = focus;
    }

    fn propagate_parental_focus(&mut self, focus: bool) {
        self.parent_has_focus = focus;
    }
}

impl Component for Channels {
    fn handle_key_press(&mut self, key: KeyEvent) {
        if self.channels.is_empty() {
            return;
        }
        if self.has_focus {
            if key.code == KeyCode::Left {
                self.channel_index = (self.channels.len() + self.channel_index - 1) % self.channels.len();
            } else if key.code == KeyCode::Right {
                self.channel_index = (self.channel_index + 1) % self.channels.len();
            }
        }
    }

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        if self.channels.is_empty() {
            return;
        }
        let areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(self.channels.iter().map(|_|(Constraint::Ratio(1,self.channels.len() as u32))).collect::<Vec<_>>())
            .split(area);

        let style = Style::new().bg(Color::Black).fg(Color::Gray);
        let select_style = Style::new().bg(Color::Green).fg(Color::Black);

        for (index, &area) in areas.iter().enumerate() {
            let channel = Paragraph::new(format!("{}", self.channels[index]))
                .style(if index == self.channel_index {
                    select_style
                } else {
                    style
                });
            frame.render_widget(channel, area);
        }        
    }
}