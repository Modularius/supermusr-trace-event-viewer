use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, LineGauge},
    Frame,
};
use strum::{Display, EnumString};
use tracing::info;

use crate::{
    finder::{SearchResults, SearchStatus},
    messages::Cache,
    tui::{ComponentStyle, ParentalFocusComponent, TextBox, TuiComponent, TuiComponentBuilder},
    Component, Select,
};

#[derive(Default, EnumString, Display)]
enum StatusMessage {
    #[default]
    #[strum(to_string = "Ready to Search. Press <Enter> to begin.")]
    Waiting,
    #[strum(to_string = "Searching Begun. Press <Esc> to halt.")]
    SearchBegun,
    #[strum(to_string = "Searching for Traces.")]
    TraceSearchInProgress,
    #[strum(to_string = "Search for Trace Finished.")]
    TraceSearchFinished,
    #[strum(to_string = "Searching for Event Lists.")]
    EventListSearchInProgress,
    #[strum(to_string = "Search for Event Lists Finished.")]
    EventListSearchFinished,
    #[strum(to_string = "Search Halted. Press <Enter> to search again.")]
    SearchHalted,
    #[strum(to_string = "Search Complete. Press <Enter> to search again.")]
    SearchFinished,
    #[strum(to_string = "{0}")]
    Text(String),
}

pub(crate) struct Statusbar {
    parent_has_focus: bool,
    info: TuiComponent<TextBox<String>>,
    status: TuiComponent<TextBox<StatusMessage>>,
    progress_steps: u32,
    num_step_passes: u32,
    total_steps: u32,
}

impl Statusbar {
    pub(crate) fn new(select: &Select) -> TuiComponent<Self> {
        TuiComponentBuilder::new(ComponentStyle::selectable())
            .is_in_block(true)
            .build(Self {
                parent_has_focus: false,
                info: TextBox::new(Default::default(), None),
                status: TextBox::new(Default::default(), Some("Status")),
                progress_steps: 0,
                num_step_passes: select.step.num_step_passes,
                total_steps: 2 * select.step.num_step_passes + 4,
            })
    }

    pub(crate) fn set_status(&mut self, status: SearchStatus) {
        match status {
            SearchStatus::Off => self.status.set(StatusMessage::SearchFinished),
            SearchStatus::TraceSearchInProgress(prog) => {
                self.status.set(StatusMessage::TraceSearchInProgress);
                self.progress_steps = prog + 1;
            }
            SearchStatus::TraceSearchFinished => {
                self.status.set(StatusMessage::TraceSearchFinished);
                self.progress_steps = self.num_step_passes + 2;
            }
            SearchStatus::EventListSearchInProgress(prog) => {
                self.status.set(StatusMessage::EventListSearchFinished);
                self.progress_steps = prog + self.num_step_passes + 2;
            }
            SearchStatus::Halted => self.status.set(StatusMessage::SearchHalted),
            SearchStatus::Successful => {
                self.status.set(StatusMessage::SearchFinished);
                self.progress_steps = 2 * self.num_step_passes + 3;
            }
            SearchStatus::Text(text) => self.status.set(StatusMessage::Text(text)),
            SearchStatus::EventListSearchFinished => {}
        }
        info!("{0}", self.status.get());
        info!("{0}", self.progress_steps);
    }

    pub(crate) fn set_info(&mut self, results: &SearchResults) {
        self.info.set(format!(
            "Found {} traces, in {},{} ms",
            results.cache.iter_traces().len(),
            results.time.num_seconds(),
            results.time.subsec_millis()
        ));
    }
}

impl Component for Statusbar {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let (info, status, progress) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(50),
                    Constraint::Min(32),
                    Constraint::Length(24),
                ])
                .split(area);
            (chunk[0], chunk[1], chunk[2])
        };

        let gauge = LineGauge::default()
            .block(Block::new().borders(Borders::ALL))
            .style(Style::new().fg(Color::LightGreen).bg(Color::Black))
            .ratio(self.progress_steps as f64 / self.total_steps as f64);

        self.info.render(frame, info);
        self.status.render(frame, status);
        frame.render_widget(gauge, progress);
    }
}

impl ParentalFocusComponent for Statusbar {
    fn propagate_parental_focus(&mut self, focus: bool) {
        self.parent_has_focus = focus;
    }
}
