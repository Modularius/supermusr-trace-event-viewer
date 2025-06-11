use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols::Marker,
    text::Span,
    widgets::{Axis, Chart, Dataset, GraphType, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame
};

use crate::{messages::{EventList, Trace}, tui::{ComponentStyle, ParentalFocusComponent, TuiComponent, TuiComponentBuilder}, Component};

fn min_max<'a, D: Default + Ord + Into<f64>, I : Iterator<Item = D> + Clone>(buffer: f64, data: I) -> Bound {
    let min: f64 = data.clone().min().unwrap_or_default().into();
    let max: f64 = buffer*(data.max().unwrap_or_default().into());
    Bound { min, max }
}

const SHIFT_COEF : f64 = 0.1;

#[derive(Default, Clone)]
struct Pair<D: Default> {
    time: D,
    intensity: D,
}

#[derive(Default, Clone)]
struct Bound {
    min: f64,
    max: f64,
}

impl Bound {
    fn mid_point(&self) -> f64 {
        (self.max + self.min)/2.0
    }
    
    fn range(&self) -> f64 {
        self.max - self.min
    }

    fn transform(&self, zoom_factor: f64, delta: f64) -> Self {
        Self {
            min: (self.min - delta)/zoom_factor + delta,
            max: (self.max - delta)/zoom_factor + delta
        }
    }
    
    fn make_axis<'a>(&self, title: &'static str, num_labels: i32) -> Axis<'static> {
        let labels: Vec<_> = (0..num_labels)
            .map(|i|self.range()*i as f64/num_labels as f64 + self.min)
            .map(|v|Span::raw(v.to_string()))
            .collect();
        Axis::default()
            .title(title)
            .bounds([self.min, self.max])
            .labels(labels)
    }
}


type Bounds = Pair<Bound>;

impl Bounds {
    fn mid_point(&self) -> Point {
        Point {
            time: self.time.mid_point(),
            intensity: self.intensity.mid_point(),
        }
    }

    fn transform(&mut self, source: &Bounds, zoom_factor: f64, delta: &Point) -> Self {
        Self{
            time: source.time.transform(zoom_factor, delta.time),
            intensity: source.intensity.transform(zoom_factor, delta.intensity)
        }
    }

    fn is_in(&self, point: Point) -> bool {
        self.time.min <= point.time && point.time <= self.time.max && self.intensity.min <= point.intensity && point.intensity <= self.intensity.max
    }
}

type Point = Pair<f64>;

impl Into<(f64,f64)> for Point {
    fn into(self) -> (f64,f64) {
        (self.time, self.intensity)
    }
}

pub(crate) struct GraphProperties {
    bounds: Bounds,
    zoomed_bounds: Bounds,
    view_port: Point,
    zoom_factor: f64,
    x_axis: Axis<'static>,
    y_axis: Axis<'static>,
}

impl GraphProperties {
    fn new(bounds: Bounds) -> Self {
        let zoomed_bounds = bounds.clone();
        let view_port = bounds.mid_point();
        
        let x_axis = bounds.time.make_axis("Time", 5);
        let y_axis = bounds.intensity.make_axis("Intensity", 5);
        Self {
            bounds,
            zoomed_bounds,
            view_port,
            zoom_factor: 1.0,
            x_axis,
            y_axis,
        }
    }

    fn calc_axes(&mut self) {
        self.zoomed_bounds.transform(&self.bounds, self.zoom_factor, &self.view_port);

        self.x_axis = self.zoomed_bounds.time.make_axis("Time", 5);
        self.y_axis = self.zoomed_bounds.intensity.make_axis("Intensity", 5);
    }

    pub(crate) fn zoom_in(&mut self) {
        self.zoom_factor *= 1.1;
        if self.zoom_factor > 4.0 {
            self.zoom_factor = 4.0;
        }
        self.calc_axes();
    }

    pub(crate) fn zoom_out(&mut self) {
        self.zoom_factor /= 1.1;
        if self.zoom_factor < 1.0 {
            self.zoom_factor = 1.0;
        }
        self.calc_axes();
    }

    pub(crate) fn move_viewport(&mut self, time: f64, intensity: f64) {
        self.view_port.time += time*SHIFT_COEF*self.zoomed_bounds.time.range();
        self.view_port.intensity += intensity*SHIFT_COEF*self.zoomed_bounds.intensity.range();
/*
        // If minimum time bound goes too far left, fix it
        if (self.time_bounds.0 - self.view_port.0)*self.zoom_factor < self.time_bounds.0 {
            self.view_port.0 = self.time_bounds.0 - self.time_bounds.0/self.zoom_factor;
        }
        
        // If maximum time bound goes too far right, fix it
        if (self.time_bounds.1 - self.view_port.0)*self.zoom_factor > self.time_bounds.1 {
            self.view_port.0 = self.time_bounds.1 - self.time_bounds.1/self.zoom_factor;
        }
        
        // If minimum intensity bound goes too far down, fix it
        if (self.intensity_bounds.0 - self.view_port.1)*self.zoom_factor < self.intensity_bounds.0 {
            self.view_port.1 = self.intensity_bounds.0 - self.intensity_bounds.0/self.zoom_factor;
        }
        
        // If maximum intensity bound goes too far up, fix it
        if (self.intensity_bounds.1 - self.view_port.1)*self.zoom_factor > self.intensity_bounds.1 {
            self.view_port.1 = self.intensity_bounds.1 - self.intensity_bounds.1/self.zoom_factor;
        } */
        self.calc_axes();
    }
}

pub(crate) struct Graph {
    parent_has_focus: bool,
    trace_data: Vec<(f64,f64)>,
    event_data: Option<Vec<(f64,f64)>>,
    properties: Option<GraphProperties>,
    hscroll_state: ScrollbarState,
    vscroll_state: ScrollbarState,
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
                hscroll_state: ScrollbarState::default(),
                vscroll_state: ScrollbarState::default(),
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
        let time_bounds = min_max(1.0625, time.clone());
        
        let values = trace_data.iter().map(|e|e.1);
        let intensity_bounds = min_max(1.125, values.clone());
        
        let properties = GraphProperties::new(Bounds { time: time_bounds, intensity: intensity_bounds});

        self.trace_data = trace_data.iter()
            .copied()
            .map(|(t,v)|(t as f64, v as f64))
            .filter(|&(time, intensity)|properties.zoomed_bounds.is_in(Point{time, intensity}))
            .collect();

        self.event_data = event_data.as_ref()
            .map(|event_data| {
                event_data.iter()
                    .copied()
                    .map(|e|(e.0 as f64, e.1 as f64))
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
            let (_info, graph) = {
                let chunk = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(area);
                (chunk[0], chunk[1])
            };
            
            let (graph, hscroll) = {
                let chunk1 = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(1)])
                    .split(graph);
                
                let chunk2 = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(0), Constraint::Length(1)])
                    .split(chunk1[1]);
                (chunk1[0], chunk2[1])
            };
            
            let (graph, vscroll) = {
                let chunk = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(0), Constraint::Length(1)])
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