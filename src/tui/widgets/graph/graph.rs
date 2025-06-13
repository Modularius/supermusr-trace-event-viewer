use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols::Marker,
    text::Span,
    widgets::{Axis, Chart, Dataset, GraphType, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::{
    graphics::{Bound, Bounds, Point},
    messages::{EventList, Trace},
    tui::{ComponentStyle, GraphProperties, ParentalFocusComponent, TuiComponent, TuiComponentBuilder},
    Component,
};

pub(crate) struct Graph {
    parent_has_focus: bool,
    trace_data: Vec<(f64, f64)>,
    event_data: Option<Vec<(f64, f64)>>,
    properties: Option<GraphProperties>,
    hscroll_state: ScrollbarState,
    vscroll_state: ScrollbarState,
}

impl Graph {
    pub(crate) fn new() -> TuiComponent<Self> {
        TuiComponentBuilder::new(ComponentStyle::selectable())
            .is_in_block(true)
            .build(Self {
                trace_data: Default::default(),
                event_data: None,
                parent_has_focus: false,
                properties: None,
                hscroll_state: ScrollbarState::default(),
                vscroll_state: ScrollbarState::default(),
            })
    }

    pub(crate) fn set(&mut self, trace_data: &Trace, event_data: Option<&EventList>) {
        let trace_data: Vec<_> = (0_u32..).zip(trace_data.iter().copied()).collect();

        let event_data: Option<Vec<_>> =
            event_data.map(|events| events.iter().map(|e| (e.time, e.intensity)).collect());

        let time = trace_data.iter().map(|e| e.0 as u32);
        let time_bounds = Bound::from(1.0625, time.clone());

        let values = trace_data.iter().map(|e| e.1);
        let intensity_bounds = Bound::from(1.125, values.clone());

        let properties = GraphProperties::new(Bounds {
            time: time_bounds,
            intensity: intensity_bounds,
        });

        self.trace_data = trace_data
            .iter()
            .copied()
            .map(|(t, v)| (t as f64, v as f64))
            .filter(|&(time, intensity)| properties.zoomed_bounds.is_in(Point { time, intensity }))
            .collect();

        self.event_data = event_data.as_ref().map(|event_data| {
            event_data
                .iter()
                .copied()
                .map(|e| (e.0 as f64, e.1 as f64))
                .collect::<Vec<_>>()
        });

        self.properties = Some(properties);
        self.hscroll_state = ScrollbarState::new(100).viewport_content_length(100);
        self.vscroll_state = ScrollbarState::new(100).viewport_content_length(100);
    }

    pub(crate) fn get_properties_mut(&mut self) -> Option<&mut GraphProperties> {
        self.properties.as_mut()
    }
}

impl Component for Graph {
    fn render(&self, frame: &mut Frame, area: Rect) {
        if let Some(properties) = &self.properties {
            // Infobar/Graph division
            let (_info, graph) = {
                let chunk = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(area);
                (chunk[0], chunk[1])
            };

            // Graph/Hscroll division
            let (graph, hscroll) = {
                let chunk1 = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(2)])
                    .split(graph);

                let chunk2 = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(0), Constraint::Length(2)])
                    .split(chunk1[1]);
                (chunk1[0], chunk2[1])
            };

            //  Graph/Vscroll division
            let (graph, vscroll) = {
                let chunk = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(0), Constraint::Length(2)])
                    .split(graph);
                (chunk[0], chunk[1])
            };

            let horiz_scroll = Scrollbar::new(ScrollbarOrientation::HorizontalBottom);
            let vert_scroll = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            frame.render_stateful_widget(horiz_scroll, hscroll, &mut self.hscroll_state.clone());
            frame.render_stateful_widget(vert_scroll, vscroll, &mut self.vscroll_state.clone());

            let trace_dataset = Dataset::default()
                .name("Trace")
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::new().fg(Color::Blue).bg(Color::Black))
                .data(&self.trace_data);

            let event_dataset = self.event_data.as_ref().map(|event_data| {
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

            let chart = Chart::new(datasets)
                .x_axis(properties.x_axis.clone())
                .y_axis(properties.y_axis.clone());

            frame.render_widget(chart, graph);
        }
    }
}

impl ParentalFocusComponent for Graph {
    fn propagate_parental_focus(&mut self, focus: bool) {
        self.parent_has_focus = focus;
    }
}
