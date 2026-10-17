//! Props to `charts-rs` and back: it draws the static SVG, we harvest where each mark
//! landed and strip its own tooltips, so the overlay and keyboard layers can be ours.

use charts_rs::{
    AxisLabelOverflow, AxisType, Color as InkColor, DEFAULT_FONT_FAMILY, LineStep,
    MarkLineCategory, Position, SeriesCategory, Symbol,
};

use super::data::{Trend, ValueFormat, ungrouped};
use crate::{
    localization::Formats,
    theme::{BarLayout, ChartInk, Curve, HexColor, PieLabels, PointSymbol},
};

mod axis;
mod bar;
mod candle;
mod funnel;
mod heatmap;
mod labels;
mod line;
mod marks;
mod pie;
mod radar;
mod scatter;
mod svg;
#[cfg(test)]
mod tests;

use axis::*;
pub(crate) use bar::*;
pub(crate) use candle::*;
pub(crate) use funnel::*;
pub(crate) use heatmap::*;
use labels::*;
pub(crate) use line::*;
use marks::*;
pub(crate) use pie::*;
pub(crate) use radar::*;
pub(crate) use scatter::*;
pub(crate) use svg::*;

/// Ahead of each y-axis label, so [`sanitize`] can write it again in the locale.
const AXIS_MARK: char = '\u{2060}';
/// The same for the labels of a numeric x axis.
const X_MARK: char = '\u{2061}';
/// Ahead of each percentage the renderer writes, to write it again in the locale.
const PERCENT_MARK: char = '\u{2062}';
/// Ahead of each label on a bar: written as the y labels, but not pinned to the axis.
const LABEL_MARK: char = '\u{2063}';
/// Before every right y axis label, written again with that axis' unit.
const AXIS2_MARK: char = '\u{2064}';
const DASH: &str = "6,4";
/// An axis' tick is 5 long; an axis or grid line is longer.
const PLOT_TICK: f32 = 10.0;
/// The pie's radius over half the plot's shorter side, as the renderer caps it.
const PIE_FILL: f32 = 0.8;
/// The least of its full radius a pie keeps to make room for its labels.
const PIE_MIN: f32 = 0.4;
/// How much wider than Roboto the x labels are fitted for, as the page's font may be.
const PAGE_FONT_SLACK: f32 = 1.15;
/// Room between a label and the SVG's edge.
const EDGE: f32 = 4.0;
/// A label on a bar is drawn in this, then given the ink that reads on its bar.
const ON_BAR: InkColor = InkColor {
    r: 1,
    g: 2,
    b: 3,
    a: 255,
};
const ON_BAR_FILL: &str = "fill=\"#010203\"";
/// The renderer strokes its error bars in the labels' colour, so in [`ON_BAR`] too.
const ON_BAR_STROKE: &str = "stroke=\"#010203\"";
/// The renderer's room at each side of a band and between two bars of a group, in px.
const RENDERER_MARGIN: f32 = 5.0;
const RENDERER_GAP: f32 = 3.0;

/// Where every mark of a drawn chart sits, in the SVG's own units.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Geometry {
    pub(crate) width: f32,
    pub(crate) height: f32,
    /// The area inside the axes; a pie's is the square its `center` fills.
    pub(crate) plot: Plot,
    /// Series by series, in data order; a gap has no mark.
    pub(crate) marks: Vec<Mark>,
    /// A donut's hole radius, round the SVG's middle; 0 without one.
    pub(crate) hole: f32,
    /// A funnel's stages are shares of its first, a pie's slices of their sum: `None`.
    pub(crate) share_of: Option<f64>,
    /// A heatmap's marks are cells: each tipped alone, not with its column.
    pub(crate) cells: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Plot {
    pub(crate) left: f32,
    pub(crate) top: f32,
    pub(crate) right: f32,
    pub(crate) bottom: f32,
}

impl Plot {
    pub(crate) fn whole(width: f32, height: f32) -> Self {
        Self {
            left: 0.0,
            top: 0.0,
            right: width,
            bottom: height,
        }
    }

    /// The box the axes and grid lines span; ticks are too short to count.
    fn of(svg: &str, width: f32, height: f32) -> Self {
        let mut found: Option<Self> = None;
        let mut rest = svg;
        while let Some(open) = rest.find("<line ") {
            let close = rest[open..]
                .find('>')
                .map_or(rest.len(), |close| open + close + 1);
            let attributes = attributes(&rest[open..close]);
            rest = &rest[close..];
            let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
            let (Some(x1), Some(y1), Some(x2), Some(y2)) =
                (number("x1"), number("y1"), number("x2"), number("y2"))
            else {
                continue;
            };
            if (x2 - x1).hypot(y2 - y1) <= PLOT_TICK {
                continue;
            }
            let line = Self {
                left: x1.min(x2),
                top: y1.min(y2),
                right: x1.max(x2),
                bottom: y1.max(y2),
            };
            found = Some(match found {
                Some(plot) => Self {
                    left: plot.left.min(line.left),
                    top: plot.top.min(line.top),
                    right: plot.right.max(line.right),
                    bottom: plot.bottom.max(line.bottom),
                },
                None => line,
            });
        }
        found
            .filter(|plot| plot.right > plot.left && plot.bottom > plot.top)
            .unwrap_or(Self::whole(width, height))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Mark {
    pub(crate) series: usize,
    pub(crate) index: usize,
    pub(crate) value: f64,
    pub(crate) shape: MarkShape,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MarkShape {
    /// A line's point.
    Point { x: f32, y: f32 },
    /// A bar.
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    /// A pie slice, as an SVG path.
    Sector { path: String },
}

/// A drawn chart: the SVG to show and where its marks are.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Drawn {
    pub(crate) svg: String,
    pub(crate) geometry: Geometry,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SeriesSpec {
    pub(crate) name: String,
    pub(crate) values: Vec<Option<f64>>,
    pub(crate) color: HexColor,
    pub(crate) dashed: bool,
    pub(crate) extras: SeriesExtras,
}

/// What a series asks beyond its values and look; a bar or a slice draws none of it.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SeriesExtras {
    pub(crate) symbol: PointSymbol,
    /// Each mark line's value and the text written by it.
    pub(crate) mark_lines: Vec<(f64, String)>,
    /// Each marked point's x index and the text pinned by it.
    pub(crate) mark_points: Vec<(usize, String)>,
    /// Measured on the right y axis.
    pub(crate) secondary: bool,
    /// A line over a bar chart's bars.
    pub(crate) line: bool,
}

/// What a line or bar chart's axes are given beyond its data.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Axes {
    /// The value axis' ends; unset, from the data and 0.
    pub(crate) min: Option<f64>,
    pub(crate) max: Option<f64>,
    /// The steps the value axis is cut into.
    pub(crate) ticks: usize,
    pub(crate) x_title: Option<String>,
    pub(crate) y_title: Option<String>,
    /// A line chart's right y axis, read downward.
    pub(crate) y2_title: Option<String>,
    /// The locale's million, billion and trillion suffixes for a long axis' ticks.
    pub(crate) compact: [&'static str; 3],
}

impl Axes {
    /// `lying`: the renderer's first y config is a horizontal bar chart's value axis, at the
    /// bottom, and its x title the categories' beside them.
    fn style(
        &self,
        chart: &mut charts_rs::ChartBase,
        y: &mut [charts_rs::YAxisConfig],
        lying: bool,
    ) {
        let (bottom, side) = match lying {
            true => (&self.y_title, &self.x_title),
            false => (&self.x_title, &self.y_title),
        };
        chart.x_axis.title = bottom.clone().unwrap_or_default();
        if let Some(first) = y.first_mut() {
            first.min = self.min.map(|min| min as f32);
            first.max = self.max.map(|max| max as f32);
            first.split_number = self.ticks.max(1);
            first.title = side.clone();
        }
        // Cut like the left one, so its labels sit on the grid's lines.
        if let Some(second) = y.get_mut(1) {
            second.split_number = self.ticks.max(1);
            second.title = self.y2_title.clone();
        }
    }
}

fn ink(hex: HexColor) -> InkColor {
    InkColor::from((hex.r(), hex.g(), hex.b()))
}

fn unit_suffix(unit: Option<&str>) -> String {
    unit.map(|unit| format!(" {unit}")).unwrap_or_default()
}

/// `(series, index, value, name)` of every mark the renderer draws, in its order.
fn expected_marks(series: &[SeriesSpec]) -> Vec<(usize, usize, f64, &str)> {
    series
        .iter()
        .enumerate()
        .flat_map(|(s, series)| {
            series
                .values
                .iter()
                .enumerate()
                .filter_map(move |(i, value)| {
                    value.map(|value| (s, i, value, series.name.as_str()))
                })
        })
        .collect()
}

/// Pairs the renderer's triggers with the marks we expect; on any mismatch, no marks
/// rather than wrong ones.
fn match_marks(expected: &[(usize, usize, f64, &str)], triggers: &[Trigger]) -> Vec<Mark> {
    let matches = expected.len() == triggers.len()
        && expected
            .iter()
            .zip(triggers)
            .all(|((.., name), trigger)| trigger.series == *name);
    if !matches {
        return Vec::new();
    }
    expected
        .iter()
        .zip(triggers)
        .map(|(&(series, index, value, _), trigger)| Mark {
            series,
            index,
            value,
            shape: trigger.shape.clone(),
        })
        .collect()
}
