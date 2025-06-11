use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use strum::{EnumIter, IntoEnumIterator};

use crate::{
    app::{Display, Results, Setup},
    finder::MessageFinder,
    messages::Cache,
    tui::{Component, ComponentContainer, FocusableComponent, InputComponent, Statusbar, TextBox, TuiComponent},
    Select
};

#[derive(Default, Debug, Clone, EnumIter)]
enum Focus {
    #[default]
    Setup,
    Results,
    Display,
}

pub(crate) struct App<M> {
    cache: Option<Cache>,
    quit: bool,
    is_changed: bool,
    message_finder: M,
    focus: Focus,
    setup: TuiComponent<Setup>,
    status: TuiComponent<Statusbar>,
    results: TuiComponent<Results>,
    display: TuiComponent<Display>,
    help: TuiComponent<TextBox<String>>,
}

impl<'a, M: MessageFinder> App<M> {
    pub(crate) fn new(message_finder: M, select: &Select) -> Self {
        let mut app = App {
            quit: false,
            is_changed: true,
            cache: None,
            message_finder,
            focus: Default::default(),
            setup: Setup::new(select),
            status: Statusbar::new(select),
            results: Results::new(),
            display: Display::new(),
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

    /// Updates the search engine.
    /// 
    /// This function is called asynchronously,
    /// hence it cannot be part of [Self::update].
    pub(crate) async fn async_update(&mut self) {
        self.message_finder.run().await;
    }

    /// Causes the function to pop any status messages or results from the [MessageFinder],
    /// as well as calling update methods of some of the apps subcomponents.
    pub(crate) fn update(&mut self) {
        // If a status message is available, pop it from the [MessageFinder].
        if let Some(status) = self.message_finder.status() {
            self.status.set_status(status);
            self.is_changed = true;
        }
        // If a result is available, pop it from the [MessageFinder].
        if let Some(cache) = self.message_finder.cache() {
            self.results.new_cache(&cache);
            self.status.set_info(&cache);

            // Take ownership of the cache
            self.cache = Some(cache);

            self.is_changed = true;
        }

        // If there is a message cache available, call update on [Self::results].
        if let Some(cache) = &self.cache {
            self.results.update(cache);
        }
    }
}

impl<M: MessageFinder> ComponentContainer for App<M> {
    fn focused_component_mut(&mut self) -> &mut dyn FocusableComponent {
        match self.focus {
            Focus::Setup => &mut self.setup,
            Focus::Results => &mut self.results,
            Focus::Display => &mut self.display,
        }
    }
}

impl<M: MessageFinder> Component for App<M> {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let (setup, status, results_display, help) = {
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(6), Constraint::Length(5), Constraint::Min(8), Constraint::Length(3)])
                .split(area);
            (chunk[0], chunk[1], chunk[2], chunk[3])
        };

        
        let (results, graph) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(50), Constraint::Min(64)])
                .split(results_display);
            (chunk[0], chunk[1])
        };

        self.setup.render(frame, setup);
        self.status.render(frame, status);
        self.results.render(frame, results);
        self.display.render(frame, graph);
        self.help.render(frame, help);
    }
}

impl<M: MessageFinder> InputComponent for App<M> {
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
                    self.setup
                        .search(&mut self.message_finder);
                }
                Focus::Results => {
                    if let Some(cache) = &self.cache {
                        if let Some((_, trace, channel)) = self.results.select(cache) {
                            self.display.select(
                                trace.traces
                                    .get(&channel)
                                    .expect(""),
                                trace.events
                                    .as_ref()
                                    .and_then(|events|events.get(&channel)));

                            
                        }
                    }
                }
                Focus::Display => {
                }
            }
        } else {
            self.focused_component_mut().handle_key_press(key);
        }
        self.is_changed = true;
    }
}