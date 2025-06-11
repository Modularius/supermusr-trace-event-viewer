use std::iter::Empty;

use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols::Marker,
    text::Span,
    widgets::{Axis, Chart, Dataset, GraphType},
    Frame
};
use supermusr_common::Time;

use crate::{messages::{Event, EventList, Trace}, tui::{ComponentStyle, FocusableComponent, ParentalFocusComponent, TuiComponent, TuiComponentBuilder}, Component};

pub(crate) struct GraphProperties {
    time_bounds: (f64, f64),
    intensity_bounds: (f64, f64),
    x_axis: Axis<'static>,
    y_axis: Axis<'static>,
}

pub(crate) struct Graph {
    parent_has_focus: bool,
    trace_data: Vec<(f64,f64)>,
    event_data: Option<Vec<(f64,f64)>>,
    properties: Option<GraphProperties>,
    //zoom: f64,
}

impl Graph where {
    pub(crate) fn new() -> TuiComponent<Self> {
        TuiComponentBuilder::new(ComponentStyle::selectable())
            .is_in_block(true)
            .build(Self {
                trace_data: Default::default(),
                event_data: None,
                parent_has_focus: false,
                properties: None,
            }
        )
    }

    pub(crate) fn set(&mut self, trace_data: &Trace, event_data: Option<&EventList>) {
        let trace_data : Vec<_> = (0_u32..).zip(
            trace_data
                .iter()
                .copied()
            )
            .collect();
        let event_data : Option<Vec<_>> = event_data
            .map(|events|events.iter().map(|e|(e.time,e.intensity)).collect());

        let time = trace_data.iter().map(|e|e.0 as u32);
        let time_bounds = Self::min_max(1.0625, time.clone());

        let x_axis = Self::make_axis("Time", 5, &time_bounds);
        
        let values = trace_data.iter().map(|e|e.1);
        let intensity_bounds = Self::min_max(1.125, values.clone());
        let y_axis = Self::make_axis("Intensity", 5, &intensity_bounds);
        
        self.properties = Some(GraphProperties {
            time_bounds,
            intensity_bounds,
            x_axis,
            y_axis
        });

        self.trace_data = trace_data.iter()
            .copied()
            .map(|(t,v)|(t as f64, v as f64))
            .collect();

        self.event_data = event_data.as_ref()
            .map(|event_data| {
                event_data.iter()
                    .copied()
                    .map(|e|(e.0 as f64, e.1 as f64))
                    .collect::<Vec<_>>()
            });
    }

    fn min_max<'a, D: Default + Ord + Into<f64>, I : Iterator<Item = D> + Clone>(buffer: f64, data: I) -> (f64,f64) {
        let min: f64 = data.clone().min().unwrap_or_default().into();
        let max: f64 = buffer*(data.max().unwrap_or_default().into());
        (min, max)
    }

    fn make_axis<'a>(title: &'static str, num_labels: i32, &(min, max): &(f64,f64)) -> Axis<'static> {
        let labels = (0..num_labels).map(|i|(max - min)*i as f64/num_labels as f64 + min).map(|v|Span::raw(v.to_string())).collect::<Vec<_>>();
        Axis::default()
            .title(title)
            .bounds([min, max])
            .labels(labels)
    }
}

impl Component for Graph {
    fn render(&self, frame: &mut Frame, area: Rect) {
        if let Some(properties) = &self.properties {
            let (_info, graph) = {
                let chunk = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(area);
                (chunk[0], chunk[1])
            };

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

            /*let zoom_box = [
                    (properties.time_bounds.0 + 1000.0, properties.intensity_bounds.0 + 10.0),
                    (properties.time_bounds.1 - 1000.0, properties.intensity_bounds.0 + 10.0),
                    (properties.time_bounds.1 - 1000.0, properties.intensity_bounds.1 - 10.0),
                    (properties.time_bounds.0 + 1000.0, properties.intensity_bounds.1 - 10.0),
                    (properties.time_bounds.0 + 1000.0, properties.intensity_bounds.0 + 10.0),
                ];
            let zoom_dataset = Dataset::default()
                .graph_type(GraphType::Line)
                .style(Style::new().fg(Color::White))
                .data(&zoom_box);
            */
            let datasets = if let Some(event_dataset) = event_dataset {
                vec![trace_dataset, event_dataset]
            } else {
                vec![trace_dataset]
            };
            //, zoom_dataset
            
            let chart = Chart::new(datasets).x_axis(properties.x_axis.clone()).y_axis(properties.y_axis.clone());

            frame.render_widget(chart, graph);
        }
    }
}

impl ParentalFocusComponent for Graph {
    fn propagate_parental_focus(&mut self, focus: bool) {
        self.parent_has_focus = focus;
    }
}