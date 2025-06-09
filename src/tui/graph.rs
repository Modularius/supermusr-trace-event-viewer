use std::{io::Stdout, str::FromStr};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect, prelude::CrosstermBackend, style::{Color, Style}, symbols::Marker, text::Span, widgets::{Axis, Chart, Dataset, GraphType, List, ListItem, ListState}, Frame
};
use supermusr_common::Intensity;

use crate::{tui::{ComponentStyle, FocusableComponent, TuiComponent, TuiComponentBuilder}, Component};

type D = (usize,Intensity);

pub(crate) struct Graph {
    has_focus: bool,
    parent_has_focus: bool,
    data: Vec<D>
}

impl Graph where {
    pub(crate) fn new(data: &[D], name: Option<&'static str>) -> TuiComponent<Self> {
        let builder = TuiComponentBuilder::new(ComponentStyle::selectable())
            .is_in_block(true);

        if let Some(name) = name {
            builder.with_name(name)
        } else {
            builder
        }.build(Self {
            data: data.to_vec(),
            has_focus: false,
            parent_has_focus: false,
        })
    }

    pub(crate) fn set(&mut self, data: Vec<D>) {
        self.data = data;
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

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
            let data_iter = self.data.iter().copied();
            let data = data_iter.clone().map(|(t,v)|(t as f64, v as f64)).collect::<Vec<_>>();

            let dataset = Dataset::default()
                .name("Trace")
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::new().fg(Color::Blue).bg(Color::Black))
                .data(&data);

            let time = data_iter.clone().map(|(t,_)|t);
            let min = time.clone().min().unwrap_or_default() as f64;
            let max = time.max().unwrap_or_default() as f64;
            let num_labels = 5;
            let labels = (0..num_labels).map(|i|(max - min)*i as f64/num_labels as f64 + min).map(|v|Span::raw(v.to_string())).collect::<Vec<_>>();
            let x_axis = Axis::default()
                .title("Time")
                .bounds([min, max])
                .labels(labels);

            let values = data_iter.map(|(_,x)|x);
            let min = values.clone().min().unwrap_or_default() as f64;
            let max = values.max().unwrap_or_default() as f64;
            let num_labels = 10;
            let labels = (0..num_labels).map(|i|(max - min)*i as f64/num_labels as f64 + min).map(|v|Span::raw(v.to_string())).collect::<Vec<_>>();

            let y_axis = Axis::default()
                .title("Intensity")
                .bounds([min, max])
                .labels(labels);

            let chart = Chart::new(vec![dataset]).x_axis(x_axis).y_axis(y_axis);
            frame.render_widget(chart, area);
    }
}