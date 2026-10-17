//! A candlestick chart's spec and drawing, its rising candles and key.

use super::*;
use std::fmt::Write as _;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CandleSpec {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) x: Vec<String>,
    pub(crate) name: String,
    /// Each period's open, high, low and close; `None`, a gap.
    pub(crate) candles: Vec<Option<[f64; 4]>>,
    pub(crate) up: HexColor,
    pub(crate) down: HexColor,
    /// Ticks for the open and the close on a line, instead of bodies.
    pub(crate) ohlc: bool,
    /// The names of a rising and a falling candle, keyed under the plot.
    pub(crate) key: Option<(String, String)>,
    pub(crate) axes: Axes,
    pub(crate) grid: bool,
    pub(crate) ink: ChartInk,
    pub(crate) format: ValueFormat,
    pub(crate) unit: Option<String>,
    pub(crate) formats: &'static Formats,
}

/// A rising candle is drawn in this, then hollowed out: the renderer's wick takes its fill.
pub(super) const UP_FILL: InkColor = InkColor {
    r: 1,
    g: 2,
    b: 4,
    a: 255,
};
pub(super) const UP_HEX: &str = "#010204";

pub(crate) fn draw_candles(spec: &CandleSpec) -> Option<Drawn> {
    // The renderer reads open, close, lowest, highest.
    let data = spec
        .candles
        .iter()
        .flat_map(|candle| match candle {
            Some([open, high, low, close]) => [*open, *close, *low, *high].map(|v| Some(v as f32)),
            None => [None; 4],
        })
        .collect();
    let series = charts_rs::Series::new_nullable(spec.name.clone(), data);
    let mut chart = charts_rs::CandlestickChart::new(vec![series], spec.x.clone());
    let room = match spec.key {
        Some(_) => KEY_ROOM.min(spec.height / 3.0),
        None => 0.0,
    };
    chart.width = spec.width;
    chart.height = spec.height - room;
    chart.background_color = InkColor::transparent();
    chart.title.text = String::new();
    chart.legend.show = Some(false);
    chart.tooltip.show = true;
    chart.animation = None;
    chart.candlestick_style = match spec.ohlc {
        true => charts_rs::CandlestickStyle::Ohlc,
        false => charts_rs::CandlestickStyle::Candle,
    };
    chart.candlestick_up_color = UP_FILL;
    chart.candlestick_up_border_color = ink(spec.up);
    chart.candlestick_down_color = ink(spec.down);
    chart.candlestick_down_border_color = ink(spec.down);
    chart.grid.stroke_color = match spec.grid {
        true => ink(spec.ink.grid),
        false => InkColor::transparent(),
    };
    chart.x_axis.stroke_color = ink(spec.ink.axis);
    chart.x_axis.font.color = ink(spec.ink.label);
    chart.x_axis.label_overflow = AxisLabelOverflow::Ellipsis;
    let unit = unit_suffix(spec.unit.as_deref());
    if let Some(y) = chart.y_axis_configs.first_mut() {
        y.stroke_color = ink(spec.ink.axis);
        y.font.color = ink(spec.ink.label);
        y.formatter = Some(format!("{AXIS_MARK}{{c}}{unit}"));
    }
    spec.axes
        .style(&mut chart.base, &mut chart.y_axis_configs, false);
    // Prices rarely start near 0: the axis spans the candles only.
    if let Some(left) = chart.y_axis_configs.first_mut() {
        round_axis(&chart.base.series_list, 0, left, None, false);
    }
    let write = |value: f64| {
        spec.format
            .format(value, spec.unit.as_deref(), spec.formats)
    };
    let auto = spec.format == ValueFormat::Auto;
    let (unit1, compact) = (spec.unit.as_deref(), &spec.axes.compact);
    let axis = |svg: &str| axis_write(svg, AXIS_MARK, auto, unit1, spec.formats, compact, write);
    let svg = fitted(&mut chart, spec.width, spec.height, |chart| {
        let mut svg = chart.svg().ok()?;
        let exact = exact_axes(&svg, &chart.y_axis_configs, LabelEdge::Side);
        if let Some(width) = y_width(&svg, &exact, AXIS_MARK, axis(&exact)) {
            if let Some(y) = chart.y_axis_configs.first_mut() {
                y.width = Some(width);
            }
            svg = chart.svg().ok()?;
        }
        Some(pin_to_axis(&svg))
    })?;
    let svg = exact_axes(&svg, &chart.y_axis_configs, LabelEdge::Side);
    let svg = fit_x_labels(&svg, &spec.x, spec.width, spec.height);
    let (svg, triggers) = inert(&svg, axis(&svg), "", CircleTriggers::Dropped);
    let expected: Vec<_> = (spec.candles.iter().enumerate())
        .filter_map(|(i, candle)| Some((0, i, candle.as_ref()?[3], spec.name.as_str())))
        .collect();
    let marks = match_marks(&expected, &triggers)
        .into_iter()
        .map(|mark| Mark {
            shape: whole_candle(&svg, &mark.shape),
            ..mark
        })
        .collect();
    let svg = hollow_rising(&svg, spec.up, spec.ink.surface, spec.ohlc);
    let plot = Plot::of(&svg, spec.width, spec.height);
    let svg = match &spec.key {
        Some(names) => with_candle_key(&svg, spec, names, plot, spec.height - room),
        None => svg,
    };
    Some(Drawn {
        geometry: Geometry {
            width: spec.width,
            height: spec.height,
            plot,
            marks,
            hole: 0.0,
            share_of: None,
            cells: false,
        },
        svg: spec.ink.paint(svg),
    })
}

/// Room under a candlestick chart's plot for its key.
pub(super) const KEY_ROOM: f32 = 28.0;

/// A rising and a falling candle, or OHLC bar, each by its name, centred under the plot.
pub(super) fn with_candle_key(
    svg: &str,
    spec: &CandleSpec,
    (up, down): &(String, String),
    plot: Plot,
    top: f32,
) -> String {
    const SIZE: f32 = SCALE_BAR;
    let y = top + 9.0;
    let room = |name: &str| SIZE + 6.0 + text_width(&[("font-size", "12")], name);
    let mut x = (plot.left + plot.right - room(up) - 16.0 - room(down)) / 2.0;
    let mut key = String::from("<g>");
    for (name, rising) in [(up, true), (down, false)] {
        let color = if rising { spec.up } else { spec.down };
        key.push_str(&match (spec.ohlc, rising) {
            // The open's tick left, the close's right: low to high when rising.
            (true, _) => {
                let (open, close) = if rising { (y + 8.0, y + 2.0) } else { (y + 2.0, y + 8.0) };
                let middle = x + SIZE / 2.0;
                format!(
                    "<g stroke=\"{color}\" stroke-width=\"1.5\"><line x1=\"{middle}\" y1=\"{y}\" x2=\"{middle}\" y2=\"{}\"/><line x1=\"{x}\" y1=\"{open}\" x2=\"{middle}\" y2=\"{open}\"/><line x1=\"{middle}\" y1=\"{close}\" x2=\"{}\" y2=\"{close}\"/></g>",
                    y + SIZE,
                    x + SIZE
                )
            }
            (false, true) => format!(
                "<rect x=\"{x}\" y=\"{y}\" width=\"{SIZE}\" height=\"{SIZE}\" fill=\"{}\" stroke=\"{color}\"/>",
                spec.ink.surface
            ),
            (false, false) => format!(
                "<rect x=\"{x}\" y=\"{y}\" width=\"{SIZE}\" height=\"{SIZE}\" fill=\"{color}\" stroke=\"{color}\"/>"
            ),
        });
        let _ = write!(
            key,
            "<text x=\"{}\" y=\"{}\" font-size=\"12\" font-family=\"inherit\" fill=\"{}\">{}</text>",
            x + SIZE + 6.0,
            y + SIZE,
            spec.ink.label,
            escape(name)
        );
        x += room(name) + 16.0;
    }
    key.push_str("</g>");
    appended(svg, spec.width, spec.height, &key)
}

/// A candle's body stretched over its wick, from the high to the low, so the tooltip sits
/// above it all.
pub(super) fn whole_candle(svg: &str, body: &MarkShape) -> MarkShape {
    let MarkShape::Rect {
        x,
        y,
        width,
        height,
    } = *body
    else {
        return body.clone();
    };
    let middle = x + width / 2.0;
    let (mut top, mut bottom) = (y, y + height);
    let mut rest = svg;
    while let Some(open) = rest.find("<line ") {
        let close = rest[open..]
            .find('>')
            .map_or(rest.len(), |close| open + close + 1);
        let attributes = attributes(&rest[open..close]);
        rest = &rest[close..];
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        if let (Some(x1), Some(x2), Some(y1), Some(y2)) =
            (number("x1"), number("x2"), number("y1"), number("y2"))
            && x1 == x2
            && (x1 - middle).abs() <= 0.5
        {
            top = top.min(y1.min(y2));
            bottom = bottom.max(y1.max(y2));
        }
    }
    MarkShape::Rect {
        x,
        y: top,
        width,
        height: bottom - top,
    }
}

/// Rising candles in `up` as outlines on the `surface`, a flat one as a line in `up`.
pub(super) fn hollow_rising(svg: &str, up: HexColor, surface: HexColor, ohlc: bool) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(start) = rest.find("<rect ") {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find('>')
            .map_or(rest.len(), |end| start + end + 1);
        let tag = &rest[start..end];
        rest = &rest[end..];
        let attributes = attributes(tag);
        if ohlc || attribute(&attributes, "fill") != Some(UP_HEX) {
            out.push_str(tag);
            continue;
        }
        let flat = attribute(&attributes, "height")
            .and_then(|height| height.parse::<f32>().ok())
            .is_some_and(|height| height <= 1.0);
        let kept: String = attributes
            .iter()
            .filter(|(name, _)| {
                !matches!(*name, "fill" | "stroke" | "stroke-opacity" | "fill-opacity")
            })
            .map(|(name, value)| format!(" {name}=\"{value}\""))
            .collect();
        out.push_str(&match flat {
            true => format!("<rect{kept} fill=\"{up}\"/>"),
            false => format!("<rect{kept} fill=\"{surface}\" stroke=\"{up}\"/>"),
        });
    }
    out.push_str(rest);
    out.replace(UP_HEX, &up.to_string())
}
