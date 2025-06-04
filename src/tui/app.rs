use std::io::Stdout;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{buffer::Buffer, layout::{Alignment, Constraint, Direction, Layout, Rect}, prelude::CrosstermBackend, style::{self, Color, Style}, symbols, text::{self, Text}, widgets::{block::Title, Block, Borders, List, ListItem, Paragraph, Widget}, Frame};
use rdkafka::consumer::BaseConsumer;

use crate::{tui::{controls::Controls, graph::Graph, results::Results, setup::Setup, Component}, Cache, Topics};

#[derive(Default)]
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
    focus: Focus
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
            setup: Setup,
            controls: Controls,
            results: Results<'a>,
            graph: Graph<'a>,
        }
    }

    pub(crate) fn is_quit(&self) -> bool {
        self.quit
    }
}

impl<'a> Component for App<'a> {
    fn changed(&self) -> bool {
        self.changed
    }

    fn acknowledge_change(&mut self) {
        self.changed = false;
    }

    fn handle_key_press(&mut self, key: KeyEvent) -> bool {
        if key == KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE) {
            self.quit = true;
        } else if key == KeyEvent::new(KeyCode::Up, KeyModifiers::NONE) {

        } else if key == KeyEvent::new(KeyCode::Down, KeyModifiers::NONE) {

        } else if key == KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE) {
            self.focus = match self.focus {
                Focus::Setup => Focus::Controls,
                Focus::Controls => Focus::Results,
                Focus::Results => Focus::Setup,
            };
            self.changed = true;
        } else {
            let has_changed = match self.focus {
                Focus::Setup => self.controls.,
                Focus::Controls => Focus::Results,
                Focus::Results => Focus::Setup,
            };
        }
    }

    fn update(&mut self) {
    }

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let (setup, display) = {
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(4), Constraint::Min(0)])
                .split(area);
                (chunk[0], chunk[1])
        };

        {
            let block = Block::new()
                .title(Title::default().alignment(Alignment::Center).content("Setup"))
                .borders(Borders::ALL)
                .border_style(Style::new().fg(match self.focus { Focus::Setup => Color::Blue, _ => Color::Black}))
                .style(Style::new().bg(Color::Gray));
            
            frame.render_widget(block, setup);
        }
        
        {
            let block = Block::new()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(match self.focus { Focus::Controls => Color::Blue, _ => Color::Black}))
                .style(Style::new().bg(Color::Gray));

            self.controls.render(frame,controls);
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
                .border_style(Style::new().fg(match self.focus { Focus::Results => Color::Blue, _ => Color::Black }))
                .style(Style::new().bg(Color::Gray));
            
            let list = List::new([ListItem::new("Null"), ListItem::new("Also Null")])
                .block(block)
                .highlight_style(Style::new().bg(Color::Cyan))
                .highlight_symbol(">");

            self.results.render(frame, block.inner(results));
            frame.render_widget(list, results);
        }

        {
            let block = Block::new()
                .title(Title::default().alignment(Alignment::Center).content("Graph"))
                .borders(Borders::all())
                .border_style(Style::new().fg(Color::DarkGray))
                .style(Style::new().bg(Color::Gray));
            
            self.graph.render(frame, block.inner(graph));
            

            self.graph.render(frame,display);
            frame.render_widget(block, graph);
        }
    }
}