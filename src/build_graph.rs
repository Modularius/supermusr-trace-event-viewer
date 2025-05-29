use std::{ops::Range, path::{Path, PathBuf}};

use plotters::{chart::{ChartBuilder, ChartContext}, coord::{types::RangedCoordf32, Shift}, prelude::{BitMapBackend, Cartesian2d, DrawingArea, DrawingAreaErrorKind, IntoDrawingArea, PathElement}, series::LineSeries, style::{IntoFont, RGBColor, BLUE}};
use crate::data::Trace;

pub(crate) struct BuildGraph {
    width: usize,
    height: usize,
    x_range: Range<f32>,
    y_range: Range<f32>,
}

impl BuildGraph {
    pub(crate) fn new(width: usize, height: usize, x_range: Range<f32>, y_range: Range<f32>) -> Self {
        Self {
            width,
            height,
            x_range,
            y_range,
        }
    }

    pub(crate) fn save_trace_graph(&self, path: &Path, trace: &Trace) -> Result<(), anyhow::Error> {
        let root = BitMapBackend::new(path, (self.width as u32, self.height as u32)).into_drawing_area();
        self.build_trace_graph(&root, trace)?;
        root.present()?;
        Ok(())
    }
    pub(crate) fn build_trace_graph(&self, root: &DrawingArea<BitMapBackend, Shift>, trace: &Trace) -> Result<(), anyhow::Error> {
        let mut chart = ChartBuilder::on(root)
            .x_label_area_size(35)
            .y_label_area_size(40)
            //.right_y_label_area_size(40)
            .margin(5)
            .caption("Trace", ("sans-serif", 50.0).into_font())
            .build_cartesian_2d(0f32..10f32, 0.1f32..1e10f32)?;
            //.set_secondary_coord(0f32..10f32, -1.0f32..1.0f32);

            
        chart
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            //.y_desc("Log Scale")
            .y_label_formatter(&|x| format!("{:e}", x))
            .draw()?;
        
        self.draw_trace_to_chart(&mut chart, trace)?;

        chart
            .configure_series_labels()
            .background_style(RGBColor(128, 128, 128))
            .draw()?;
        Ok(())
    }

    fn draw_trace_to_chart(&self, chart: &mut ChartContext<BitMapBackend, Cartesian2d<RangedCoordf32, RangedCoordf32>>, trace: &Trace) -> Result<(), anyhow::Error> {
        let data = trace.iter().cloned().enumerate().map(|(x,y)|(x as f32,y as f32));
        chart
            .draw_series(LineSeries::new(
                data,
                &BLUE,
            ))?
            .label("y = Trace")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));
        Ok(())
    }

    pub(crate) fn build_event_graph(&self, root: &DrawingArea<BitMapBackend, Shift>, trace: Trace) -> Result<(), anyhow::Error> {
        
        Ok(())
    }
}