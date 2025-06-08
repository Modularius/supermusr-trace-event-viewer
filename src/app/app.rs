use std::io::Stdout;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    Frame,
};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};
use tokio::sync::{mpsc, oneshot};

use crate::{
    app::{results::Results, setup::Setup}, finder::{MessageFinder, SearchStatus}, tui::{Component, ComponentContainer, FocusableComponent, TextBox, TuiComponent}, Select
};

#[derive(Default, Debug, Clone, EnumIter)]
enum Focus {
    #[default]
    Setup,
    Results,
}

#[derive(Default, EnumString, Display)]
enum StatusMessage {
    #[default]
    #[strum(to_string = "Ready to Search. Press <Enter> to begin.")]
    Waiting,
    #[strum(to_string = "Searching Begun. Press <Esc> to halt.")]
    SearchBegun,
    #[strum(to_string = "Searching for Traces: {0}/{1}. Press <Esc> to halt.")]
    TraceSearchInProgress(u32,u32),
    #[strum(to_string = "Searching for Event Lists: {0}/{1}. Press <Esc> to halt.")]
    EventListSearchInProgress(u32,u32),
    #[strum(to_string = "Search Halted. Press <Enter> to search again.")]
    SearchHalted,
    #[strum(to_string = "Search Complete. Press <Enter> to search again.")]
    SearchFinished
}

#[derive(Default)]
struct SearchTools {
    send_halt: Option<oneshot::Sender<()>>,
    recv_status: Option<mpsc::Receiver<SearchStatus>>,
}

pub(crate) struct App<M> {
    quit: bool,
    is_changed: bool,
    message_finder: M,
    search: SearchTools,
    focus: Focus,
    setup: TuiComponent<Setup>,
    status: TuiComponent<TextBox<StatusMessage>>,
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
            search: Default::default(),
            setup: Setup::new(select.timestamp),
            status: TextBox::new(Default::default(), Some("Status")),
            results: Results::new(),
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

    pub(crate) async fn run(&mut self) {
        if let Some(recv_status) = self.search.recv_status.as_mut() {
            if let Some(status) = recv_status.recv().await {
                match status {
                    SearchStatus::Off => self.status.underlying_mut().set(StatusMessage::SearchFinished),
                    SearchStatus::TraceSearchInProgress(prog, total) => self.status.underlying_mut().set(StatusMessage::TraceSearchInProgress(prog, total)),
                    SearchStatus::EventListSearchInProgress(prog, total) => self.status.underlying_mut().set(StatusMessage::EventListSearchInProgress(prog, total)),
                    SearchStatus::Halted => {
                        self.status.underlying_mut().set(StatusMessage::SearchHalted);
                        self.search = Default::default();
                    },
                    SearchStatus::Successful(cache) => {
                        self.status.underlying_mut().set(StatusMessage::SearchFinished);
                        self.search = Default::default();
                        self.results.underlying_mut().push(cache);
                    },
                };
            }
        }
        self.message_finder.retrieve_consumer();
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
        if key == KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE) {
            self.quit = true;
        } else if key == KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE) {
            self.focused_component_mut().set_focus(false);

            self.focus = Focus::iter()
                .cycle()
                .skip(self.focus.clone() as usize + 1)
                .next()
                .expect("");

            self.focused_component_mut().set_focus(true);
        } else if key == KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE) {
            if let Some(send_halt) = self.search.send_halt.take() {
                send_halt.send(()).expect("Send halt should not fail.");
            }
        } else if key == KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE) {
            match self.focus {
                Focus::Setup => {
                    if self.search.recv_status.is_none() {
                        let resp = self.setup
                            .underlying_mut()
                            .search(&mut self.message_finder)
                            .expect("");
                        self.search.recv_status = Some(resp.recv_status);
                        self.search.send_halt = Some(resp.send_halt);

                        self.status.underlying_mut().set(StatusMessage::SearchBegun);
                    }
                }
                Focus::Results => {}
            }
        } else {
            self.focused_component_mut().handle_key_press(key);
        }
        self.is_changed = true;
    }

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let (setup, status, results, help) = {
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(6), Constraint::Length(4), Constraint::Min(8), Constraint::Length(4)])
                .split(area);
            (chunk[0], chunk[1], chunk[2], chunk[3])
        };
        self.setup.render(frame, setup);
        self.status.render(frame, status);
        self.results.render(frame, results);
        self.help.render(frame, help);
    }
}
