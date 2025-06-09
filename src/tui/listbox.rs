use std::{io::Stdout, str::FromStr};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    prelude::CrosstermBackend,
    style::{Color, Style},
    widgets::{List, ListItem, ListState},
    Frame,
};

use crate::{tui::{ComponentStyle, FocusableComponent, TuiComponent, TuiComponentBuilder}, Component};

pub(crate) struct ListBox<D> {
    has_state_changed: bool,
    has_focus: bool,
    parent_has_focus: bool,
    data: Vec<D>,
    state: ListState,
}

impl<D> ListBox<D> where D: Clone + ToString + FromStr, <D as FromStr>::Err: std::fmt::Debug {
    pub(crate) fn new(data: &[D], name: Option<&'static str>) -> TuiComponent<Self> {
        let builder = TuiComponentBuilder::new(ComponentStyle::selectable())
            .is_in_block(true);

        if let Some(name) = name {
            builder.with_name(name)
        } else {
            builder
        }.build(Self {
            data: data.to_vec(),
            has_focus: false,
            parent_has_focus: false,
            state: ListState::default(),
            has_state_changed: true,
        })
    }

    pub(crate) fn set(&mut self, data: Vec<D>) {
        self.data = data;
        self.state = ListState::default()
    }

    pub(crate) fn pop_state_change(&mut self) -> bool {
        if self.has_state_changed {
            self.has_state_changed = false;
            true
        } else {
            false
        }
    }

    pub(crate) fn select(&mut self) -> Option<usize> {
        if self.data.is_empty() {
            None
        } else {
            self.state.select(Some(self.state.offset()));
            self.has_state_changed = true;
            Some(self.state.offset())
        }
    }
}

impl<D> FocusableComponent for ListBox<D> where D: Clone + ToString + FromStr, <D as FromStr>::Err: std::fmt::Debug {
    fn set_focus(&mut self, focus: bool) {
        self.has_focus = focus;
    }

    fn propagate_parental_focus(&mut self, focus: bool) {
        self.parent_has_focus = focus;
    }
}

impl<D> Component for ListBox<D> where D: Clone + ToString + FromStr, <D as FromStr>::Err: std::fmt::Debug {
    fn handle_key_press(&mut self, key: KeyEvent) {
        if self.data.is_empty() {
            return;
        }
        if self.has_focus {
            if key.code == KeyCode::Up {
                let new_offset = (self.data.len() + self.state.offset() - 1) % self.data.len();
                self.state = self.state.clone().with_offset(new_offset);
                self.has_state_changed = true;
            } else if key.code == KeyCode::Up {
                let new_offset = (self.data.len() + self.state.offset() + 1) % self.data.len();
                self.state = self.state.clone().with_offset(new_offset);
                self.has_state_changed = true;
            }
        }
    }

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let style = Style::new().bg(Color::Black).fg(Color::Gray);
        let focus_style = Style::new().bg(Color::Black).fg(Color::Green);
        let select_style = Style::new().bg(Color::Green).fg(Color::Black);
        
        let list = List::new(
            self.data.iter()
                .map(ToString::to_string)
                .map(ListItem::new)
                .enumerate()
                .map(|(i,t)|{
                    if self.state.selected().is_some_and(|si|si == i) {
                        t.style(select_style)
                    } else if i == self.state.offset() {
                        t.style(focus_style)
                    } else {
                        t.style(style)
                    }
                })
                .collect::<Vec<_>>())
            .style(style);
        frame.render_stateful_widget(list, area, &mut self.state.clone());
        
    }
}