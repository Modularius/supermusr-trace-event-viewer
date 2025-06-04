use std::io::Stdout;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{buffer::Buffer, layout::{Alignment, Constraint, Direction, Layout, Rect}, prelude::CrosstermBackend, style::{self, Color, Style}, symbols, text::{self, Text}, widgets::{block::Title, Block, Borders, List, ListItem, Paragraph, Widget}, Frame};
use rdkafka::consumer::BaseConsumer;

use crate::{tui::{controls::Controls, graph::Graph, results::Results, setup::Setup, Component}, Cache, Topics};

#[derive(Default, Clone)]
enum Focus {
    #[default]
    Setup,
    Controls,
    Results
}

pub(crate) struct App<'a> {
    changed: bool,
    quit: bool,
    consumer: &'a BaseConsumer,
    topics: &'a Topics,
    cache: Cache,
    focus: Focus,
    setup: Setup,
    controls: Controls,
    results: Results,
    graph: Graph<'a>
}

impl<'a> App<'a> {
    pub(crate) fn new(consumer: &'a BaseConsumer, topics: &'a Topics) -> Self {
        App{
            changed: true,
            quit: false,
            consumer,
            topics,
            cache: Cache::default(),
            focus: Default::default(),
            setup: Setup::new(),
            controls: Controls::new(),
            results: Results::new(),
            graph: Graph::new(),
        }
    }

    pub(crate) fn is_quit(&self) -> bool {
        self.quit
    }

    fn get_border_colour(&self, component: &Focus) -> Color {
        match (self.focus.clone(), component) {
            (Focus::Setup, Focus::Setup) => Color::Cyan,
            (Focus::Controls, Focus::Controls) => Color::Cyan,
            (Focus::Results, Focus::Results) => Color::Cyan,
            _ => Color::Black
        }
    }
}

impl<'a> Component for App<'a> {
    fn changed(&self) -> bool {
        self.changed
    }

    fn acknowledge_change(&mut self) {
        self.changed = false;
    }

    fn handle_key_press(&mut self, key: KeyEvent) {
        if key == KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE) {
            self.quit = true;
        } else if key == KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE) {
            self.focus = match self.focus {
                Focus::Setup => Focus::Controls,
                Focus::Controls => Focus::Results,
                Focus::Results => Focus::Setup,
            };
            self.changed = true;
        } else {
            match self.focus {
                Focus::Setup => self.setup.handle_key_press(key),
                Focus::Controls => self.controls.handle_key_press(key),
                Focus::Results => self.results.handle_key_press(key),
            }
            self.changed = self.setup.changed() || self.controls.changed() || self.results.changed();
        }
    }

    fn update(&mut self) {
    }

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let (setup, controls, display) = {
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(8), Constraint::Length(4), Constraint::Min(0)])
                .split(area);
                (chunk[0], chunk[1], chunk[2])
        };

        {
            let block = Block::new()
                .title(Title::default().alignment(Alignment::Center).content("Setup"))
                .borders(Borders::ALL)
                .border_style(Style::new().fg(self.get_border_colour(&Focus::Setup)))
                .style(Style::new().bg(Color::LightGreen));
            frame.render_widget(block.clone(), setup);
            self.setup.render(frame,block.inner(setup));
        }
        
        {
            let block = Block::new()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(self.get_border_colour(&Focus::Controls)))
                .style(Style::new().bg(Color::LightGreen));

            frame.render_widget(block.clone(), controls);
            self.controls.render(frame,block.inner(controls));
        }


        let (results, graph) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(32), Constraint::Min(0)])
                .split(display);
                (chunk[0], chunk[1])
        };

        {
            let block = Block::new()
                .title(Title::default().alignment(Alignment::Center).content("Results"))
                .borders(Borders::ALL)
                .border_style(Style::new().fg(self.get_border_colour(&Focus::Results)))
                .style(Style::new().bg(Color::LightGreen));
            
            frame.render_widget(block.clone(), results);
            self.results.render(frame, block.inner(results));
        }

        {
            let block = Block::new()
                .title(Title::default().alignment(Alignment::Center).content("Graph"))
                .borders(Borders::all())
                .border_style(Style::new().fg(Color::Black))
                .style(Style::new().bg(Color::LightGreen));
            
            frame.render_widget(block.clone(), graph);
            self.graph.render(frame, block.inner(graph));
        }
    }
}