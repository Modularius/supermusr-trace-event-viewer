use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use strum::{EnumCount, EnumIter};
use supermusr_common::Time;

use crate::{
    app::{Display, Results, Setup},
    finder::MessageFinder,
    graphics::{Bound, Bounds, FileFormat, GraphSaver},
    messages::Cache,
    tui::{
        Component, ComponentContainer, FocusableComponent, InputComponent, Statusbar, TextBox,
        TuiComponent,
    },
    Select,
};

pub(crate) trait AppDependencies {
    type MessageFinder: MessageFinder;
    type GraphSaver: GraphSaver;
}

#[derive(Default, Debug, Clone, EnumIter, EnumCount)]
pub(crate) enum Focus {
    #[default]
    Setup,
    Results,
    Display,
}

pub(crate) struct App<D: AppDependencies> {
    ///
    cache: Option<Cache>,
    /// Flag indicating the program should quit.
    quit: bool,
    /// Flag indicating the app should be redrawn.
    is_changed: bool,
    message_finder: D::MessageFinder,
    focus: Focus,
    setup: TuiComponent<Setup>,
    status: TuiComponent<Statusbar>,
    results: TuiComponent<Results>,
    display: TuiComponent<Display>,
    help: TuiComponent<TextBox<String>>,
}

impl<'a, D: AppDependencies> App<D> {
    /// Creates a new App instance.
    ///
    /// # Attributes
    /// - message_finder: TODO
    /// - select: TODO
    pub(crate) fn new(message_finder: D::MessageFinder, select: &Select) -> Self {
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

    /// Returns whether the app has changed and needs to be redrawn.
    pub(crate) fn changed(&self) -> bool {
        self.is_changed
    }

    /// Returns whether the `quit` flag has been set.
    pub(crate) fn is_quit(&self) -> bool {
        self.quit
    }

    /// Updates the search engine.
    ///
    /// This function is called asynchronously,
    /// hence it cannot be part of [Self::update].
    pub(crate) async fn async_update(&mut self) {
        self.message_finder.update().await;
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
        if let Some(cache) = self.message_finder.results() {
            self.results.new_cache(&cache.cache);
            self.status.set_info(&cache);

            // Take ownership of the cache
            self.cache = Some(cache.cache);

            self.is_changed = true;
        }

        // If there is a message cache available, call update on [Self::results].
        if let Some(cache) = &self.cache {
            self.results.update(cache);
        }
    }
}

impl<D: AppDependencies> ComponentContainer for App<D> {
    type Focus = Focus;

    fn get_focused_component_mut(&mut self, focus: Self::Focus) -> &mut dyn FocusableComponent {
        match focus {
            Focus::Setup => &mut self.setup,
            Focus::Results => &mut self.results,
            Focus::Display => &mut self.display,
        }
    }

    fn get_focus(&self) -> Self::Focus {
        self.focus.clone()
    }

    fn set_focus(&mut self, focus: Self::Focus) {
        self.focus = focus;
    }
}

impl<D: AppDependencies> Component for App<D> {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let (setup, status, results_display, help) = {
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(8),
                    Constraint::Length(5),
                    Constraint::Min(8),
                    Constraint::Length(3),
                ])
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

impl<D: AppDependencies> InputComponent for App<D> {
    fn handle_key_press(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Esc {
            self.quit = true;
        } else if key == KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT) {
            self.set_focus_index(self.focus.clone() as isize - 1);
        } else if key == KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE) {
            self.set_focus_index(self.focus.clone() as isize + 1)
        } else if key.code == KeyCode::Enter {
            match self.focus {
                Focus::Setup => {
                    self.setup.search(&mut self.message_finder);
                }
                Focus::Results => {
                    if let Some(cache) = &self.cache {
                        if let Some((_, trace, channel)) = self.results.select(cache) {
                            self.display.select(
                                trace.traces.get(&channel).expect(""),
                                trace
                                    .events
                                    .as_ref()
                                    .and_then(|events| events.get(&channel)),
                            );
                        }
                    }
                }
                Focus::Display => {
                    if let Some(cache) = &self.cache {
                        if let Some((metadata, trace, channel)) = self.results.select(cache) {
                            D::GraphSaver::save_as_svg(
                                trace,
                                vec![channel],
                                FileFormat::Svg
                                    .build_path(&self.setup.get_path(), metadata, channel)
                                    .expect(""),
                                self.setup.get_image_size(),
                                Bounds {
                                    time: Bound::from(
                                        1.0,
                                        [0, trace.traces[&channel].len() as Time].into_iter(),
                                    ),
                                    intensity: Bound::from(
                                        1.0,
                                        trace.traces[&channel].iter().copied(),
                                    ),
                                },
                            )
                            .expect("");
                        }
                    }
                }
            }
        } else {
            self.focused_component_mut().handle_key_press(key);
        }
        self.is_changed = true;
    }
}
