use std::io::Stdout;

use chrono::Utc;
use ratatui::{layout::Rect, prelude::CrosstermBackend, Frame};

use crate::{
    finder::{InitSearchResponse, MessageFinder, SearchTarget},
    tui::{ComponentStyle, FocusableComponent, TuiComponent, TuiComponentBuilder},
    Component,
};

pub(crate) struct Setup {}

impl Setup {
    pub(crate) fn new() -> TuiComponent<Self> {
        TuiComponentBuilder::new(ComponentStyle::default()).build(Self {})
    }

    pub(crate) fn search<M: MessageFinder>(
        &self,
        message_finder: &mut M,
    ) -> Option<InitSearchResponse> {
        message_finder.init_search(SearchTarget {
            timestamp: Utc::now(),
            channels: vec![],
            digitiser_ids: vec![],
        })
    }
}

impl FocusableComponent for Setup {
    fn set_focus(&mut self, focus: bool) {
        todo!()
    }

    fn propagate_parental_focus(&mut self, focus: bool) {
        todo!()
    }
}

impl Component for Setup {
    fn handle_key_press(&mut self, key: crossterm::event::KeyEvent) {
        todo!()
    }

    fn update(&mut self) -> bool {
        todo!()
    }

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        todo!()
    }
}
