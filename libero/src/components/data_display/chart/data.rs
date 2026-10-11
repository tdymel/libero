use crate::{
    theme::{Color, ColorShade, HexColor},
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
    Categories(Vec<String>),
    /// Numbers, written with the chart's `value_format`, one step apart each.
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

    /// Every x value as the chart writes it.
    pub(crate) fn labels(&self, format: &ValueFormat, decimal: &str) -> Vec<String> {
        match self {
            Self::Categories(values) => values.clone(),
            Self::Numbers(values) => values
                .iter()
                .map(|value| format.format(*value, None, decimal))
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
/// # let _ = (revenue, costs);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Series {
    pub name: String,
    pub values: Vec<Option<f64>>,
    /// Unset, the theme's palette colour for the series' place.
    pub color: Option<ChartColor>,
    /// A dashed line: a second cue beside the colour.
    pub dashed: bool,
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

/// A point of a chart, as a click or a focus reports it.
#[derive(Clone, Debug, PartialEq)]
pub struct ChartPoint {
    /// The series' place in `series`.
    pub series: usize,
    pub series_name: String,
    /// The point's place on the x axis.
    pub index: usize,
    /// The x value as the chart writes it.
    pub x: String,
    pub value: f64,
}

/// How a chart writes its values: on the axis, in the summary, in the table.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[allow(
    unpredictable_function_pointer_comparisons,
    reason = "`Custom` compares by address; a miss on a copied closure only re-renders"
)]
pub enum ValueFormat {
    /// Up to two decimals, trailing zeros dropped.
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
    /// `value` with the `unit` after a space and the locale's decimal separator.
    pub(crate) fn format(&self, value: f64, unit: Option<&str>, decimal: &str) -> String {
        let number = match self {
            Self::Custom(format) => return format(value),
            Self::Auto => trim_zeros(format!("{value:.2}")),
            Self::Fixed(decimals) => format!("{value:.*}", usize::from(*decimals)),
            Self::Percent => {
                return format!("{}%", trim_zeros(format!("{:.1}", value * 100.0)))
                    .replace('.', decimal);
            }
        };
        let number = number.replace('.', decimal);
        match unit.filter(|unit| !unit.is_empty()) {
            Some(unit) => format!("{number} {unit}"),
            None => number,
        }
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
        let auto = ValueFormat::Auto;
        assert_eq!(auto.format(2.5, Some("kg"), ","), "2,5 kg");
        assert_eq!(auto.format(3.0, None, "."), "3");
        assert_eq!(auto.format(-0.001, None, "."), "0");
        assert_eq!(ValueFormat::Fixed(2).format(1.0, None, "."), "1.00");
        assert_eq!(ValueFormat::Percent.format(0.425, None, ","), "42,5%");
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
