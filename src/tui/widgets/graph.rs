use std::io::Stdout;

use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Color, Style},
    symbols::Marker,
    text::Span,
    widgets::{Axis, Chart, Dataset, GraphType},
    Frame
};
use supermusr_common::Intensity;

use crate::{messages::{Event, EventList, Trace}, tui::{ComponentStyle, FocusableComponent, TuiComponent, TuiComponentBuilder}, Component};

type D = (usize,Intensity);

pub(crate) struct Graph {
    has_focus: bool,
    parent_has_focus: bool,
    trace_data: EventList,
    event_data: Option<EventList>
}

impl Graph where {
    pub(crate) fn new() -> TuiComponent<Self> {
        TuiComponentBuilder::new(ComponentStyle::selectable())
            .is_in_block(true)
            .build(Self {
                trace_data: Default::default(),
                event_data: None,
                has_focus: false,
                parent_has_focus: false,
            }
        )
    }

    pub(crate) fn set(&mut self, trace_data: Vec<Event>, event_data: Option<EventList>) {
        self.trace_data = trace_data;
        self.event_data = event_data;
    }
}

impl FocusableComponent for Graph {
    fn set_focus(&mut self, focus: bool) {
        self.has_focus = focus;
    }

    fn propagate_parental_focus(&mut self, focus: bool) {
        self.parent_has_focus = focus;
    }
}

impl Component for Graph {
    fn handle_key_press(&mut self, key: KeyEvent) {
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let (_info, graph) = {
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(area);
            (chunk[0], chunk[1])
        };

        let trace_data_iter = self.trace_data.iter().copied();
        let trace_data = trace_data_iter.clone().map(|e|(e.time as f64, e.intensity as f64)).collect::<Vec<_>>();

        let trace_dataset = Dataset::default()
            .name("Trace")
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::new().fg(Color::Blue).bg(Color::Black))
            .data(&trace_data);

        let time = trace_data_iter.clone().map(|e|e.time);
        let min = time.clone().min().unwrap_or_default() as f64;
        let max = 1.0625*(time.max().unwrap_or_default() as f64);
        let num_labels = 5;
        let labels = (0..num_labels).map(|i|(max - min)*i as f64/num_labels as f64 + min).map(|v|Span::raw(v.to_string())).collect::<Vec<_>>();
        let x_axis = Axis::default()
            .title("Time")
            .bounds([min, max])
            .labels(labels);

        let values = trace_data_iter.map(|e|e.intensity);
        let min = values.clone().min().unwrap_or_default() as f64;
        let max = 1.125*(values.max().unwrap_or_default() as f64);
        let num_labels = 10;
        let labels = (0..num_labels).map(|i|(max - min)*i as f64/num_labels as f64 + min).map(|v|Span::raw(v.to_string())).collect::<Vec<_>>();

        let y_axis = Axis::default()
            .title("Intensity")
            .bounds([min, max])
            .labels(labels);

        
        let event_data = self.event_data.as_ref().map(|event_data| {
            let event_data_iter = event_data.iter().copied();
            event_data_iter.clone().map(|e|(e.time as f64, e.intensity as f64)).collect::<Vec<_>>()
        });
        let event_dataset = event_data.as_ref().map(|event_data| {
                Dataset::default()
                    .name("Events")
                    .marker(Marker::Dot)
                    .graph_type(GraphType::Scatter)
                    .style(Style::new().fg(Color::LightRed).bg(Color::Black))
                    .data(event_data.as_slice())
        });

        let datasets = if let Some(event_dataset) = event_dataset {
            vec![trace_dataset, event_dataset]
        } else {
            vec![trace_dataset]
        };
        let chart = Chart::new(datasets).x_axis(x_axis).y_axis(y_axis);

        frame.render_widget(chart, graph);
    }
}