mod bounds;
mod svg;

use std::{
    fs::create_dir_all,
    marker::PhantomData,
    ops::Range,
    path::{Path, PathBuf},
};

use anyhow::{bail, Error};
use plotters::{
    chart::{ChartBuilder, ChartContext},
    coord::{types::RangedCoordf32, Shift},
    prelude::{
        BitMapBackend, Cartesian2d, Circle, DrawingArea, DrawingBackend, IntoDrawingArea,
        PathElement, SVGBackend,
    },
    series::{LineSeries, PointSeries},
    style::{IntoFont, RGBColor, ShapeStyle, BLACK, BLUE, WHITE},
};
use strum::{Display, EnumIter, EnumString};
use supermusr_common::Channel;
use tracing::{info, instrument};

use crate::messages::DigitiserTrace;

pub(crate) use bounds::{Bound, Bounds, Point};
pub(crate) use svg::SvgSaver;

#[derive(Clone, EnumString, Display, EnumIter)]
pub(crate) enum FileFormat {
    Svg,
}

pub(crate) trait Backend<'b> {
    const EXTENSION: &'static str;
    type Backend: DrawingBackend;

    fn new<T>(path: &'b T, size: (u32, u32)) -> Self::Backend
    where
        T: AsRef<Path> + ?Sized;
}

pub(crate) struct BackendSVG<'a> {
    phantom: PhantomData<&'a ()>,
}

impl<'a, 'b: 'a> Backend<'b> for BackendSVG<'a> {
    const EXTENSION: &'static str = "svg";
    type Backend = SVGBackend<'a>;

    fn new<T>(path: &'b T, size: (u32, u32)) -> Self::Backend
    where
        T: AsRef<Path> + ?Sized,
    {
        SVGBackend::new(path, size)
    }
}

pub(crate) trait GraphSaver: Default {
    fn save_as_svg(trace: &DigitiserTrace, channels: Vec<Channel>, path: PathBuf);
}
