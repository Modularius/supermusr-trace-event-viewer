use std::io::Stdout;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    Frame,
};
use strum::{EnumIter, IntoEnumIterator};
use tracing::info;

use crate::{
    app::{results::Results, setup::Setup}, finder::{MessageFinder, SearchStatus}, tui::{Component, ComponentContainer, FocusableComponent, TextBox, TuiComponent}, Select
};

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
    focus: Focus,
    setup: TuiComponent<Setup>,
    results: TuiComponent<Results>,
    help: TuiComponent<TextBox<String>>,
}

impl<'a, M: MessageFinder> App<M> {
    pub(crate) fn new(message_finder: M, select: &Select) -> Self {
        let mut app = App {
            quit: false,
            is_changed: true,
            message_finder,
            focus: Default::default(),
            setup: Setup::new(select.timestamp),
            results: Results::new(select),
            help: TextBox::new(Default::default(), None),
        };
        app.focused_component_mut().set_focus(true);
        app
    }

    pub(crate) fn changed(&self) -> bool {
        self.is_changed
    }

    pub(crate) fn is_quit(&self) -> bool {
        self.quit
    }

    pub(crate) async fn async_run(&mut self) {
        self.message_finder.run().await;
    }

    pub(crate) fn run(&mut self) {
        if let Some(status) = self.message_finder.status() {
            self.results.underlying_mut().set_status(status);
            self.is_changed = true;
        }
        if let Some(cache) = self.message_finder.cache() {
            self.results.underlying_mut().push(cache);
            self.is_changed = true;
        }
    }
}

impl<'a, M: MessageFinder> ComponentContainer for App<M> {
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

impl<'a, M: MessageFinder> Component for App<M> {
    fn handle_key_press(&mut self, key: KeyEvent) {
        if key == KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE) {
            self.quit = true;
        } else if key == KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE) {
            self.focused_component_mut().set_focus(false);

            self.focus = Focus::iter()
                .cycle()
                .skip(self.focus.clone() as usize + 1)
                .next()
                .expect("");

            self.focused_component_mut().set_focus(true);
        } else if key == KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE) {
            match self.focus {
                Focus::Setup => {
                    self.setup.underlying_mut()
                        .search(&mut self.message_finder);
                }
                Focus::Results => {
                    if let Some((metadata, trace)) = self.results.underlying_mut().select() {
                        
                    }
                }
            }
        } else {
            self.focused_component_mut().handle_key_press(key);
        }
        self.is_changed = true;
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let (setup, results, help) = {
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(6), Constraint::Min(8), Constraint::Length(3)])
                .split(area);
            (chunk[0], chunk[1], chunk[2])
        };
        self.setup.render(frame, setup);
        self.results.render(frame, results);
        self.help.render(frame, help);
    }
}
