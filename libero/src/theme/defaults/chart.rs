use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::theme::{Color, ColorShade, CssVar};
use crate::tokens::Size;
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
    /// How a bar chart sets its series' bars at one x value.
    pub enum BarLayout {
        /// Side by side.
        #[default]
        Grouped = "grouped",
        /// On top of each other: the stack is the total.
        Stacked = "stacked",
        /// Stacked, every stack scaled to 100%: each bar is its series' share.
        Percent = "percent",
    }
}

str_enum! {
    /// What a pie chart writes beside each slice.
    pub enum PieLabels {
        None = "none",
        Value = "value",
        /// The slice's share of the whole.
        #[default]
        Percent = "percent",
        Name = "name",
    }
}

/// Theme defaults for `LineChart` and the other charts, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartDefaults {
    /// Width over height of the plot.
    pub aspect_ratio: f32,
    /// Series colours in order, from the brand ramp. Each is drawn at least 3:1 on the page
    /// (a step darker if not), one that reads like an earlier one is skipped, and past the
    /// last the colours start over, a line dashed.
    pub palette: [(Color, ColorShade); 8],
    pub curve: Curve,
    pub legend: LegendPosition,
    pub grid: bool,
    /// A dot on every point of a line, up to [`points_limit`](Self::points_limit) points.
    pub points: bool,
    pub points_limit: usize,
    /// A tooltip and a crosshair under the pointer and the keys.
    pub tooltip: bool,
    pub stroke_width: f32,
    /// The steps a line or bar chart's value axis is cut into.
    pub y_ticks: usize,
    pub bar_layout: BarLayout,
    /// Corner radius of a bar's free end, a step on the theme's radius scale.
    pub bar_radius: Size,
    /// Each bar's value written at its end.
    pub value_labels: bool,
    /// A donut's ring width over the pie's radius.
    pub donut_thickness: f32,
    pub pie_labels: PieLabels,
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
            // S9: at S7 its cyan falls together with the blue for a tritan eye.
            (Color::Info, ColorShade::S9),
            (Color::Neutral, ColorShade::S6),
            (Color::Primary, ColorShade::S9),
        ],
        curve: Curve::Linear,
        legend: LegendPosition::Bottom,
        grid: true,
        points: true,
        points_limit: 200,
        tooltip: true,
        stroke_width: 2.0,
        y_ticks: 6,
        bar_layout: BarLayout::Grouped,
        bar_radius: Size::Xs,
        value_labels: false,
        donut_thickness: 0.4,
        pie_labels: PieLabels::Percent,
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
    /// Parts two pie slices.
    pub(crate) surface: HexColor,
}

#[cfg(any(feature = "charts", test))]
impl ChartInk {
    /// Black or white, whichever reads better on `fill`: one of them always passes 4.5:1,
    /// where a theme's dimmed ends may not.
    pub(crate) fn on(&self, fill: HexColor) -> HexColor {
        let (black, white) = (HexColor::new(0x000000), HexColor::new(0xFFFFFF));
        match black.contrast_ratio(fill) >= white.contrast_ratio(fill) {
            true => black,
            false => white,
        }
    }
}

#[cfg(any(feature = "charts", test))]
impl ChartInk {
    /// Under forced colours: stand-ins [`paint`](Self::paint) trades for the system colours.
    const FORCED: Self = Self {
        label: HexColor::new(0x020101),
        axis: HexColor::new(0x020102),
        grid: HexColor::new(0x020103),
        surface: HexColor::new(0x020104),
    };

    pub(crate) fn of(theme: &Theme, forced: bool) -> Self {
        if forced {
            return Self::FORCED;
        }
        let ends = Ends {
            surface: theme.surface,
            ink: theme.ink,
        };
        let cards: Vec<HexColor> = HexColor::parse(theme.paper.background)
            .into_iter()
            .collect();
        let muted = |shade| theme.muted.shade(shade, ShadeRamp::Neutral, ends);
        let grounds = grounds(theme, false);
        // The axes frame the marks' values: 3:1 like the marks.
        let axis = SHADES[ColorShade::S6 as usize - 1..]
            .iter()
            .map(|shade| muted(*shade))
            .find(|hex| reaches(*hex, &grounds, MARK_CONTRAST))
            .unwrap_or(theme.ink);
        let text = theme.muted.text_base(ShadeRamp::Neutral, ends, &cards);
        // A dimmed ramp short of 4.5:1 on a tinted card steps on toward the ink.
        let label = SHADES[ColorShade::S6 as usize - 1..]
            .iter()
            .map(|shade| text.shade(*shade, ShadeRamp::Neutral, ends))
            .find(|hex| reaches(*hex, &grounds, 4.5))
            .unwrap_or(theme.ink);
        Self {
            label,
            axis,
            // Decorative, like a table's row lines.
            grid: muted(ColorShade::S3),
            surface: theme.surface,
        }
    }

    /// The drawn SVG in the user's system colours where it was drawn in the forced stand-ins.
    pub(crate) fn paint(&self, svg: String) -> String {
        if *self != Self::FORCED {
            return svg;
        }
        [
            (self.label, "CanvasText"),
            (self.axis, "CanvasText"),
            (self.grid, "GrayText"),
            (self.surface, "Canvas"),
        ]
        .into_iter()
        .fold(svg, |svg, (hex, system)| {
            svg.replace(&hex.to_string(), system)
        })
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

/// A mark's floor on the page and on a card: non-text contrast (WCAG 1.4.11).
#[cfg(any(feature = "charts", test))]
const MARK_CONTRAST: f32 = 3.0;
/// The least OKLab distance two series keep, in normal and in each simulated colour vision.
#[cfg(any(feature = "charts", test))]
const SERIES_DISTANCE: f32 = 0.03;

#[cfg(any(feature = "charts", test))]
const SHADES: [ColorShade; 9] = [
    ColorShade::S1,
    ColorShade::S2,
    ColorShade::S3,
    ColorShade::S4,
    ColorShade::S5,
    ColorShade::S6,
    ColorShade::S7,
    ColorShade::S8,
    ColorShade::S9,
];

/// The page and, when it is a colour, a card's background: where a chart may sit.
/// Under forced colours, a `Canvas` either black or white.
#[cfg(any(feature = "charts", test))]
fn grounds(theme: &Theme, forced: bool) -> Vec<HexColor> {
    if forced {
        return vec![HexColor::new(0x000000), HexColor::new(0xFFFFFF)];
    }
    std::iter::once(theme.surface)
        .chain(HexColor::parse(theme.paper.background))
        .collect()
}

#[cfg(any(feature = "charts", test))]
fn reaches(hex: HexColor, grounds: &[HexColor], floor: f32) -> bool {
    grounds
        .iter()
        .all(|ground| hex.contrast_ratio(*ground) >= floor)
}

/// The theme palette as series are drawn in it: each colour stepped until it reaches 3:1 on
/// the page and on a card (on black and white under forced colours); one that reads like an
/// earlier one is left out.
#[cfg(any(feature = "charts", test))]
pub(crate) fn chart_palette(theme: &Theme, forced: bool) -> Vec<HexColor> {
    let grounds = grounds(theme, forced);
    let mut kept: Vec<HexColor> = Vec::with_capacity(theme.chart.palette.len());
    for (color, shade) in theme.chart.palette {
        let hex = legible(theme, color, shade, &grounds);
        if kept.iter().all(|earlier| distinct(*earlier, hex)) {
            kept.push(hex);
        }
    }
    kept
}

/// A series' own theme colour as it is drawn: stepped to 3:1 like the palette's.
#[cfg(feature = "charts")]
pub(crate) fn chart_color(
    theme: &Theme,
    color: Color,
    shade: ColorShade,
    forced: bool,
) -> HexColor {
    legible(theme, color, shade, &grounds(theme, forced))
}

/// A grey that reaches 3:1 on black, on white and on most pages: the last resort.
#[cfg(any(feature = "charts", test))]
const MIDDLE: HexColor = HexColor::new(0x767676);

/// `color` at `shade`, or the nearest shade that reaches 3:1, toward the ink first;
/// past the ramp, mixed into the ink, then into a middle grey.
#[cfg(any(feature = "charts", test))]
fn legible(theme: &Theme, color: Color, shade: ColorShade, grounds: &[HexColor]) -> HexColor {
    let from = shade as usize - 1;
    let ramp = SHADES[from..].iter().chain(SHADES[..from].iter().rev());
    let base = chart_hex(theme, color, shade);
    let mixes = [theme.ink, MIDDLE]
        .into_iter()
        .flat_map(|to| (1..=10).map(move |step| blend(base, to, step as f32 / 10.0)));
    ramp.map(|shade| chart_hex(theme, color, *shade))
        .chain(mixes)
        .find(|hex| reaches(*hex, grounds, MARK_CONTRAST))
        .unwrap_or(theme.ink)
}

#[cfg(any(feature = "charts", test))]
fn blend(from: HexColor, to: HexColor, weight: f32) -> HexColor {
    let channel =
        |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * weight).round() as u32;
    HexColor::new(
        (channel(from.r(), to.r()) << 16)
            | (channel(from.g(), to.g()) << 8)
            | channel(from.b(), to.b()),
    )
}

/// Far enough apart in OKLab for normal vision and for protan, deutan and tritan vision.
#[cfg(any(feature = "charts", test))]
pub(crate) fn distinct(a: HexColor, b: HexColor) -> bool {
    series_distance(a, b) >= SERIES_DISTANCE
}

/// The smallest OKLab distance of `a` and `b` over normal and the three dichromat visions.
#[cfg(any(feature = "charts", test))]
pub(crate) fn series_distance(a: HexColor, b: HexColor) -> f32 {
    // Machado, Oliveira and Fernandes 2009, severity 1, on linear RGB.
    const VISIONS: [[[f32; 3]; 3]; 4] = [
        [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        [
            [0.152_286, 1.052_583, -0.204_868],
            [0.114_503, 0.786_281, 0.099_216],
            [-0.003_882, -0.048_116, 1.051_998],
        ],
        [
            [0.367_322, 0.860_646, -0.227_968],
            [0.280_085, 0.672_501, 0.047_413],
            [-0.011_820, 0.042_940, 0.968_881],
        ],
        [
            [1.255_528, -0.076_749, -0.178_779],
            [-0.078_411, 0.930_809, 0.147_602],
            [0.004_733, 0.691_367, 0.303_900],
        ],
    ];
    VISIONS
        .iter()
        .map(|vision| {
            let (a, b) = (oklab(seen(a, vision)), oklab(seen(b, vision)));
            ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
        })
        .fold(f32::INFINITY, f32::min)
}

#[cfg(any(feature = "charts", test))]
fn seen(hex: HexColor, vision: &[[f32; 3]; 3]) -> [f32; 3] {
    let linear = |value: u8| {
        let value = f32::from(value) / 255.0;
        match value <= 0.04045 {
            true => value / 12.92,
            false => ((value + 0.055) / 1.055).powf(2.4),
        }
    };
    let rgb = [linear(hex.r()), linear(hex.g()), linear(hex.b())];
    vision.map(|row| (row[0] * rgb[0] + row[1] * rgb[1] + row[2] * rgb[2]).clamp(0.0, 1.0))
}

#[cfg(any(feature = "charts", test))]
fn oklab([r, g, b]: [f32; 3]) -> [f32; 3] {
    let l = (0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b).cbrt();
    let m = (0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b).cbrt();
    let s = (0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b).cbrt();
    [
        0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    ]
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
        let (light, dark) = (
            ChartInk::of(&Theme::DEFAULT, false),
            ChartInk::of(&Theme::DARK, false),
        );
        assert_ne!(light, dark);
        assert!(light.label.contrast_ratio(Theme::DEFAULT.surface) >= 4.5);
        assert!(dark.label.contrast_ratio(Theme::DARK.surface) >= 4.5);
    }

    fn shipped() -> impl Iterator<Item = (&'static str, &'static Theme)> {
        crate::theme::ThemeSet::CATALOGUE.iter().flat_map(|set| {
            [Some(set.light_theme()), set.dark_theme()]
                .into_iter()
                .flatten()
                .map(|theme| (set.name(), theme))
        })
    }

    /// A value on a bar reads on every drawn palette colour, in every shipped theme.
    #[test]
    fn text_on_a_bar_reads_on_the_palette() {
        for (name, theme) in shipped() {
            let ink = ChartInk::of(theme, false);
            for bar in chart_palette(theme, false) {
                assert!(ink.on(bar).contrast_ratio(bar) >= 4.5, "{name} {bar}");
            }
        }
    }

    /// Todo 3335 s5: every series colour reaches 3:1 on the page and on a card (1.4.11),
    /// the axes too, the axis labels 4.5:1, in every shipped theme and scheme.
    #[test]
    fn every_shipped_palette_reaches_3_to_1_on_page_and_card() {
        let mut short = Vec::new();
        for (name, theme) in shipped() {
            let grounds = grounds(theme, false);
            let ink = ChartInk::of(theme, false);
            let scheme = theme.surface.color_scheme();
            for hex in chart_palette(theme, false) {
                if !reaches(hex, &grounds, MARK_CONTRAST) {
                    short.push(format!("{name} {scheme} series {hex}"));
                }
            }
            if !reaches(ink.axis, &grounds, MARK_CONTRAST) {
                short.push(format!("{name} {scheme} axis {}", ink.axis));
            }
            if !reaches(ink.label, &grounds, 4.5) {
                short.push(format!("{name} {scheme} label {}", ink.label));
            }
        }
        assert!(short.is_empty(), "{short:?}");
    }

    /// Every two series colours stay apart, also to protan, deutan and tritan eyes,
    /// and every shipped theme keeps at least four of them.
    #[test]
    fn every_shipped_palette_keeps_its_series_apart() {
        for (name, theme) in shipped() {
            let palette = chart_palette(theme, false);
            assert!(palette.len() >= 4, "{name}: {palette:?}");
            for (i, a) in palette.iter().enumerate() {
                for b in &palette[..i] {
                    assert!(distinct(*a, *b), "{name}: {a} {b}");
                }
            }
        }
    }

    /// The default theme keeps its whole palette in either scheme: eight series, eight colours.
    #[test]
    fn the_default_palette_is_whole() {
        assert_eq!(chart_palette(&Theme::DEFAULT, false).len(), 8);
        assert_eq!(chart_palette(&Theme::DARK, false).len(), 8);
    }

    /// Forced colours keep the series' hex on the user's `Canvas`, black or white: every
    /// colour reaches 3:1 on both, the series stay apart.
    #[test]
    fn every_shipped_palette_reaches_3_to_1_on_a_black_and_a_white_canvas() {
        for (name, theme) in shipped() {
            let palette = chart_palette(theme, true);
            assert!(palette.len() >= 3, "{name}: {palette:?}");
            for (i, hex) in palette.iter().enumerate() {
                assert!(
                    reaches(*hex, &grounds(theme, true), MARK_CONTRAST),
                    "{name} {hex}"
                );
                assert!(
                    palette[..i].iter().all(|b| distinct(*hex, *b)),
                    "{name} {hex}"
                );
            }
        }
    }

    /// Under forced colours the axes, grid and labels are drawn in the system colours.
    #[test]
    fn forced_ink_paints_in_system_colours() {
        let ink = ChartInk::of(&Theme::DEFAULT, true);
        let svg = format!(
            "<text fill=\"{}\"/><line stroke=\"{}\"/><line stroke=\"{}\"/><path stroke=\"{}\"/>",
            ink.label, ink.axis, ink.grid, ink.surface
        );
        assert_eq!(
            ink.paint(svg),
            "<text fill=\"CanvasText\"/><line stroke=\"CanvasText\"/><line stroke=\"GrayText\"/>\
             <path stroke=\"Canvas\"/>"
        );
        let plain = ChartInk::of(&Theme::DEFAULT, false);
        assert_eq!(
            plain.paint(format!("{}", plain.label)),
            plain.label.to_string()
        );
    }

    /// Too light for the page, a colour steps toward the ink; one that passes stays.
    #[test]
    fn a_light_colour_steps_to_3_to_1() {
        let theme = &Theme::DEFAULT;
        let grounds = grounds(theme, false);
        let warning = chart_hex(theme, Color::Warning, ColorShade::S7);
        assert!(!reaches(warning, &grounds, MARK_CONTRAST));
        let stepped = legible(theme, Color::Warning, ColorShade::S7, &grounds);
        assert!(reaches(stepped, &grounds, MARK_CONTRAST), "{stepped}");
        let primary = chart_hex(theme, Color::Primary, ColorShade::S6);
        assert_eq!(
            legible(theme, Color::Primary, ColorShade::S6, &grounds),
            primary
        );
    }

    /// Red and green that only differ in hue fall together for a deutan eye.
    #[test]
    fn red_and_green_of_one_lightness_are_not_distinct() {
        let (red, green) = (HexColor::new(0xB8_5C_00), HexColor::new(0x6F_7F_00));
        assert!(!distinct(red, green));
        assert!(distinct(
            HexColor::new(0x22_8B_E6),
            HexColor::new(0xBB_84_03)
        ));
    }
}
