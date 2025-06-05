use std::io::Stdout;

use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    Frame,
};
use supermusr_streaming_types::frame_metadata_v2_generated::GpsTime;

use crate::{
    tui::{components::textbox::TextBox, traits::{Component, ComponentContainer}, ComponentStyle, TuiComponent},
};

#[derive(Default, Clone)]
enum DateBoxFocus {
    #[default]
    Year,
    Day,
    Hour,
    Minute,
    Second,
    Nanosecond,
}
pub(crate) struct DateBox {
    is_changed: bool,
    time: GpsTime,
    focus: DateBoxFocus,
    year: TuiComponent<TextBox>,
    day: TuiComponent<TextBox>,
    hour: TuiComponent<TextBox>,
    min: TuiComponent<TextBox>,
    sec: TuiComponent<TextBox>,
    ns: TuiComponent<TextBox>,
}

impl DateBox {
    pub(crate) fn new(time: DateTime<Utc>) -> TuiComponent<Self> {
        let time : GpsTime = time.into();
        TuiComponent::new(
            Self {
                is_changed: true,
                time,
                focus: Default::default(),
                year: TextBox::new(&time.year().to_string()).with_name("(Y))"),
                day: TextBox::new(&time.day().to_string()).with_name("(D))"),
                hour: TextBox::new(&time.hour().to_string()).with_name("(H))"),
                min: TextBox::new(&time.minute().to_string()).with_name("(M))"),
                sec: TextBox::new(&time.second().to_string()).with_name("(S))"),
                ns: TextBox::new(&time.nanosecond().to_string()).with_name("(N))"),
            },
            ComponentStyle::selectable(),
        )
    }
}

impl ComponentContainer for DateBox {
    fn focused_component(&self) -> &dyn Component {
        match self.focus {
            DateBoxFocus::Year => &self.year,
            DateBoxFocus::Day => &self.day,
            DateBoxFocus::Hour => &self.hour,
            DateBoxFocus::Minute => &self.min,
            DateBoxFocus::Second => &self.sec,
            DateBoxFocus::Nanosecond => &self.ns,
        }
    }

    fn focused_component_mut(&mut self) -> &mut dyn Component {
        match self.focus {
            DateBoxFocus::Year => &mut self.year,
            DateBoxFocus::Day => &mut self.day,
            DateBoxFocus::Hour => &mut self.hour,
            DateBoxFocus::Minute => &mut self.min,
            DateBoxFocus::Second => &mut self.sec,
            DateBoxFocus::Nanosecond => &mut self.ns,
        }
    }
}

impl Component for DateBox {
    fn changed(&self) -> bool {
        self.is_changed
    }

    fn acknowledge_change(&mut self) {
        self.is_changed = false;
    }

    fn give_focus(&mut self) {
        self.is_changed = true;
        self.acknowledge_focus();
        self.focused_component_mut().give_focus();
    }
    
    fn acknowledge_focus(&mut self) {
        self.year.acknowledge_focus();
        self.day.acknowledge_focus();
        self.hour.acknowledge_focus();
        self.min.acknowledge_focus();
        self.sec.acknowledge_focus();
        self.ns.acknowledge_focus();
    }

    fn handle_key_press(&mut self, key: KeyEvent) {
        if key == KeyEvent::new(KeyCode::Char('y'), KeyModifiers::SHIFT) {
            self.focus = DateBoxFocus::Year;
            self.give_focus();
            self.is_changed = true;
        } else if key == KeyEvent::new(KeyCode::Char('d'), KeyModifiers::SHIFT) {
            self.focus = DateBoxFocus::Day;
            self.give_focus();
            self.is_changed = true;
        } else if key == KeyEvent::new(KeyCode::Char('h'), KeyModifiers::SHIFT) {
            self.focus = DateBoxFocus::Hour;
            self.give_focus();
            self.is_changed = true;
        } else if key == KeyEvent::new(KeyCode::Char('m'), KeyModifiers::SHIFT) {
            self.focus = DateBoxFocus::Minute;
            self.give_focus();
            self.is_changed = true;
        } else if key == KeyEvent::new(KeyCode::Char('s'), KeyModifiers::SHIFT) {
            self.focus = DateBoxFocus::Second;
            self.give_focus();
            self.is_changed = true;
        } else if key == KeyEvent::new(KeyCode::Char('n'), KeyModifiers::SHIFT) {
            self.focus = DateBoxFocus::Nanosecond;
            self.give_focus();
            self.is_changed = true;
        } else {
            self.focused_component_mut().handle_key_press(key);
            self.is_changed = self.focused_component().changed();
        }
    }

    fn update(&mut self) {}

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let (year,day,hour,min,sec,ns) =
        {
            let layout = Layout::new()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Ratio(1,9),  // year
                    Constraint::Ratio(1,9),  // day
                    Constraint::Ratio(1,9),  // hour
                    Constraint::Ratio(1,9),  // min
                    Constraint::Ratio(1,9),  // sec
                    Constraint::Ratio(4,9),  // ns
                ])
                .split(area);
            (layout[0],layout[1],layout[2],layout[3],layout[4],layout[5])
        };

        self.year.render(frame,year);
        self.day.render(frame,day);
        self.hour.render(frame,hour);
        self.min.render(frame,min);
        self.sec.render(frame,sec);
        self.ns.render(frame,ns);
    }
}
