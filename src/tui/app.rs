use std::io::Stdout;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    Frame,
};
use rdkafka::consumer::BaseConsumer;

use crate::{
    tui::{controls::Controls, graph::Graph, results::Results, setup::Setup, traits::{ComponentContainer}, Component, TuiComponent},
    Cache, Topics,
};

use strum::{EnumIter, IntoEnumIterator};

#[derive(Default, Debug, Clone, EnumIter)]
enum Focus {
    #[default]
    Setup,
    Results,
}

pub(crate) struct App<'a> {
    changed: bool,
    quit: bool,
    consumer: &'a BaseConsumer,
    topics: &'a Topics,
    cache: Cache,
    focus: Focus,
    setup: TuiComponent<Setup>,
    results: TuiComponent<Results>,
    graph: TuiComponent<Graph<'a>>,
    help: TuiComponent<Controls>,
}

impl<'a> App<'a> {
    pub(crate) fn new(consumer: &'a BaseConsumer, topics: &'a Topics) -> Self {
        App {
            changed: true,
            quit: false,
            consumer,
            topics,
            cache: Cache::default(),
            focus: Default::default(),
            setup: Setup::new(),
            results: Results::new(),
            graph: Graph::new(),
            help: Controls::new(),
        }
    }

    pub(crate) fn is_quit(&self) -> bool {
        self.quit
    }
}

impl<'a> ComponentContainer for App<'a> {
    fn focused_component(&self) -> &dyn Component {
        match self.focus {
            Focus::Setup => &self.setup,
            Focus::Results => &self.results,
        }
    }

    fn focused_component_mut(&mut self) -> &mut dyn Component {
        match self.focus {
            Focus::Setup => &mut self.setup,
            Focus::Results => &mut self.results,
        }
    }
}

impl<'a> Component for App<'a> {
    fn changed(&self) -> bool {
        self.changed
    }

    fn acknowledge_change(&mut self) {
        self.setup.acknowledge_change();
        self.results.acknowledge_change();
        self.help.acknowledge_change();
        self.changed = false;
    }

    fn give_focus(&mut self) {
        self.acknowledge_focus();
        self.focused_component_mut().give_focus();
        let text = self.focused_component().help();
        self.help.underlying_mut().set(text)
    }
    
    fn acknowledge_focus(&mut self) {
        self.setup.acknowledge_focus();
        self.results.acknowledge_focus();
        self.help.acknowledge_focus();
    }

    fn handle_key_press(&mut self, key: KeyEvent) {
        if key == KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE) {
            self.quit = true;
        } else if key == KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE) {
            self.focus = Focus::iter().cycle().skip(self.focus.clone() as usize + 1).next().expect("");
            self.give_focus();
            self.changed = true;
        } else {
            self.focused_component_mut().handle_key_press(key);
            self.changed = self.focused_component().changed();
        }
    }

    fn update(&mut self) {}

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let (setup, display, help) = {
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(8),
                    Constraint::Length(4),
                    Constraint::Min(0),
                ])
                .split(area);
            (chunk[0], chunk[1], chunk[2])
        };
        self.setup.render(frame, setup);

        let (results, graph) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(32), Constraint::Min(0)])
                .split(display);
            (chunk[0], chunk[1])
        };
        self.results.render(frame, results);
        self.graph.render(frame, graph);

        self.help.render(frame, help);
    }
}
