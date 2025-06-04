use std::io::Stdout;

use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{layout::Rect, prelude::CrosstermBackend, style::{Color, Style}, widgets::List, Frame};
use rdkafka::consumer::BaseConsumer;

use crate::{data::{DigitiserMetadata, DigitiserTrace}, Cache, Component};

use supermusr_common::{Channel, DigitizerId};

#[derive(Default)]
enum Focus {
    #[default]
    TimestampFrom,
    TimestampTo,
    Channels,
    Digitisers,
}

pub(crate) struct Setup{
    changed: bool,
    focus: Focus,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    channels: Vec<Channel>,
    digitisers: Vec<DigitizerId>
}

impl Setup {
    pub(crate) fn new() -> Self {
        Setup {
            changed: true,
            focus: Default::default(),
            from: Default::default(),
            to: Default::default(),
            channels: Default::default(),
            digitisers: Default::default(),
        }
    }
}

impl Component for Setup {
    fn changed(&self) -> bool {
        self.changed
    }

    fn acknowledge_change(&mut self) {
        self.changed = false;
    }

    fn handle_key_press(&mut self, key: KeyEvent) {
        if key == KeyEvent::new(KeyCode::Up, KeyModifiers::NONE) {

        } else if key == KeyEvent::new(KeyCode::Down, KeyModifiers::NONE) {

        } else if key == KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE) {
            self.changed = true;
        } else {
        }
    }

    fn update(&mut self) {
    }

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        
    }
}