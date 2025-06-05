use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    Frame,
};
use std::io::Stdout;
use strum::{EnumCount, EnumIter, IntoEnumIterator};
use crate::tui::{components::{DateBox, TextBox}, traits::{Component, ComponentContainer}, ComponentStyle, TuiComponent};

#[derive(Default, Clone, EnumIter, EnumCount)]
enum Focus {
    #[default]
    TimestampFrom,
    Number,
    Channels,
    Digitisers,
}

pub(crate) struct Setup {
    is_changed: bool,
    is_editing: bool,
    focus: Focus,
    from: TuiComponent<DateBox>,
    number: TuiComponent<TextBox>,
    channels: TuiComponent<TextBox>,
    digitisers: TuiComponent<TextBox>,
}

impl Setup {
    pub(crate) fn new() -> TuiComponent<Self> {
        TuiComponent::new(
            Setup {
                is_changed: true,
                is_editing: false,
                focus: Default::default(),
                from: DateBox::new(Utc::now()).with_name("timestamp from"),
                number: TextBox::new("1").with_name("number"),
                channels: TextBox::new("").with_name("channel"),
                digitisers: TextBox::new("").with_name("digitiser id"),
            },
            ComponentStyle::selectable(),
        )
        .with_name("setup")
    }
}

impl ComponentContainer for Setup {
    fn focused_component(&self) -> &dyn Component {
        match self.focus {
            Focus::TimestampFrom => &self.from,
            Focus::Number => &self.number,
            Focus::Channels => &self.channels,
            Focus::Digitisers => &self.digitisers
        }
    }

    fn focused_component_mut(&mut self) -> &mut dyn Component {
        match self.focus {
            Focus::TimestampFrom => &mut self.from,
            Focus::Number => &mut self.number,
            Focus::Channels => &mut self.channels,
            Focus::Digitisers => &mut self.digitisers
        }
    }
}

impl Component for Setup {
    fn changed(&self) -> bool {
        self.is_changed
    }

    fn acknowledge_change(&mut self) {
        self.is_changed = false;
    }

    fn give_focus(&mut self) {
        self.acknowledge_focus();
        self.focused_component_mut().give_focus();
    }
    
    fn acknowledge_focus(&mut self) {
        self.from.acknowledge_focus();
        self.number.acknowledge_focus();
        self.channels.acknowledge_focus();
        self.digitisers.acknowledge_focus();
    }

    fn handle_key_press(&mut self, key: KeyEvent) {
        if key == KeyEvent::new(KeyCode::Right, KeyModifiers::NONE) {
            self.focus = Focus::iter().cycle().skip(self.focus.clone() as usize + 1).next().expect("");
            self.give_focus();
            self.is_changed = true;
        } else if key == KeyEvent::new(KeyCode::Left, KeyModifiers::NONE) {
            self.focus = Focus::iter().rev().cycle().skip(Focus::COUNT - self.focus.clone() as usize).next().expect("");
            self.give_focus();
            self.is_changed = true;
        } else {
            self.focused_component_mut().handle_key_press(key);
            self.is_changed = self.focused_component().changed();
        }
    }

    fn update(&mut self) {}

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let (from, number, channels, digitisers) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Ratio(2, 5),
                    Constraint::Ratio(1, 5),
                    Constraint::Ratio(1, 5),
                    Constraint::Ratio(1, 5)])
                .split(area);
            (chunk[0], chunk[1], chunk[2], chunk[3])
        };

        // Timestamp From
        self.from.render(frame, from);

        // Timestamp To
        self.number.render(frame, number);

        // Channels
        self.channels.render(frame, channels);

        // Digitisers
        self.digitisers.render(frame, digitisers);
    }
    
    fn help(&self) -> &'static str {
        "Use [Tab] to switch, [Left/Right] Arrows to navigate, [Num Keys] to edit, and [Enter] to Search."
    }
}