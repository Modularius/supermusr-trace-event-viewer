use std::io::Stdout;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{layout::{Constraint, Direction, Layout, Rect}, prelude::CrosstermBackend, Frame};
use strum::{EnumCount, EnumIter, IntoEnumIterator};
use supermusr_common::{Channel, DigitizerId};

use crate::{
    finder::{MessageFinder, SearchTarget},
    tui::{ComponentContainer, ComponentStyle, FocusableComponent, EditBox, TuiComponent, TuiComponentBuilder},
    Component, Timestamp,
};

#[derive(Default, Clone, EnumCount, EnumIter)]
enum Focus {
    Date,
    #[default]
    Time,
    Number,
    Channel,
    DigitiserId
}

pub(crate) struct Setup {
    focus: Focus,
    date: TuiComponent<EditBox<NaiveDate>>,
    time: TuiComponent<EditBox<NaiveTime>>,
    number: TuiComponent<EditBox<usize>>,
    channel: TuiComponent<EditBox<Channel>>,
    digitiser_id: TuiComponent<EditBox<DigitizerId>>,
}

impl Setup {
    pub(crate) fn new(timestamp: Timestamp) -> TuiComponent<Self> {
        let comp = Self {
            focus: Default::default(),
            date: EditBox::new(timestamp.date_naive(), Some("Date (YYYY-MM-DD)")),
            time: EditBox::new(timestamp.time(), Some("Time (hh:mm:ss.f)")),
            number: EditBox::new(1, Some("Number to Collect")),
            channel: EditBox::new(1, Some("Channel to Seek")),
            digitiser_id: EditBox::new(4, Some("Digitiser Id to Seek"))
        };
        let mut setup = TuiComponentBuilder::new(ComponentStyle::default()).build(comp);
        setup.focused_component_mut().set_focus(true);
        setup
    }

    pub(crate) fn search<M: MessageFinder>(
        &self,
        message_finder: &mut M,
    ) -> bool {
        let timestamp = {
            let date = self.date.underlying().get();
            let time = self.time.underlying().get();
            Timestamp::from_naive_utc_and_offset(NaiveDateTime::new(date.clone(), time.clone()), Utc)
        };
        let number = *self.number.underlying().get();
        let channel = *self.channel.underlying().get();
        let digitiser_id = *self.digitiser_id.underlying().get();
        message_finder.init_search(SearchTarget {
            timestamp,
            number,
            channels: vec![channel],
            digitiser_ids: vec![digitiser_id],
        })
    }
}

impl ComponentContainer for Setup {
    fn focused_component(&self) -> &dyn FocusableComponent {
        match self.focus {
            Focus::Date => &self.date,
            Focus::Time => &self.time,
            Focus::Number => &self.number,
            Focus::Channel => &self.channel,
            Focus::DigitiserId => &self.digitiser_id,
        }
    }

    fn focused_component_mut(&mut self) -> &mut dyn FocusableComponent {
        match self.focus {
            Focus::Date => &mut self.date,
            Focus::Time => &mut self.time,
            Focus::Number => &mut self.number,
            Focus::Channel => &mut self.channel,
            Focus::DigitiserId => &mut self.digitiser_id,
        }
    }
}

impl FocusableComponent for Setup {
    fn set_focus(&mut self, focus: bool) {
        self.propagate_parental_focus(focus);
    }

    fn propagate_parental_focus(&mut self, focus: bool) {
        self.date.propagate_parental_focus(focus);
        self.date.propagate_parental_focus(focus);
        self.number.propagate_parental_focus(focus);
        self.channel.propagate_parental_focus(focus);
        self.digitiser_id.propagate_parental_focus(focus);
    }
}

impl Component for Setup {
    fn handle_key_press(&mut self, key: crossterm::event::KeyEvent) {
        if key == KeyEvent::new(KeyCode::Right, KeyModifiers::NONE) {
            self.focused_component_mut().set_focus(false);

            self.focus = Focus::iter().cycle().skip(self.focus.clone() as usize + 1).next().expect("");
            
            self.focused_component_mut().set_focus(true);
        } else if key == KeyEvent::new(KeyCode::Left, KeyModifiers::NONE) {
            self.focused_component_mut().set_focus(false);

            self.focus = Focus::iter().rev().cycle().skip(Focus::COUNT - self.focus.clone() as usize).next().expect("");
            
            self.focused_component_mut().set_focus(true);
        } else {
            self.focused_component_mut().handle_key_press(key);
        }
    }

    fn update(&mut self) -> bool {
        todo!()
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let (datetime, number, channel, digitiser_id) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(48), Constraint::Max(24), Constraint::Max(24), Constraint::Max(24)])
                .split(area);
            (chunk[0], chunk[1], chunk[2], chunk[3])
        };

        let (date, time) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(datetime);
            (chunk[0], chunk[1])
        };

        self.date.render(frame, date);
        self.time.render(frame, time);
        //self.timestamp.render(frame, timestamp);
        self.number.render(frame, number);
        self.channel.render(frame, channel);
        self.digitiser_id.render(frame, digitiser_id);
    }
}
