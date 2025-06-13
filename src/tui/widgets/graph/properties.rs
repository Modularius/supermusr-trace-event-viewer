use ratatui::{text::Span, widgets::Axis};

use crate::graphics::{Bound, Bounds, Point};

const SHIFT_COEF: f64 = 0.1;

fn make_axis<'a>(bound: &Bound, title: &'static str, num_labels: i32) -> Axis<'static> {
    let labels: Vec<_> = (0..num_labels)
        .map(|i| bound.range() * i as f64 / num_labels as f64 + bound.min)
        .map(|v| Span::raw(format!("{v:.3}")))
        .collect();
    Axis::default()
        .title(title)
        .bounds([bound.min, bound.max])
        .labels(labels)
}

pub(crate) struct GraphProperties {
    pub(super) bounds: Bounds,
    pub(super) zoomed_bounds: Bounds,
    pub(super) view_port: Point,
    pub(super) zoom_factor: f64,
    pub(super) x_axis: Axis<'static>,
    pub(super) y_axis: Axis<'static>,
}

impl GraphProperties {
    pub(super) fn new(bounds: Bounds) -> Self {
        let zoomed_bounds = bounds.clone();
        let view_port = bounds.mid_point();

        let x_axis = make_axis(&bounds.time, "Time", 5);
        let y_axis = make_axis(&bounds.intensity, "Intensity", 5);
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
        self.zoomed_bounds = self.bounds.transform(self.zoom_factor, &self.view_port);
        //info!("{}, {}, {}", self.zoom_factor, self.view_port.time, self.view_port.intensity);

        self.x_axis = make_axis(&self.zoomed_bounds.time, "Time", 10);
        self.y_axis = make_axis(&self.zoomed_bounds.intensity, "Intensity", 5);
    }

    pub(crate) fn zoom_in(&mut self) {
        self.zoom_factor *= 1.1;
        if self.zoom_factor > 64.0 {
            self.zoom_factor = 64.0;
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
        self.view_port.time += time * SHIFT_COEF * self.zoomed_bounds.time.range();
        self.view_port.intensity += intensity * SHIFT_COEF * self.zoomed_bounds.intensity.range();
        self.calc_axes();
    }

    pub(crate) fn get_info(&self) -> String {
        format!("({}, {}): {:.2}", self.view_port.time, self.view_port.intensity, self.zoom_factor)
    }
}
