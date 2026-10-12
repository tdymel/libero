use crate::{
    localization::Formats,
    theme::{Color, ColorShade, HexColor, PointSymbol},
    utils::warn,
};

/// A chart's x axis, one entry per point of every series.
///
/// ```rust
/// # use libero::components::XValues;
/// let months = XValues::from(["Jan", "Feb", "Mar"]);
/// let years = XValues::from(vec![2024.0, 2025.0, 2026.0]);
/// # let _ = (months, years);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum XValues {
    /// Labels, one evenly spaced slot each, whatever they say.
    Categories(Vec<String>),
    /// Numbers on a value axis: each point sits at its number, so `1, 2, 10` leaves a gap
    /// of eight steps. Written with the chart's `value_format`; keep them finite.
    Numbers(Vec<f64>),
}

impl XValues {
    /// One x value per row: `XValues::from_rows(&sales, |row| row.month.clone())`.
    pub fn from_rows<T, X: Into<String>>(rows: &[T], x: impl Fn(&T) -> X) -> Self {
        Self::Categories(rows.iter().map(|row| x(row).into()).collect())
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Categories(values) => values.len(),
            Self::Numbers(values) => values.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The numbers of a value axis; none for categories.
    pub(crate) fn numbers(&self) -> &[f64] {
        match self {
            Self::Categories(_) => &[],
            Self::Numbers(values) => values,
        }
    }

    /// Every x value as the chart writes it; numbers ungrouped, as years are written.
    pub(crate) fn labels(&self, format: &ValueFormat, formats: &Formats) -> Vec<String> {
        let ungrouped = ungrouped(formats);
        match self {
            Self::Categories(values) => values.clone(),
            Self::Numbers(values) => values
                .iter()
                .map(|value| format.format(*value, None, &ungrouped))
                .collect(),
        }
    }
}

impl Default for XValues {
    fn default() -> Self {
        Self::Categories(Vec::new())
    }
}

impl<const N: usize> From<[&str; N]> for XValues {
    fn from(values: [&str; N]) -> Self {
        Self::Categories(values.iter().map(|value| value.to_string()).collect())
    }
}

impl From<Vec<&str>> for XValues {
    fn from(values: Vec<&str>) -> Self {
        Self::Categories(values.into_iter().map(str::to_string).collect())
    }
}

impl From<Vec<String>> for XValues {
    fn from(values: Vec<String>) -> Self {
        Self::Categories(values)
    }
}

impl From<Vec<f64>> for XValues {
    fn from(values: Vec<f64>) -> Self {
        Self::Numbers(values)
    }
}

impl From<Vec<i32>> for XValues {
    fn from(values: Vec<i32>) -> Self {
        Self::Numbers(values.into_iter().map(f64::from).collect())
    }
}

/// One value of a series; `None`, or a value that is not finite, is a gap in the line.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Value(pub Option<f64>);

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self(value.is_finite().then_some(value))
    }
}

impl From<Option<f64>> for Value {
    fn from(value: Option<f64>) -> Self {
        Self(value.filter(|value| value.is_finite()))
    }
}

macro_rules! value_from {
    ($($ty:ty),+) => {$(
        impl From<$ty> for Value {
            #[allow(clippy::cast_lossless, reason = "one macro for the exact and the rounding casts")]
            fn from(value: $ty) -> Self {
                Self::from(value as f64)
            }
        }

        impl From<Option<$ty>> for Value {
            fn from(value: Option<$ty>) -> Self {
                Self(value.and_then(|value| Self::from(value).0))
            }
        }
    )+};
}

value_from!(f32, i32, u32, i64, u64, usize);

/// A series' colour: a theme colour at a shade, or a fixed hex.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChartColor {
    Palette(Color, ColorShade),
    Hex(HexColor),
}

impl From<Color> for ChartColor {
    fn from(color: Color) -> Self {
        Self::Palette(color, ColorShade::S6)
    }
}

impl From<HexColor> for ChartColor {
    fn from(hex: HexColor) -> Self {
        Self::Hex(hex)
    }
}

/// `"primary"`, `"success.7"` or `"#1e88e5"`; anything else warns and draws in `neutral.6`.
impl From<&str> for ChartColor {
    fn from(value: &str) -> Self {
        if let Some(hex) = HexColor::parse(value) {
            return Self::Hex(hex);
        }
        let (name, shade) = match value.split_once('.') {
            Some((name, shade)) => (name, Some(shade)),
            None => (value, None),
        };
        match Color::parse(name) {
            Some(color) => Self::Palette(color, ColorShade::parse(shade)),
            None => {
                warn(&format!(
                    "chart colour `{value}` is no theme colour or hex, drawing it in neutral.6"
                ));
                Self::Palette(Color::Neutral, ColorShade::S6)
            }
        }
    }
}

/// One line of a chart: a name and one value per x value.
///
/// ```rust
/// # use libero::components::Series;
/// let revenue = Series::new("Revenue", [120.0, 132.0, 101.0]).color("success");
/// let costs = Series::new("Costs", [Some(80), None, Some(95)]).dashed();
/// let share = Series::new("Share", [0.4, 0.5, 0.45]).unit("%");
/// let profit = Series::new("Profit", [40, -15, 20])
///     .negative_color("error")
///     .errors([(35, 45), (-20, -10), (15, 25)]);
/// # let _ = (revenue, costs, share, profit);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Series {
    pub name: String,
    pub values: Vec<Option<f64>>,
    /// Unset, the theme's palette colour for the series' place.
    pub color: Option<ChartColor>,
    /// A dashed line: a second cue beside the colour.
    pub dashed: bool,
    /// Written after each value in the tooltip and summary. Unset, the chart's `unit`.
    pub unit: Option<String>,
    /// The shape of a line's points and of its legend key. Unset, a circle.
    pub symbol: Option<PointSymbol>,
    /// A bar's own colour, one per x value; `None` or past the end, the series' colour.
    pub colors: Vec<Option<ChartColor>>,
    /// The colour of a bar below 0. Unset, the series' colour.
    pub negative_color: Option<ChartColor>,
    /// Each value's error range, `(low, high)` per x value: a bar's error bar, a band
    /// around a line. `None`, none there.
    pub errors: Vec<Option<(f64, f64)>>,
    /// Lines across the plot at a value of the series, in its colour.
    pub mark_lines: Vec<MarkLine>,
    /// A line's lowest or highest point, its value pinned by it.
    pub mark_points: Vec<MarkPoint>,
    /// Measured on the second y axis, at the right.
    pub secondary_axis: bool,
    /// How the series is drawn. Unset, the chart's own kind.
    pub kind: Option<SeriesKind>,
}

/// What a series is drawn as, where a chart can draw more than its own kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeriesKind {
    Bar,
    /// A line over the bars of an upright [`BarChart`](super::BarChart), through each
    /// category's middle.
    Line,
}

impl Series {
    pub fn new<V: Into<Value>>(
        name: impl Into<String>,
        values: impl IntoIterator<Item = V>,
    ) -> Self {
        Self {
            name: name.into(),
            values: values.into_iter().map(|value| value.into().0).collect(),
            color: None,
            dashed: false,
            unit: None,
            symbol: None,
            colors: Vec::new(),
            negative_color: None,
            errors: Vec::new(),
            mark_lines: Vec::new(),
            mark_points: Vec::new(),
            secondary_axis: false,
            kind: None,
        }
    }

    /// One value per row: `Series::from_rows("Revenue", &sales, |row| row.revenue)`.
    pub fn from_rows<T, V: Into<Value>>(
        name: impl Into<String>,
        rows: &[T],
        value: impl Fn(&T) -> V,
    ) -> Self {
        Self::new(name, rows.iter().map(value))
    }

    pub fn color(mut self, color: impl Into<ChartColor>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn dashed(mut self) -> Self {
        self.dashed = true;
        self
    }

    /// The series' own unit, where series of one chart measure different things.
    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// A line's points as squares, triangles or diamonds: told apart without the colour.
    pub fn symbol(mut self, symbol: PointSymbol) -> Self {
        self.symbol = Some(symbol);
        self
    }

    /// A colour per bar, in x order: `.colors(["success", "warning", "error"])`.
    pub fn colors<C: Into<ChartColor>>(mut self, colors: impl IntoIterator<Item = C>) -> Self {
        self.colors = colors.into_iter().map(|color| Some(color.into())).collect();
        self
    }

    /// The bars below 0 in `color`: `.negative_color("error")`.
    pub fn negative_color(mut self, color: impl Into<ChartColor>) -> Self {
        self.negative_color = Some(color.into());
        self
    }

    /// An error range per value, `(low, high)` in x order, drawn over a bar or as a band
    /// around a line; a missing or not finite end leaves that value without one:
    /// `.errors([(110, 130), (125, 140)])`.
    pub fn errors<V: Into<Value>>(mut self, errors: impl IntoIterator<Item = (V, V)>) -> Self {
        self.errors = errors
            .into_iter()
            .map(|(low, high)| {
                let (low, high) = (low.into().0?, high.into().0?);
                Some((low.min(high), low.max(high)))
            })
            .collect();
        self
    }

    /// A dashed line across the plot at the series' average, lowest, highest or a fixed
    /// value, labelled: `Series::new("Sales", [3, 5]).mark_line(MarkLine::average())`.
    pub fn mark_line(mut self, line: MarkLine) -> Self {
        self.mark_lines.push(line);
        self
    }

    /// A line's lowest or highest point with its value in a pin in the series' colour:
    /// `.mark_point(MarkPoint::Max)`. The first of equal values is marked.
    pub fn mark_point(mut self, point: MarkPoint) -> Self {
        if !self.mark_points.contains(&point) {
            self.mark_points.push(point);
        }
        self
    }

    /// Measured on a y axis at the right, by its own scale and unit: for a series of
    /// another size or kind, `Series::new("Rain", [12, 30]).unit("mm").secondary_axis()`.
    /// A line chart's, or an upright bar chart's: a line always, a bar unless the chart is
    /// `percent`, where the bars share one scale.
    pub fn secondary_axis(mut self) -> Self {
        self.secondary_axis = true;
        self
    }

    /// Drawn as another kind than the chart's: a line over the bars of an upright bar
    /// chart, `Series::new("Target", [130, 140]).kind(SeriesKind::Line)`. A line chart
    /// draws every series as a line.
    pub fn kind(mut self, kind: SeriesKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Drawn as a line in a bar chart.
    pub(crate) fn is_line(&self) -> bool {
        self.kind == Some(SeriesKind::Line)
    }
}

/// Which of a line's points a [`Series::mark_point`] pins its value by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkPoint {
    Min,
    Max,
}

/// Where a series' [`MarkLine`] sits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum MarkLineAt {
    Average,
    Min,
    Max,
    Value(f64),
}

/// A labelled line across a line or vertical bar chart at a value of its series.
///
/// ```rust
/// # use libero::components::{MarkLine, Series};
/// let sales = Series::new("Sales", [120.0, 132.0, 101.0])
///     .mark_line(MarkLine::average())
///     .mark_line(MarkLine::value(150.0).label("Target"));
/// # let _ = sales;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct MarkLine {
    pub(crate) at: MarkLineAt,
    /// Written by the line and in the summary. Unset, the localized "Average", "Lowest",
    /// "Highest", or "Mark" for a fixed value.
    pub(crate) label: Option<String>,
}

impl MarkLine {
    pub fn average() -> Self {
        Self::at(MarkLineAt::Average)
    }

    pub fn min() -> Self {
        Self::at(MarkLineAt::Min)
    }

    pub fn max() -> Self {
        Self::at(MarkLineAt::Max)
    }

    /// A fixed value, as a target; one outside the y axis is not drawn.
    pub fn value(value: f64) -> Self {
        Self::at(MarkLineAt::Value(value))
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    fn at(at: MarkLineAt) -> Self {
        Self { at, label: None }
    }

    /// Its value over the series' `values`; `None` with no value to take it from.
    pub(crate) fn resolve(&self, values: &[Option<f64>]) -> Option<f64> {
        let known = values.iter().flatten().copied().filter(|v| v.is_finite());
        match self.at {
            MarkLineAt::Value(value) => value.is_finite().then_some(value),
            MarkLineAt::Average => {
                let (sum, count) = known.fold((0.0, 0usize), |(s, c), v| (s + v, c + 1));
                (count > 0).then(|| sum / count as f64)
            }
            MarkLineAt::Min => known.reduce(f64::min),
            MarkLineAt::Max => known.reduce(f64::max),
        }
    }
}

/// One slice of a pie: a name and its share.
#[derive(Clone, Debug, PartialEq)]
pub struct Slice {
    pub name: String,
    pub value: f64,
    pub color: Option<ChartColor>,
}

impl Slice {
    pub fn new(name: impl Into<String>, value: f64) -> Self {
        Self {
            name: name.into(),
            value,
            color: None,
        }
    }

    pub fn color(mut self, color: impl Into<ChartColor>) -> Self {
        self.color = Some(color.into());
        self
    }
}

impl From<(&str, f64)> for Slice {
    fn from((name, value): (&str, f64)) -> Self {
        Self::new(name, value)
    }
}

/// One period of a [`CandlestickChart`](super::CandlestickChart): its open, high, low and
/// close. A price that is not finite leaves the period empty.
///
/// ```rust
/// # use libero::components::Candle;
/// let monday = Candle::new(20.0, 38.0, 10.0, 34.0);
/// let tuesday: Candle = (34.0, 40.0, 28.0, 30.0).into();
/// # let _ = (monday, tuesday);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

impl Candle {
    pub fn new(open: f64, high: f64, low: f64, close: f64) -> Self {
        Self {
            open,
            high,
            low,
            close,
        }
    }

    /// Open, high, low and close, when all are finite.
    pub(crate) fn prices(&self) -> Option<[f64; 4]> {
        let prices = [self.open, self.high, self.low, self.close];
        prices
            .iter()
            .all(|price| price.is_finite())
            .then_some(prices)
    }
}

impl From<(f64, f64, f64, f64)> for Candle {
    fn from((open, high, low, close): (f64, f64, f64, f64)) -> Self {
        Self::new(open, high, low, close)
    }
}

/// One point of a [`ScatterSeries`]: where it sits and, for a bubble, its size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScatterPoint {
    pub x: f64,
    pub y: f64,
    /// A bubble's size: its area grows with it. `None`, a plain point.
    pub size: Option<f64>,
}

impl ScatterPoint {
    /// Drawn only with a finite x and y; a size that is not finite counts as none.
    pub(crate) fn finite(&self) -> Option<(f64, f64, Option<f64>)> {
        (self.x.is_finite() && self.y.is_finite())
            .then(|| (self.x, self.y, self.size.filter(|size| size.is_finite())))
    }
}

impl<X: Into<Value>, Y: Into<Value>> From<(X, Y)> for ScatterPoint {
    fn from((x, y): (X, Y)) -> Self {
        Self {
            x: x.into().0.unwrap_or(f64::NAN),
            y: y.into().0.unwrap_or(f64::NAN),
            size: None,
        }
    }
}

impl<X: Into<Value>, Y: Into<Value>, S: Into<Value>> From<(X, Y, S)> for ScatterPoint {
    fn from((x, y, size): (X, Y, S)) -> Self {
        Self {
            size: size.into().0,
            ..Self::from((x, y))
        }
    }
}

/// One group of a [`ScatterChart`](super::ScatterChart): a name and its points.
///
/// ```rust
/// # use libero::components::ScatterSeries;
/// # use libero::theme::PointSymbol;
/// let heights = ScatterSeries::new("Girls", [(150, 42.5), (160, 51.0)]).color("success");
/// let bubbles = ScatterSeries::new("Cities", [(1.2, 40.0, 900), (3.4, 22.0, 120)])
///     .symbol(PointSymbol::Square);
/// # let _ = (heights, bubbles);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ScatterSeries {
    pub name: String,
    /// A point with an x or y that is not finite is left out.
    pub points: Vec<ScatterPoint>,
    /// Unset, the theme's palette colour for the series' place.
    pub color: Option<ChartColor>,
    /// The shape of its points and its legend key. Unset, circle, triangle, square and
    /// diamond by the series' place; a bubble is a circle.
    pub symbol: Option<PointSymbol>,
    /// Written after each y value in the tooltip and summary. Unset, the chart's `unit`.
    pub unit: Option<String>,
}

impl ScatterSeries {
    pub fn new<P: Into<ScatterPoint>>(
        name: impl Into<String>,
        points: impl IntoIterator<Item = P>,
    ) -> Self {
        Self {
            name: name.into(),
            points: points.into_iter().map(Into::into).collect(),
            color: None,
            symbol: None,
            unit: None,
        }
    }

    /// One point per row: `ScatterSeries::from_rows("Sales", &rows, |r| (r.price, r.sold))`.
    pub fn from_rows<T, P: Into<ScatterPoint>>(
        name: impl Into<String>,
        rows: &[T],
        point: impl Fn(&T) -> P,
    ) -> Self {
        Self::new(name, rows.iter().map(point))
    }

    pub fn color(mut self, color: impl Into<ChartColor>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn symbol(mut self, symbol: PointSymbol) -> Self {
        self.symbol = Some(symbol);
        self
    }

    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Any point with a size makes the series bubbles.
    pub(crate) fn is_bubble(&self) -> bool {
        self.points
            .iter()
            .any(|point| point.finite().is_some_and(|p| p.2.is_some()))
    }
}

/// The curve a [`ScatterChart`](super::ScatterChart) fits through each series' points, by
/// least squares, in the series' colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trend {
    /// A straight line.
    Linear,
    /// `y = a·e^(b·x)`, fitted to the points above 0.
    Exponential,
    /// `y = a + b·ln(x)`, fitted to the points right of 0.
    Logarithmic,
    /// A polynomial of this order, held to 2 to 6: a first order is `Linear`, a higher one
    /// only chases the points' noise.
    Polynomial(u8),
}

/// A point of a chart, as a click or a focus reports it.
#[derive(Clone, Debug, PartialEq)]
pub struct ChartPoint {
    /// The series' place in `series`.
    pub series: usize,
    pub series_name: String,
    /// The point's place on the x axis; a scatter point's place in its series.
    pub index: usize,
    /// The x value as the chart writes it.
    pub x: String,
    pub value: f64,
}

/// How a chart writes its values: on the axis, in the tooltip, in the summary.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[allow(
    unpredictable_function_pointer_comparisons,
    reason = "`Custom` compares by address; a miss on a copied closure only re-renders"
)]
pub enum ValueFormat {
    /// Up to two decimals, trailing zeros dropped; two significant digits below 0.01, so
    /// `0.0012` never reads `0`.
    #[default]
    Auto,
    /// Exactly this many decimals.
    Fixed(u8),
    /// A fraction as a percentage: `0.42` is `42%`.
    Percent,
    /// Your own text, unit and decimal separator included.
    Custom(fn(f64) -> String),
}

impl ValueFormat {
    /// `value` with the `unit` after a space, in the locale's decimal and group separators.
    pub(crate) fn format(&self, value: f64, unit: Option<&str>, formats: &Formats) -> String {
        let number = match self {
            Self::Custom(format) => return format(value),
            Self::Auto => trim_zeros(format!("{value:.*}", auto_decimals(value))),
            Self::Fixed(decimals) => format!("{value:.*}", usize::from(*decimals)),
            Self::Percent => {
                let share = trim_zeros(format!("{:.1}", value * 100.0));
                return format!("{}%", formats.write_number(&share));
            }
        };
        let number = formats.write_number(&number);
        match unit.filter(|unit| !unit.is_empty()) {
            Some(unit) => format!("{number} {unit}"),
            None => number,
        }
    }
}

/// `ValueFormat::Auto`'s decimals: two, or two significant digits for a value below 0.01.
fn auto_decimals(value: f64) -> usize {
    match value.abs() {
        small if small > 0.0 && small < 0.01 => (-small.log10()).floor() as usize + 2,
        _ => 2,
    }
    .min(15)
}

/// `formats` without a group separator, for an x axis' numbers.
pub(crate) fn ungrouped(formats: &Formats) -> Formats {
    Formats {
        group_separator: "",
        ..*formats
    }
}

fn trim_zeros(number: String) -> String {
    if !number.contains('.') {
        return number;
    }
    let trimmed = number.trim_end_matches('0').trim_end_matches('.');
    match trimmed {
        "-0" => "0".to_string(),
        trimmed => trimmed.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_value_that_is_not_finite_is_a_gap() {
        let series = Series::new("a", [1.0, f64::NAN, f64::INFINITY]);
        assert_eq!(series.values, [Some(1.0), None, None]);
        assert_eq!(Series::new("b", [Some(2), None]).values, [Some(2.0), None]);
    }

    #[test]
    fn a_value_is_written_in_the_locale() {
        let (en, de) = (&Formats::AMERICAN, &Formats::GERMAN);
        let auto = ValueFormat::Auto;
        assert_eq!(auto.format(2.5, Some("kg"), de), "2,5 kg");
        assert_eq!(auto.format(3.0, None, en), "3");
        // A value that is not 0 never reads 0 (3518).
        assert_eq!(auto.format(-0.001, None, en), "-0.001");
        assert_eq!(auto.format(0.0012, None, de), "0,0012");
        assert_eq!(auto.format(0.00999, None, en), "0.01");
        assert_eq!(auto.format(0.0, None, en), "0");
        assert_eq!(ValueFormat::Fixed(2).format(1.0, None, en), "1.00");
        assert_eq!(ValueFormat::Percent.format(0.425, None, de), "42,5%");
    }

    /// Todo 3335 s5: the whole is grouped in threes with the locale's separator.
    #[test]
    fn a_large_value_is_grouped_in_the_locale() {
        let (en, de) = (&Formats::AMERICAN, &Formats::GERMAN);
        let auto = ValueFormat::Auto;
        assert_eq!(auto.format(12_500.5, Some("EUR"), en), "12,500.5 EUR");
        assert_eq!(auto.format(12_500.5, Some("EUR"), de), "12.500,5 EUR");
        assert_eq!(auto.format(-1_234_567.0, None, en), "-1,234,567");
        assert_eq!(auto.format(999.0, None, en), "999");
        assert_eq!(
            ValueFormat::Fixed(1).format(100_000.0, None, de),
            "100.000,0"
        );
        assert_eq!(ValueFormat::Percent.format(12.5, None, en), "1,250%");
        let none = Formats {
            group_separator: "",
            ..Formats::AMERICAN
        };
        assert_eq!(auto.format(12_500.0, None, &none), "12500");
    }

    /// A numeric x axis is often years: its labels stay ungrouped.
    #[test]
    fn numbers_on_the_x_axis_are_not_grouped() {
        let years = XValues::from(vec![2025, 2026]);
        assert_eq!(
            years.labels(&ValueFormat::Auto, &Formats::AMERICAN),
            ["2025", "2026"]
        );
    }

    #[test]
    fn a_colour_reads_a_theme_name_or_a_hex() {
        assert_eq!(
            ChartColor::from("success.7"),
            ChartColor::Palette(Color::Success, ColorShade::S7)
        );
        assert_eq!(
            ChartColor::from("#102030"),
            ChartColor::Hex(HexColor::new(0x102030))
        );
    }
}
