use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::theme::{Color, ColorShade, CssVar};
#[cfg(any(feature = "charts", test))]
use crate::{
    theme::{HexColor, Theme},
    tokens::{Ends, ShadeRamp},
};

pub const CHART_ASPECT_RATIO: CssVar = CssVar::new("--lsx-chart-aspect-ratio");
pub const CHART_LEGEND_GAP: CssVar = CssVar::new("--lsx-chart-legend-gap");
pub const CHART_SWATCH_SIZE: CssVar = CssVar::new("--lsx-chart-swatch-size");

str_enum! {
    /// How a line runs between its points.
    pub enum Curve {
        /// Straight segments: the line never claims a value between points.
        #[default]
        Linear = "linear",
        /// A smooth curve through every point. It may overshoot between them.
        Smooth = "smooth",
        /// Holds each value until the next point.
        Step = "step",
    }
}

str_enum! {
    /// Where a chart's legend sits, or `None` for no legend. Start and end follow the text direction.
    pub enum LegendPosition {
        #[default]
        Bottom = "bottom",
        Top = "top",
        Start = "start",
        End = "end",
        None = "none",
    }
}

str_enum! {
    /// The chart's data as a table: hidden for assistive technology, shown under the chart, or left out.
    pub enum ChartTable {
        #[default]
        Hidden = "hidden",
        Visible = "visible",
        Off = "off",
    }
}

/// Theme defaults for `LineChart` and the other charts, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartDefaults {
    /// Width over height of the plot.
    pub aspect_ratio: f32,
    /// Series colours in order, from the brand ramp; the ninth series starts over.
    pub palette: [(Color, ColorShade); 8],
    pub curve: Curve,
    pub legend: LegendPosition,
    pub grid: bool,
    /// A dot on every point of a line, up to [`points_limit`](Self::points_limit) points.
    pub points: bool,
    pub points_limit: usize,
    pub stroke_width: f32,
    /// The most cells the data table holds; a larger chart keeps only its summary.
    pub table_cell_cap: usize,
    pub legend_gap: &'static str,
    pub swatch_size: &'static str,
}

impl ChartDefaults {
    pub const DEFAULT: Self = Self {
        aspect_ratio: 16.0 / 9.0,
        palette: [
            (Color::Primary, ColorShade::S6),
            (Color::Warning, ColorShade::S7),
            (Color::Success, ColorShade::S7),
            (Color::Error, ColorShade::S6),
            (Color::Secondary, ColorShade::S6),
            (Color::Info, ColorShade::S7),
            (Color::Neutral, ColorShade::S6),
            (Color::Primary, ColorShade::S9),
        ],
        curve: Curve::Linear,
        legend: LegendPosition::Bottom,
        grid: true,
        points: true,
        points_limit: 200,
        stroke_width: 2.0,
        table_cell_cap: 2_000,
        legend_gap: "16px",
        swatch_size: "12px",
    };
}

impl ToCssDeclarations for ChartDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            CHART_ASPECT_RATIO.declare(self.aspect_ratio.to_string()),
            CHART_LEGEND_GAP.declare(self.legend_gap),
            CHART_SWATCH_SIZE.declare(self.swatch_size),
        ]
    }
}

/// The colours a chart's SVG is drawn in, as hex: its renderer resolves no `var()`.
#[cfg(any(feature = "charts", test))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ChartInk {
    /// Axis labels: the dimmed text role.
    pub(crate) label: HexColor,
    pub(crate) axis: HexColor,
    pub(crate) grid: HexColor,
}

#[cfg(any(feature = "charts", test))]
impl ChartInk {
    pub(crate) fn of(theme: &Theme) -> Self {
        let ends = Ends {
            surface: theme.surface,
            ink: theme.ink,
        };
        let cards: Vec<HexColor> = HexColor::parse(theme.paper.background)
            .into_iter()
            .collect();
        let muted = |shade| theme.muted.shade(shade, ShadeRamp::Neutral, ends);
        Self {
            label: theme
                .muted
                .text_base(ShadeRamp::Neutral, ends, &cards)
                .shade(ColorShade::S6, ShadeRamp::Neutral, ends),
            axis: muted(ColorShade::S6),
            // Decorative, like a table's row lines.
            grid: muted(ColorShade::S3),
        }
    }
}

#[cfg(any(feature = "charts", test))]
/// `color` at `shade` of the brand ramp in `theme`, as the `--lsx-<color>-<shade>` var paints it.
pub(crate) fn chart_hex(theme: &Theme, color: Color, shade: ColorShade) -> HexColor {
    let base = super::gradient::palette(theme, color).unwrap_or(theme.ink);
    if matches!(color, Color::Ink | Color::Surface) {
        return base;
    }
    let ends = Ends {
        surface: theme.surface,
        ink: theme.ink,
    };
    base.shade(shade, color.shade_ramp(), ends)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::Stylesheet;

    /// The hex a series is drawn in is the one the page's `--lsx-primary-6` names.
    #[test]
    fn a_series_hex_is_the_brand_var_s_value() {
        let css = Stylesheet::from(&Theme::DEFAULT).as_str().to_string();
        let hex = chart_hex(&Theme::DEFAULT, Color::Primary, ColorShade::S6).to_string();
        assert!(css.contains(&format!("--lsx-primary-6:{hex}")), "{hex}");
    }

    /// Dark pages get their own ink: the labels flip with the scheme.
    #[test]
    fn the_ink_follows_the_scheme() {
        let (light, dark) = (ChartInk::of(&Theme::DEFAULT), ChartInk::of(&Theme::DARK));
        assert_ne!(light, dark);
        assert!(light.label.contrast_ratio(Theme::DEFAULT.surface) >= 4.5);
        assert!(dark.label.contrast_ratio(Theme::DARK.surface) >= 4.5);
    }
}
