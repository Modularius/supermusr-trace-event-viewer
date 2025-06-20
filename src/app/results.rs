use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use supermusr_common::Channel;

use crate::{
    messages::{Cache, DigitiserMetadata, DigitiserTrace},
    tui::{
        Channels, ComponentStyle, FocusableComponent, InputComponent, ListBox,
        ParentalFocusComponent, TuiComponent, TuiComponentBuilder,
    },
    Component,
};

pub(crate) struct Results {
    list: TuiComponent<ListBox<String>>,
    channels: TuiComponent<Channels>,
}

impl Results {
    pub(crate) fn new() -> TuiComponent<Self> {
        TuiComponentBuilder::new(ComponentStyle::selectable()).build(Self {
            list: ListBox::new(&vec![], Some("Traces"), None),
            channels: Channels::new(),
        })
    }

    pub(crate) fn new_cache(&mut self, cache: &Cache) {
        let list = cache
            .iter_traces()
            .map(|(metadata, trace)| {
                format!(
                    "[{}]\nid: {}, num channels {}, num_bins: {}",
                    metadata.timestamp,
                    metadata.id,
                    trace.traces.len(),
                    trace
                        .traces
                        .iter()
                        .map(|(_, t)| t.len())
                        .max()
                        .unwrap_or_default()
                )
            })
            .collect();
        self.list.set(list);
    }

    pub(crate) fn select<'a>(
        &mut self,
        cache: &'a Cache,
    ) -> Option<(&'a DigitiserMetadata, &'a DigitiserTrace, Channel)> {
        self.list
            .get_index()
            .and_then(|i| cache.iter_traces().nth(i))
            .and_then(|(m, t)| self.channels.get().map(|c| (m, t, c)))
    }

    ///
    pub(crate) fn update(&mut self, cache: &Cache) {
        if self.list.pop_state_change() {
            let channels = self
                .list
                .get_index()
                .and_then(|i| cache.iter_traces().nth(i))
                .map(|(_, trace)| trace.traces.keys().copied().collect::<Vec<_>>())
                .unwrap_or_default();
            self.channels.set(channels);
        }
    }
}

impl Component for Results {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let (list, channels) = {
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(4), Constraint::Length(3)])
                .split(area);
            (chunk[0], chunk[1])
        };

        self.list.render(frame, list);
        self.channels.render(frame, channels);
        //}
    }
}

impl InputComponent for Results {
    fn handle_key_press(&mut self, key: KeyEvent) {
        self.list.handle_key_press(key);
        self.channels.handle_key_press(key);
    }
}

impl FocusableComponent for Results {
    fn set_focus(&mut self, focus: bool) {
        self.list.set_focus(focus);
        self.channels.set_focus(focus);
        self.propagate_parental_focus(focus);
    }
}

impl ParentalFocusComponent for Results {
    fn propagate_parental_focus(&mut self, focus: bool) {
        self.list.propagate_parental_focus(focus);
        self.channels.propagate_parental_focus(focus);
    }
}
