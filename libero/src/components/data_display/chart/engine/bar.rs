//! A bar chart's spec and drawing: rounded free ends, error bars and inked labels.

use super::*;
use crate::components::common::Orientation;
use std::fmt::Write as _;

/// Everything a bar chart's SVG depends on.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BarSpec {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) x: Vec<String>,
    pub(crate) series: Vec<SeriesSpec>,
    pub(crate) layout: BarLayout,
    pub(crate) horizontal: bool,
    /// Corner radius of every bar, in pixels.
    pub(crate) radius: f32,
    pub(crate) value_labels: bool,
    pub(crate) axes: Axes,
    /// The value labels in the middle of their bars rather than past their ends.
    pub(crate) labels_inside: bool,
    /// Per series, a bar's own colour per x value; `None`, the series' colour.
    pub(crate) bar_colors: Vec<Vec<Option<HexColor>>>,
    /// Per series, each value's error range; a percent stack draws none.
    pub(crate) errors: Vec<Vec<Option<(f64, f64)>>>,
    /// The share of a category's band its bars fill; `None`, the renderer's 5 px margins.
    pub(crate) bar_width: Option<f32>,
    /// Between two bars of a group, in pixels.
    pub(crate) bar_gap: f32,
    pub(crate) grid: bool,
    pub(crate) ink: ChartInk,
    pub(crate) format: ValueFormat,
    pub(crate) unit: Option<String>,
    /// The right y axis' unit, of its lines.
    pub(crate) y2_unit: Option<String>,
    /// The lines' look, as a line chart's.
    pub(crate) curve: Curve,
    pub(crate) points: bool,
    pub(crate) stroke_width: f32,
    pub(crate) formats: &'static Formats,
}

/// The marks in the renderer's order: every bar series', then every line's.
pub(super) fn bar_marks(series: &[SeriesSpec]) -> Vec<(usize, usize, f64, &str)> {
    let (lines, bars): (Vec<_>, Vec<_>) = expected_marks(series)
        .into_iter()
        .partition(|mark| series[mark.0].extras.line);
    bars.into_iter().chain(lines).collect()
}

/// The settings a vertical and a horizontal bar chart share; both deref to the same base.
macro_rules! style_bars {
    ($chart:ident, $spec:ident, $axes:ident, $unit:expr) => {
        $chart.width = $spec.width;
        $chart.height = $spec.height;
        $chart.background_color = InkColor::transparent();
        $chart.title.text = String::new();
        $chart.legend.show = Some(false);
        // Its triggers are the marks we harvest; `sanitize` drops its tips.
        $chart.tooltip.show = true;
        $chart.animation = None;
        $chart.series.colors = $spec.series.iter().map(|s| ink(s.color)).collect();
        $chart.series.label.formatter = format!("{LABEL_MARK}{{c}}{}", $unit);
        $chart.grid.stroke_color = match $spec.grid {
            true => ink($spec.ink.grid),
            false => InkColor::transparent(),
        };
        $chart.x_axis.stroke_color = ink($spec.ink.axis);
        $chart.x_axis.font.color = ink($spec.ink.label);
        if let Some(y) = $chart.y_axis_configs.first_mut() {
            y.stroke_color = ink($spec.ink.axis);
            y.font.color = ink($spec.ink.label);
            y.formatter = Some(format!("{AXIS_MARK}{{c}}{}", $unit));
        }
        $axes.style(
            &mut $chart.base,
            &mut $chart.y_axis_configs,
            $spec.horizontal,
        );
        let inside = $spec.labels_inside;
        $chart.series_label_position = inside.then_some(Position::Inside);
        $chart.series.label.font.color = match inside {
            true => ON_BAR,
            false => ink($spec.ink.label),
        };
    };
}

pub(crate) fn draw_bar(spec: &BarSpec) -> Option<Drawn> {
    // The renderer lists a horizontal chart's categories bottom up; ours read top down.
    let order: Vec<usize> = if spec.horizontal {
        (0..spec.x.len()).rev().collect()
    } else {
        (0..spec.x.len()).collect()
    };
    let ordered: Vec<SeriesSpec> = spec
        .series
        .iter()
        .map(|series| SeriesSpec {
            values: order
                .iter()
                .map(|&i| series.values.get(i).copied().flatten())
                .collect(),
            ..series.clone()
        })
        .collect();
    // The specs carry mark lines only where a mark stands at its own value.
    let marked = if spec.horizontal {
        Vec::new()
    } else {
        kept_mark_lines(&ordered, &spec.axes)
    };
    // The renderer draws a range on upright grouped bars only; the rest are drawn after it.
    let errors = spec.layout == BarLayout::Grouped && !spec.horizontal;
    let series = bar_series(spec, &order, &ordered, &marked, errors);
    let x = order.iter().map(|&i| spec.x[i].clone()).collect();
    let percent = spec.layout == BarLayout::Percent;
    let write = |value: f64| {
        if percent {
            ValueFormat::Percent.format(value / 100.0, None, spec.formats)
        } else {
            spec.format
                .format(value, spec.unit.as_deref(), spec.formats)
        }
    };
    let write2 = |value: f64| {
        spec.format
            .format(value, spec.y2_unit.as_deref(), spec.formats)
    };
    let auto = spec.format == ValueFormat::Auto;
    let (unit1, compact) = (spec.unit.as_deref(), &spec.axes.compact);
    let axis = |svg: &str| {
        axis_write(
            svg,
            AXIS_MARK,
            auto && !percent,
            unit1,
            spec.formats,
            compact,
            write,
        )
    };
    let axis2 = |svg: &str| {
        let unit = spec.y2_unit.as_deref();
        axis_write(svg, AXIS2_MARK, auto, unit, spec.formats, compact, write2)
    };
    let ranged = (spec.series.iter().zip(&spec.errors))
        .any(|(series, errors)| !series.extras.line && errors.iter().any(Option::is_some));
    let axes = match (percent, ranged && !errors) {
        (true, _) => percent_axes(&spec.axes, &ordered),
        (false, true) => error_axes(&spec.axes, spec),
        (false, false) => spec.axes.clone(),
    };
    let svg = if spec.horizontal {
        horizontal_bars_svg(spec, series, x, &axes, axis)?
    } else {
        vertical_bars_svg(spec, series, x, &axes, axis, axis2)?
    };

    let writer = axis(&svg);
    let svg = if spec.horizontal {
        thin_tick_labels(&svg, AXIS_MARK, &writer)
    } else {
        svg
    };
    let (svg, triggers) = inert(&svg, writer, "", CircleTriggers::Dropped);
    let svg = relabel(&svg, AXIS2_MARK, axis2(&svg));
    // The renderer's margins would leave a narrow band's bars a sliver.
    let narrow = {
        let plot = Plot::of(&svg, spec.width, spec.height);
        let span = if spec.horizontal {
            plot.bottom - plot.top
        } else {
            plot.right - plot.left
        };
        span < 4.0 * RENDERER_MARGIN * spec.x.len() as f32
    };
    let expected = bar_marks(&ordered);
    let (svg, triggers) = if narrow || spec.bar_width.is_some() || spec.bar_gap != RENDERER_GAP {
        relayout(&svg, triggers, &expected, spec)
    } else {
        (svg, triggers)
    };
    let negative: Vec<bool> = expected
        .iter()
        .map(|&(_, _, value, _)| value < 0.0)
        .collect();
    let orientation = match spec.horizontal {
        true => Orientation::Horizontal,
        false => Orientation::Vertical,
    };
    let svg = round_free_ends(&svg, &triggers, &negative, orientation, spec.radius);
    let svg = bar_value_labels(svg, spec, &ordered, write);
    let last = spec.x.len().saturating_sub(1);
    let marks = match_marks(&expected, &triggers)
        .into_iter()
        .map(|mark| Mark {
            index: if spec.horizontal {
                last - mark.index
            } else {
                mark.index
            },
            // The drawn bars of a percent chart carry shares; the mark keeps the value.
            value: spec.series[mark.series].values[order[mark.index]].unwrap_or(mark.value),
            ..mark
        })
        .collect::<Vec<_>>();
    let svg = ink_bar_labels(&svg, &marks, |mark| {
        spec.ink.on(bar_color(spec, mark.series, mark.index))
    });
    let svg = draw_error_bars(svg, &marks, spec);
    // An error bar is drawn in the labels' ink; one on a bar is in the page's.
    let svg = svg.replace(ON_BAR_STROKE, &format!("stroke=\"{}\"", spec.ink.label));
    let plot = Plot::of(&svg, spec.width, spec.height);
    let svg = bar_mark_lines(&svg, spec, &marked, &marks, plot.left);
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

/// A bar's own colour, else its series'.
fn bar_color(spec: &BarSpec, series: usize, index: usize) -> HexColor {
    spec.bar_colors
        .get(series)
        .and_then(|colors| colors.get(index).copied().flatten())
        .unwrap_or(spec.series[series].color)
}

/// The renderer's series in the drawn `order`: the bars, with their ranges when it draws
/// them (`errors`), and the lines over them.
fn bar_series(
    spec: &BarSpec,
    order: &[usize],
    ordered: &[SeriesSpec],
    marked: &[Vec<&(f64, String)>],
    errors: bool,
) -> Vec<charts_rs::Series> {
    let error_at =
        |s: usize, i: usize| spec.errors.get(s).and_then(|e| e.get(i).copied().flatten());
    ordered
        .iter()
        .enumerate()
        .map(|(s, series)| {
            let values = series.values.iter().map(|v| v.map(|v| v as f32)).collect();
            let mut drawn = charts_rs::Series::new_nullable(series.name.clone(), values);
            if let Some(lines) = marked.get(s) {
                drawn.mark_lines = lines.iter().copied().map(renderer_mark_line).collect();
            }
            let ranges: Vec<Option<(f64, f64)>> = order.iter().map(|&i| error_at(s, i)).collect();
            // A line keeps out of the stacks and the bars' slots, its colour by its place.
            if series.extras.line {
                drawn.category = Some(SeriesCategory::Line);
                drawn.index = Some(s);
                drawn.stroke_dash_array = series.dashed.then(|| DASH.to_string());
                drawn.y_axis_index = series.extras.secondary as usize;
                drawn.symbol = Some(symbol(spec.points, spec.stroke_width, series));
                drawn.band = band_of(&ranges, &series.values);
                bend(&mut drawn, spec.curve);
                return drawn;
            }
            drawn.label_show = spec.value_labels;
            drawn.y_axis_index = series.extras.secondary as usize;
            drawn.stack = (spec.layout != BarLayout::Grouped).then(|| "stack".to_string());
            drawn.colors = Some(
                order
                    .iter()
                    .map(|&i| Some(ink(bar_color(spec, s, i))))
                    .collect(),
            );
            if errors && ranges.iter().any(Option::is_some) {
                let end = |pick: fn((f64, f64)) -> f64| {
                    ranges.iter().map(|r| r.map(|r| pick(r) as f32)).collect()
                };
                drawn.error_bar = Some(charts_rs::SeriesBand {
                    lower: end(|r| r.0),
                    upper: end(|r| r.1),
                    stroke_width: None,
                });
            }
            drawn
        })
        .collect()
}

/// A percent chart's values are shares out of 100: they take no unit.
fn bar_unit(spec: &BarSpec) -> String {
    if spec.layout == BarLayout::Percent {
        String::new()
    } else {
        unit_suffix(spec.unit.as_deref())
    }
}

/// The renderer's SVG of lying bars, their category labels cut to a third of the width.
fn horizontal_bars_svg<W: Fn(f64) -> String>(
    spec: &BarSpec,
    series: Vec<charts_rs::Series>,
    x: Vec<String>,
    axes: &Axes,
    axis: impl Fn(&str) -> W,
) -> Option<String> {
    let unit = bar_unit(spec);
    let percent = spec.layout == BarLayout::Percent;
    let mut chart = charts_rs::HorizontalBarChart::new(series, x);
    style_bars!(chart, spec, axes, unit);
    chart.stack_percent = percent;
    round_axes(&chart.base, &mut chart.y_axis_configs, percent);
    // The renderer sizes the category column by its widest label.
    let size = chart.x_axis.font.size.to_string();
    let room = spec.width / 3.0;
    for label in &mut chart.x_axis.data {
        *label = shorten(&[("font-size", &size)], label, room);
    }
    let svg = fitted(&mut chart, spec.width, spec.height, |chart| {
        let svg = chart.svg().ok()?;
        let exact = exact_axes(&svg, &chart.y_axis_configs, LabelEdge::Bottom);
        Some(pin_to_ticks(&svg, AXIS_MARK, spec.width, axis(&exact)))
    })?;
    Some(exact_axes(&svg, &chart.y_axis_configs, LabelEdge::Bottom))
}

/// The renderer's SVG of upright bars, its y axes as wide as their labels.
fn vertical_bars_svg<W: Fn(f64) -> String, W2: Fn(f64) -> String>(
    spec: &BarSpec,
    series: Vec<charts_rs::Series>,
    x: Vec<String>,
    axes: &Axes,
    axis: impl Fn(&str) -> W,
    axis2: impl Fn(&str) -> W2,
) -> Option<String> {
    let unit = bar_unit(spec);
    let percent = spec.layout == BarLayout::Percent;
    let mut chart = charts_rs::BarChart::new(series, x.clone());
    style_bars!(chart, spec, axes, unit);
    chart.x_axis.label_overflow = AxisLabelOverflow::Ellipsis;
    chart.stack_percent = percent;
    chart.series.stroke_width = spec.stroke_width;
    if spec.series.iter().any(|s| s.extras.secondary) {
        second_axis(&mut chart.y_axis_configs, &spec.y2_unit);
        axes.style(&mut chart.base, &mut chart.y_axis_configs, false);
    }
    round_axes(&chart.base, &mut chart.y_axis_configs, percent);
    let svg = fitted(&mut chart, spec.width, spec.height, |chart| {
        let mut svg = chart.svg().ok()?;
        let exact = exact_axes(&svg, &chart.y_axis_configs, LabelEdge::Side);
        let widths = [
            y_width(&svg, &exact, AXIS_MARK, axis(&exact)),
            y_width(&svg, &exact, AXIS2_MARK, axis2(&exact)),
        ];
        if widths.iter().any(Option::is_some) {
            for (y, width) in chart.y_axis_configs.iter_mut().zip(widths) {
                y.width = width.or(y.width);
            }
            svg = chart.svg().ok()?;
        }
        Some(pin_to_axis(&svg))
    })?;
    let svg = exact_axes(&svg, &chart.y_axis_configs, LabelEdge::Side);
    Some(fit_x_labels(&svg, &x, spec.width, spec.height))
}

/// The bars' value labels placed, thinned and written by `write`, each from the value its
/// bar was drawn from.
fn bar_value_labels(
    svg: String,
    spec: &BarSpec,
    ordered: &[SeriesSpec],
    write: impl Fn(f64) -> String,
) -> String {
    let svg = if spec.horizontal || spec.labels_inside {
        svg
    } else {
        let bottom = Plot::of(&svg, spec.width, spec.height).bottom;
        place_outside_labels(&svg, bottom, spec.ink.surface)
    };
    // The renderer writes its f32s; a label writes the value its bar was drawn from.
    let values: Vec<f64> = (ordered.iter().filter(|s| !s.extras.line))
        .flat_map(|s| s.values.iter().flatten().copied())
        .collect();
    let exact = |label: f64| {
        let near = |value: &f64| (value - label).abs() <= value.abs() * 1e-6;
        values.iter().copied().find(near).unwrap_or(label)
    };
    let percent = spec.layout == BarLayout::Percent;
    let written = |label: f64| {
        if percent {
            write(label)
        } else {
            write(exact(label))
        }
    };
    let svg = drop_crowded_labels(&svg, written);
    relabel(&svg, LABEL_MARK, written)
}

/// The series' mark lines labelled off the bars and the mark point pins, from `left`.
fn bar_mark_lines(
    svg: &str,
    spec: &BarSpec,
    marked: &[Vec<&(f64, String)>],
    marks: &[Mark],
    left: f32,
) -> String {
    let texts: Vec<&str> = marked
        .iter()
        .flatten()
        .map(|(_, text)| text.as_str())
        .collect();
    let pins = pins_of(&spec.series, spec.width, marks);
    let bars: Vec<[f32; 4]> = marks
        .iter()
        .filter_map(|mark| match mark.shape {
            MarkShape::Rect {
                x,
                y,
                width,
                height,
            } => Some([x, y, x + width, y + height]),
            _ => None,
        })
        .collect();
    let svg = label_mark_lines(svg, &texts, &spec.ink, left, &pins, &bars);
    pin_mark_points(&svg, &pins, &spec.ink)
}

/// The bars as paths rounded only at the free end, the one away from the axis they grow
/// from (the top of a rising bar, the bottom of a falling one, the right end of a lying
/// one); the end at the axis and an end against another bar stay square.
pub(super) fn round_free_ends(
    svg: &str,
    triggers: &[Trigger],
    negative: &[bool],
    orientation: Orientation,
    radius: f32,
) -> String {
    let horizontal = orientation == Orientation::Horizontal;
    let bars: Vec<[f32; 4]> = triggers
        .iter()
        .filter_map(|trigger| match trigger.shape {
            MarkShape::Rect {
                x,
                y,
                width,
                height,
            } => Some([x, y, x + width, y + height]),
            _ => None,
        })
        .collect();
    let near = |a: f32, b: f32| (a - b).abs() < 0.5;
    let overlap = |a0: f32, a1: f32, b0: f32, b1: f32| a0 < b1 - 0.5 && b0 < a1 - 0.5;
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(open) = rest.find("<rect ") {
        out.push_str(&rest[..open]);
        let close = rest[open..]
            .find("/>")
            .map_or(rest.len(), |close| open + close + 2);
        let tag = &rest[open..close];
        rest = &rest[close..];
        let attributes = attributes(tag);
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        let (Some(x), Some(y), Some(width), Some(height)) =
            (number("x"), number("y"), number("width"), number("height"))
        else {
            out.push_str(tag);
            continue;
        };
        let [left, top, right, bottom] = [x, y, x + width, y + height];
        let own = [left, top, right, bottom];
        let falls = bars
            .iter()
            .position(|bar| *bar == own)
            .and_then(|i| negative.get(i))
            .copied()
            .unwrap_or(false);
        let mut touched = false;
        for &[l, t, r, b] in bars.iter().filter(|other| **other != own) {
            touched |= match (horizontal, falls) {
                (false, false) => near(b, top) && overlap(l, r, left, right),
                (false, true) => near(t, bottom) && overlap(l, r, left, right),
                (true, false) => near(l, right) && overlap(t, b, top, bottom),
                (true, true) => near(r, left) && overlap(t, b, top, bottom),
            };
        }
        let r = match touched {
            true => 0.0,
            false => radius.min(width / 2.0).min(height / 2.0).max(0.0),
        };
        let [tl, tr, br, bl] = match (horizontal, falls) {
            (false, false) => [r, r, 0.0, 0.0],
            (false, true) => [0.0, 0.0, r, r],
            (true, false) => [0.0, r, r, 0.0],
            (true, true) => [r, 0.0, 0.0, r],
        };
        let d = format!(
            "M{},{top} H{} A{tr},{tr} 0 0 1 {right},{} V{} A{br},{br} 0 0 1 {},{bottom} H{} A{bl},{bl} 0 0 1 {left},{} V{} A{tl},{tl} 0 0 1 {},{top} Z",
            left + tl,
            right - tr,
            top + tr,
            bottom - br,
            right - br,
            left + bl,
            bottom - bl,
            top + tl,
            left + tl,
        );
        let kept: String = attributes
            .iter()
            .filter(|(name, _)| !matches!(*name, "x" | "y" | "width" | "height"))
            .map(|(name, value)| format!(" {name}=\"{value}\""))
            .collect();
        let _ = write!(out, "<path d=\"{d}\"{kept}/>");
    }
    out.push_str(rest);
    out
}

/// The bars set again across their axis at the spec's width and gap, each centred in its
/// band as the renderer centres them; their labels and error bars move with their centres.
pub(super) fn relayout(
    svg: &str,
    triggers: Vec<Trigger>,
    expected: &[(usize, usize, f64, &str)],
    spec: &BarSpec,
) -> (String, Vec<Trigger>) {
    let count = spec.x.len();
    if expected.len() != triggers.len() || count == 0 {
        return (svg.to_string(), triggers);
    }
    let plot = Plot::of(svg, spec.width, spec.height);
    let (start, span) = match spec.horizontal {
        true => (plot.top, plot.bottom - plot.top),
        false => (plot.left, plot.right - plot.left),
    };
    let grouped = spec.layout == BarLayout::Grouped;
    // Lines take no slot; a stack takes one per y axis, as the renderer keys them.
    let bars: Vec<usize> = (0..spec.series.len())
        .filter(|s| !spec.series[*s].extras.line)
        .collect();
    let mut keys: Vec<usize> = Vec::new();
    for &bar in &bars {
        let key = match grouped {
            true => bar,
            false => spec.series[bar].extras.secondary as usize,
        };
        if !keys.contains(&key) {
            keys.push(key);
        }
    }
    let slot_of = |series: usize| {
        let key = match grouped {
            true => series,
            false => spec.series[series].extras.secondary as usize,
        };
        keys.iter().position(|k| *k == key).unwrap_or(0)
    };
    let slots = keys.len().max(1);
    let unit = span / count as f32;
    let group = match spec.bar_width {
        Some(share) => unit * share.clamp(0.1, 1.0),
        // A band narrower than the margins keeps half of it.
        None => (unit - 2.0 * RENDERER_MARGIN).max(unit / 2.0),
    };
    let gap = spec.bar_gap.max(0.0).min(group / slots as f32 / 2.0);
    let width = ((group - gap * (slots - 1) as f32) / slots as f32).max(1.0);
    // Old `(from, to)` across the axis to the new ones, one per drawn bar.
    let mut moves: Vec<((f32, f32), (f32, f32))> = Vec::new();
    let triggers = triggers
        .into_iter()
        .zip(expected)
        .map(|(trigger, &(series, index, ..))| {
            let MarkShape::Rect {
                x,
                y,
                width: w,
                height: h,
            } = trigger.shape
            else {
                return trigger;
            };
            // A lying chart's first category is at the bottom.
            let band = if spec.horizontal {
                count - 1 - index
            } else {
                index
            };
            let slot = slot_of(series);
            let from = start + unit * band as f32 + (unit - group) / 2.0;
            // Round trips through the SVG's text unchanged.
            let new: f32 = format!("{}", from + (width + gap) * slot as f32)
                .parse()
                .unwrap_or_default();
            let old = if spec.horizontal { (y, h) } else { (x, w) };
            moves.push((old, (new, width)));
            let shape = match spec.horizontal {
                true => MarkShape::Rect {
                    x,
                    y: new,
                    width: w,
                    height: width,
                },
                false => MarkShape::Rect {
                    x: new,
                    y,
                    width,
                    height: h,
                },
            };
            Trigger { shape, ..trigger }
        })
        .collect();

    let near = |a: f32, b: f32| (a - b).abs() < 0.15;
    let centre = |(at, size): (f32, f32)| at + size / 2.0;
    let shift = |middle: f32| {
        moves
            .iter()
            .find(|(old, _)| near(centre(*old), middle))
            .map(|(old, new)| centre(*new) - centre(*old))
    };
    let (along, size) = if spec.horizontal {
        ("y", "height")
    } else {
        ("x", "width")
    };
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(open) = rest.find('<') {
        out.push_str(&rest[..open]);
        let close = rest[open..]
            .find('>')
            .map_or(rest.len(), |close| open + close + 1);
        let tag = &rest[open..close];
        rest = &rest[close..];
        let name = tag_name(tag);
        let attributes = attributes(tag);
        let number = |name: &str| attribute(&attributes, name)?.parse::<f32>().ok();
        let replaced: Vec<(&str, f32)> = match name {
            "rect" => number(along)
                .zip(number(size))
                .and_then(|old| moves.iter().find(|(from, _)| *from == old))
                .map(|(_, (at, width))| vec![(along, *at), (size, *width)])
                .unwrap_or_default(),
            "text" => number(along)
                .and_then(|at| Some(vec![(along, at + shift(at)?)]))
                .unwrap_or_default(),
            "line" => {
                let ends = if spec.horizontal {
                    ["y1", "y2"]
                } else {
                    ["x1", "x2"]
                };
                number(ends[0])
                    .zip(number(ends[1]))
                    .and_then(|(a, b)| {
                        let by = shift((a + b) / 2.0)?;
                        Some(vec![(ends[0], a + by), (ends[1], b + by)])
                    })
                    .unwrap_or_default()
            }
            _ => Vec::new(),
        };
        if replaced.is_empty() {
            out.push_str(tag);
            continue;
        }
        let kept: String = attributes
            .iter()
            .map(
                |(attribute, value)| match replaced.iter().find(|(name, _)| name == attribute) {
                    Some((_, number)) => format!(" {attribute}=\"{number}\""),
                    None => format!(" {attribute}=\"{value}\""),
                },
            )
            .collect();
        let end = if tag.ends_with("/>") { "/>" } else { ">" };
        let _ = write!(out, "<{name}{kept}{end}");
    }
    out.push_str(rest);
    (out, triggers)
}

/// The error ranges of the bars the renderer draws none on, lying ones and stacks: each is
/// measured from the end of its own segment, at the scale of the segment's value.
pub(super) fn draw_error_bars(svg: String, marks: &[Mark], spec: &BarSpec) -> String {
    let own = spec.layout == BarLayout::Grouped && !spec.horizontal;
    if own || spec.layout == BarLayout::Percent {
        return svg;
    }
    let mut lines = String::new();
    for mark in marks {
        let range = spec.errors.get(mark.series).and_then(|e| e.get(mark.index));
        let (
            Some(Some((low, high))),
            MarkShape::Rect {
                x,
                y,
                width,
                height,
            },
        ) = (range, &mark.shape)
        else {
            continue;
        };
        let (x, y, width, height) = (*x, *y, *width, *height);
        let value = mark.value;
        if spec.series[mark.series].extras.line || value == 0.0 {
            continue;
        }
        let line = |[x1, y1, x2, y2]: [f32; 4]| {
            format!(
                "<line stroke-width=\"1.5\" x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" stroke=\"{}\"/>",
                spec.ink.label
            )
        };
        // Lying, the value grows to the right; upright, up the page.
        let (across, thickness, length, reach, sign) = match spec.horizontal {
            true => (y + height / 2.0, height, width, [x, x + width], 1.0),
            false => (x + width / 2.0, width, height, [y + height, y], -1.0),
        };
        let reach = reach[(value > 0.0) as usize];
        let scale = length / value.abs() as f32;
        let [from, to] = [*low, *high].map(|end| reach + sign * (end - value) as f32 * scale);
        let cap = (thickness / 4.0).clamp(2.0, 6.0);
        let [stem, a, b] = match spec.horizontal {
            true => [
                [from, across, to, across],
                [from, across - cap, from, across + cap],
                [to, across - cap, to, across + cap],
            ],
            false => [
                [across, from, across, to],
                [across - cap, from, across + cap, from],
                [across - cap, to, across + cap, to],
            ],
        };
        lines.extend([line(stem), line(a), line(b)]);
    }
    match svg.rfind("</svg>") {
        Some(end) if !lines.is_empty() => format!("{}{lines}{}", &svg[..end], &svg[end..]),
        _ => svg,
    }
}

/// Gives every label drawn on a bar the ink that reads on the bar nearest to it.
pub(super) fn ink_bar_labels(svg: &str, marks: &[Mark], ink: impl Fn(&Mark) -> HexColor) -> String {
    let distance = |mark: &Mark, (x, y): (f32, f32)| match mark.shape {
        MarkShape::Rect {
            x: left,
            y: top,
            width,
            height,
        } => {
            let dx = (left - x).max(x - left - width).max(0.0);
            let dy = (top - y).max(y - top - height).max(0.0);
            dx.hypot(dy)
        }
        _ => f32::INFINITY,
    };
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(start) = rest.find("<text ") {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find('>')
            .map_or(rest.len(), |end| start + end + 1);
        let tag = &rest[start..end];
        let attributes = attributes(tag);
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        let nearest = number("x")
            .zip(number("y"))
            .filter(|_| tag.contains(ON_BAR_FILL))
            .and_then(|point| {
                marks
                    .iter()
                    .min_by(|a, b| distance(a, point).total_cmp(&distance(b, point)))
            });
        match nearest {
            Some(mark) => {
                out.push_str(&tag.replace(ON_BAR_FILL, &format!("fill=\"{}\"", ink(mark))))
            }
            None => out.push_str(tag),
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}
