use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use strum::{EnumCount, EnumIter};

use crate::{
    app::{Display, Results, Setup},
    finder::MessageFinder,
    graphics::GraphSaver,
    messages::Cache,
    tui::{Component, ComponentContainer, FocusableComponent, InputComponent, Statusbar, TextBox, TuiComponent},
    Select
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

pub(crate) struct App<D : AppDependencies> {
    cache: Option<Cache>,
    quit: bool,
    is_changed: bool,
    message_finder: D::MessageFinder,
    graph_saver: D::GraphSaver,
    focus: Focus,
    setup: TuiComponent<Setup>,
    status: TuiComponent<Statusbar>,
    results: TuiComponent<Results>,
    display: TuiComponent<Display>,
    help: TuiComponent<TextBox<String>>,
}

impl<'a, D: AppDependencies> App<D> {
    pub(crate) fn new(message_finder: D::MessageFinder, select: &Select) -> Self {
        let mut app = App {
            quit: false,
            is_changed: true,
            cache: None,
            message_finder,
            graph_saver: Default::default(),
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
                .constraints([Constraint::Length(8), Constraint::Length(5), Constraint::Min(8), Constraint::Length(3)])
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
        } else if key == KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL) {
            if let Some(cache) = &self.cache {
                if let Some((_, trace, channel)) = self.results.select(cache) {
                    D::GraphSaver::save_as_svg(trace, vec![channel], self.setup.get_path());
                    //let graph = BuildGraph::<BackendSVG<'_>>::new(800,600,bounds.time_range(), bounds.intensity_range());

                    //let path_buf = graph.build_path(&output_to_file.path, metadata, *channel).expect("extension should write");
                    //let eventlist = traces.events.as_ref().and_then(|ev|ev.get(channel));
                    //graph.save_trace_graph(&path_buf, &trace, eventlist).expect("");
                }
            }
        } else {
            self.focused_component_mut().handle_key_press(key);
        }
        self.is_changed = true;
    }
}