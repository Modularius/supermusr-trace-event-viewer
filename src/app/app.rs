use std::io::Stdout;

use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    Frame,
};

use crate::{
    app::{results::Results, setup::Setup}, finder::{InitSearchResponse, MessageFinder, SearchTarget}, tui::{Component, ComponentContainer, FocusableComponent, TuiComponent}
};

use strum::{EnumIter, IntoEnumIterator};

#[derive(Default, Debug, Clone, EnumIter)]
enum Focus {
    #[default]
    Setup,
    Results,
}

pub(crate) struct App<M> {
    quit: bool,
    is_changed: bool,
    message_finder: M,
    search: Option<InitSearchResponse>,
    focus: Focus,
    setup: TuiComponent<Setup>,
    results: TuiComponent<Results>,
    //graph: TuiComponent<Graph<'a>>,
    //help: TuiComponent<Controls>,
}

impl<'a, M : MessageFinder> App<M> {
    pub(crate) fn new(message_finder: M) -> Self {
        App {
            quit: false,
            is_changed: true,
            message_finder,
            focus: Default::default(),
            search: None,
            setup: Setup::new(),
            results: Results::new(),
            //graph: Graph::new(),
            //help: Controls::new(),
        }
    }

    pub(crate) fn changed(&self) -> bool {
        self.is_changed
    }

    pub(crate) fn is_quit(&self) -> bool {
        self.quit
    }
}

impl<'a,M> ComponentContainer for App<M> {
    fn focused_component(&self) -> &dyn FocusableComponent {
        match self.focus {
            Focus::Setup => &self.setup,
            Focus::Results => &self.results,
        }
    }

    fn focused_component_mut(&mut self) -> &mut dyn FocusableComponent {
        match self.focus {
            Focus::Setup => &mut self.setup,
            Focus::Results => &mut self.results,
        }
    }
}

impl<'a, M : MessageFinder> Component for App<M> {
    fn handle_key_press(&mut self, key: KeyEvent) {
        if key == KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE) {
            self.quit = true;
        } else if key == KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE) {
            
            self.focused_component_mut()
                .set_focus(false);

            self.focus = Focus::iter().cycle()
                .skip(self.focus.clone() as usize + 1)
                .next()
                .expect("");

            self.focused_component_mut()
                .set_focus(true);

        } else if key == KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE) {
            if let Some(search) = self.search.take() {
                search.send_halt.send(()).expect("");
            }
        } else if key == KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE) {
            match self.focus {
                Focus::Setup => {
                    if self.search.is_none() {
                        self.search = self.setup
                            .underlying_mut()
                            .search(&mut self.message_finder);
                    }
                },
                Focus::Results => {

                },
            }
        } else {
            self.focused_component_mut().handle_key_press(key);
        }
        self.is_changed = true;
    }

    fn update(&mut self) -> bool {
        //self.setup.update();
        if self.results.update() {
            self.is_changed = true;
        }
        self.is_changed
    }

    /*
    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let (setup, display, help) = {
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(8),
                    Constraint::Min(0),
                    Constraint::Length(4),
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
    } */
   fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let (setup, results) = {
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4),
                    Constraint::Min(4),
                ])
                .split(area);
            (chunk[0], chunk[1])
        };
        self.setup.render(frame, setup);

        self.results.render(frame, results);
    } 
}
