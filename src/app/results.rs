use std::io::Stdout;

use crossterm::event::KeyEvent;
use ratatui::{layout::{Alignment, Constraint, Direction, Layout, Rect}, prelude::CrosstermBackend, style::{Color, Style}, widgets::Paragraph, Frame};

use crate::{
    messages::{Cache, DigitiserMetadata, DigitiserTrace}, tui::{Channels, ComponentStyle, FocusableComponent, Graph, ListBox, TuiComponent, TuiComponentBuilder}, Component
};

pub(crate) struct Results {
    cache: Option<Cache>,
    list: TuiComponent<ListBox<String>>,
    channels: TuiComponent<Channels>,
    graph: TuiComponent<Graph>
}

impl Results {
    pub(crate) fn new() -> TuiComponent<Self> {
        TuiComponentBuilder::new(ComponentStyle::selectable()).build(Self {
            cache: None,
            list: ListBox::new(&vec![], Some("Traces")),
            graph: Graph::new(&vec![], None),
            channels: Channels::new(None),
        })
    }

    pub(crate) fn push(&mut self, cache: Cache) {
        self.cache = Some(cache);
        let list = self.cache.as_ref().expect("").iter_traces().map(|(metadata,trace)|{
            format!("[{}]\nid: {}, num channels {}, num_bins: {}",
                metadata.timestamp,
                metadata.id,
                trace.traces.len(),
                trace.traces
                    .iter()
                    .map(|(_,t)|t.len()).max().unwrap_or_default()
                )
        }).collect();
        self.list.underlying_mut().set(list);
    }

    pub(crate) fn select(&mut self) -> Option<(&DigitiserMetadata, &DigitiserTrace)> {
        let pair = self.cache
            .as_ref()
            .and_then(|cache|
                self.list
                    .underlying_mut()
                    .select()
                    .and_then(|i|cache.iter_traces().nth(i))
                );
        match pair.as_ref() {
            Some((_,trace)) => {
                if let Some(channel) = self.channels.underlying().get() {
                    self.graph.underlying_mut().set(trace.traces[&channel].iter().enumerate().map(|(t,&x)|(t,x)).collect());
                }
            },
            None => self.graph.underlying_mut().set(vec![]),
        }
        pair
    }
}

impl FocusableComponent for Results {
    fn set_focus(&mut self, focus: bool) {
        self.list.set_focus(focus);
        self.channels.set_focus(focus);
        self.graph.set_focus(focus);
        self.propagate_parental_focus(focus);
    }

    fn propagate_parental_focus(&mut self, focus: bool) {
        self.list.propagate_parental_focus(focus);
        self.channels.propagate_parental_focus(focus);
        self.graph.propagate_parental_focus(focus);
    }
}

impl Component for Results {
    fn handle_key_press(&mut self, key: KeyEvent) {
        if let Some(cache) = &self.cache {
            self.list.handle_key_press(key);
            self.channels.handle_key_press(key);

            if self.list.underlying_mut().pop_state_change() {
                let channels = self.list
                    .underlying_mut()
                    .select()
                    .and_then(|i|cache
                        .iter_traces()
                        .nth(i)
                    ).map(|(_, trace)|trace.traces
                        .keys()
                        .copied()
                        .collect::<Vec<_>>()
                    ).unwrap_or_default();
                self.channels.underlying_mut().set(channels);
            }
        }
    }

    fn update(&mut self) -> bool {
        todo!()
    }

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        if let Some(cache) = self.cache.as_ref() {
            let (panel, graph) = {
                let chunk = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(48), Constraint::Min(64)])
                    .split(area);
                (chunk[0], chunk[1])
            };

            let (info, list, channels) = {
                let chunk = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(4), Constraint::Min(4), Constraint::Length(4)])
                    .split(panel);
                (chunk[0], chunk[1], chunk[2])
            };

            let number = Paragraph::new(format!("Number of traces/events: {}/{}", cache.iter_traces().len(),cache.iter_events().len()))
                .alignment(Alignment::Left)
                .style(Style::new().fg(Color::White).bg(Color::Black));
            frame.render_widget(number, info);

            self.list.render(frame, list);
            self.channels.render(frame, channels);
            
            self.graph.render(frame, graph);
        }
    }
}
