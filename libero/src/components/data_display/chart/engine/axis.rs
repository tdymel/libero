//! The value axes: their ends and steps rounded, their tick labels written and pinned.

use super::*;
use std::fmt::Write as _;

/// A percent stack with losses reads each share of the stack's absolute sum, up above 0 and
/// down below it: the axis runs to the furthest of each, in `ticks` round steps (else the
/// renderer's).
pub(super) fn percent_axes(axes: &Axes, series: &[SeriesSpec]) -> Axes {
    let bars: Vec<_> = series.iter().filter(|s| !s.extras.line).collect();
    let (mut up, mut down) = (0.0_f64, 0.0_f64);
    for index in 0..bars.iter().map(|s| s.values.len()).max().unwrap_or(0) {
        let values = bars
            .iter()
            .filter_map(|s| s.values.get(index).copied().flatten());
        let (gain, loss) = values.fold((0.0, 0.0), |(gain, loss), v: f64| {
            (gain + v.max(0.0), loss + (-v).max(0.0))
        });
        let total = gain + loss;
        if total > 0.0 {
            up = up.max(gain / total * 100.0);
            down = down.max(loss / total * 100.0);
        }
    }
    if down == 0.0 {
        return axes.clone();
    }
    round_ends(axes, -down, up)
}

/// Puts axis `index`'s unset ends on whole steps of 1, 2, 2.5 or 5 times a power of ten, the
/// step count nearest its `split_number`, or exactly `count` steps; set ends stay. With
/// `zero`, the axis spans 0 too.
pub(super) fn round_axis(
    list: &[charts_rs::Series],
    index: usize,
    config: &mut charts_rs::YAxisConfig,
    count: Option<usize>,
    zero: bool,
) {
    if config.min.is_some() && config.max.is_some() {
        return;
    }
    // What the renderer spans: each value, each band's ends and each stack's sums.
    let mut values = Vec::new();
    let mut stacks = std::collections::BTreeMap::<(&str, usize), (f64, f64)>::new();
    for series in list.iter().filter(|s| s.y_axis_index == index) {
        for (i, value) in series.data.iter().enumerate() {
            let Some(value) = value.filter(|v| v.is_finite()).map(f64::from) else {
                continue;
            };
            match &series.stack {
                None => values.push(value),
                Some(key) => {
                    let sums = stacks.entry((key, i + series.start_index)).or_default();
                    match value >= 0.0 {
                        true => sums.0 += value,
                        false => sums.1 += value,
                    }
                }
            }
        }
        for bounds in [&series.band, &series.error_bar].into_iter().flatten() {
            let ends = bounds.lower.iter().chain(&bounds.upper).flatten();
            values.extend(ends.filter(|v| v.is_finite()).map(|v| f64::from(*v)));
        }
    }
    values.extend(stacks.into_values().flat_map(|(up, down)| [up, down]));
    if values.is_empty() {
        return;
    }
    let (min, max) = (config.min.map(f64::from), config.max.map(f64::from));
    let start = if zero { 0.0 } else { values[0] };
    let low = min.unwrap_or(values.iter().fold(start, |low, v| v.min(low)));
    let high = max.unwrap_or(values.iter().fold(start, |high, v| v.max(high)));
    // Only zeros: an axis up to 1.
    let high = match (high > low, max) {
        (false, None) => low + 1.0,
        _ => high,
    };
    if high.partial_cmp(&low) != Some(std::cmp::Ordering::Greater) {
        return;
    }
    let target = count.unwrap_or(config.split_number).max(1);
    let ends = |step: f64| {
        let steps = |span: f64| (span / step - 1e-9).ceil() * step;
        match (min, max) {
            (Some(min), _) => (min, min + steps(high - min)),
            (_, Some(max)) => (max - steps(max - low), max),
            _ => (
                (low / step + 1e-9).floor() * step,
                (high / step - 1e-9).ceil() * step,
            ),
        }
    };
    let steps = |step: f64| {
        let (low, high) = ends(step);
        ((high - low) / step).round() as usize
    };
    let power = 10_f64.powf(((high - low) / target as f64).log10().floor());
    let candidates = [0.1, 1.0, 10.0]
        .into_iter()
        .flat_map(|scale| [1.0, 2.0, 2.5, 5.0].map(|m| m * scale * power));
    // Exactly `count`: the finest step that fits. Else the nearest count, then the tightest ends.
    let step = match count {
        Some(count) => candidates
            .filter(|step| steps(*step) <= count)
            .reduce(f64::min),
        None => candidates.min_by(|a, b| {
            let off = |step: f64| steps(step).abs_diff(target);
            let span = |step: f64| ends(step).1 - ends(step).0;
            let tighter = span(*a).total_cmp(&span(*b));
            off(*a).cmp(&off(*b)).then(tighter).then(b.total_cmp(a))
        }),
    };
    let Some(step) = step else { return };
    let (low, _) = ends(step);
    let n = count.unwrap_or(steps(step)).max(1);
    let high = match (min, max) {
        (_, Some(max)) => max,
        _ => low + step * n as f64,
    };
    let low = match max {
        Some(max) => max - step * n as f64,
        None => low,
    };
    config.min = Some(low as f32);
    config.max = Some(high as f32);
    config.split_number = n;
}

/// [`round_axis`] on the left axis, and the right one cut into as many steps. With `percent`
/// the left axis stays: it reads shares.
pub(super) fn round_axes(
    base: &charts_rs::ChartBase,
    y: &mut [charts_rs::YAxisConfig],
    percent: bool,
) {
    if let Some(left) = y.first_mut().filter(|_| !percent) {
        round_axis(&base.series_list, 0, left, None, true);
    }
    let count = y.first().map(|left| left.split_number);
    if let Some(right) = y.get_mut(1) {
        round_axis(&base.series_list, 1, right, count, true);
    }
}

/// Where an axis' labels run: along the bottom, or up the side.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum LabelEdge {
    Bottom,
    Side,
}

/// [`exact_ticks`] on both value axes; the right one always runs up the side.
pub(super) fn exact_axes(svg: &str, y: &[charts_rs::YAxisConfig], edge: LabelEdge) -> String {
    let svg = match y.first() {
        Some(left) => exact_ticks(svg, AXIS_MARK, left, edge),
        None => svg.to_string(),
    };
    match y.get(1) {
        Some(right) => exact_ticks(&svg, AXIS2_MARK, right, LabelEdge::Side),
        None => svg,
    }
}

/// Writes axis `mark`'s labels in full, as `config` cuts it: the renderer keeps one decimal.
pub(super) fn exact_ticks(
    svg: &str,
    mark: char,
    config: &charts_rs::YAxisConfig,
    edge: LabelEdge,
) -> String {
    let (Some(min), Some(max)) = (config.min, config.max) else {
        return svg.to_string();
    };
    let n = config.split_number.max(1);
    let step = (f64::from(max) - f64::from(min)) / n as f64;
    let mut at = Vec::new();
    let mut rest = 0;
    while let Some(open) = svg[rest..].find("<text").map(|open| rest + open) {
        let Some(close) = svg[open..].find('>').map(|close| open + close) else {
            break;
        };
        let attributes = attributes(&svg[open..=close]);
        let label = &svg[close + 1..];
        let label = label[..label.find('<').unwrap_or(label.len())].trim_start();
        if label.starts_with(mark) {
            let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
            let place = number(match edge {
                LabelEdge::Bottom => "x",
                LabelEdge::Side => "y",
            })
            .unwrap_or(0.0);
            let start = svg.len() - svg[close + 1..].trim_start().len() + mark.len_utf8();
            let length = svg[start..].find([' ', '\n', '<']).unwrap_or(0);
            at.push((place, start, length));
        }
        rest = close + 1;
    }
    if at.len() != n + 1 {
        return svg.to_string();
    }
    // Lowest first: leftmost along the bottom, lowest up the side.
    at.sort_by(|a, b| match edge {
        LabelEdge::Bottom => a.0.total_cmp(&b.0),
        LabelEdge::Side => b.0.total_cmp(&a.0),
    });
    let mut values: Vec<_> = at
        .iter()
        .enumerate()
        .map(|(i, &(_, start, length))| (start, length, f64::from(min) + step * i as f64))
        .collect();
    values.sort_by_key(|value| value.0);
    let mut out = String::with_capacity(svg.len());
    let mut last = 0;
    for (start, length, value) in values {
        out.push_str(&svg[last..start]);
        let _ = write!(out, "{value}");
        last = start + length;
    }
    out.push_str(&svg[last..]);
    out
}

/// The value axis of bars whose ranges the renderer does not know of, stacks and lying bars:
/// unset ends reach the furthest range end, on whole steps of a round size.
pub(super) fn error_axes(axes: &Axes, spec: &BarSpec) -> Axes {
    let stacked = spec.layout != BarLayout::Grouped;
    let (mut low, mut high) = (0.0_f64, 0.0_f64);
    for index in 0..spec.x.len() {
        let (mut up, mut down) = (0.0_f64, 0.0_f64);
        for (s, series) in spec.series.iter().enumerate() {
            let value = series.values.get(index).copied().flatten();
            let Some(value) = value.filter(|_| !series.extras.line && !series.extras.secondary)
            else {
                continue;
            };
            let base = match (stacked, value >= 0.0) {
                (false, _) => 0.0,
                (true, true) => up,
                (true, false) => down,
            };
            let range = spec
                .errors
                .get(s)
                .and_then(|e| e.get(index).copied().flatten());
            for end in [value]
                .into_iter()
                .chain(range.into_iter().flat_map(|r| [r.0, r.1]))
            {
                low = low.min(base + end);
                high = high.max(base + end);
            }
            match value >= 0.0 {
                true => up += value,
                false => down += value,
            }
        }
    }
    round_ends(axes, low, high)
}

/// `axes` with unset ends reaching `low` and `high` on whole steps of a round size, at most
/// `ticks` of them.
pub(super) fn round_ends(axes: &Axes, low: f64, high: f64) -> Axes {
    let raw = (high - low) / axes.ticks.max(1) as f64;
    if axes.min.is_some() || axes.max.is_some() || raw <= 0.0 || !raw.is_finite() {
        return axes.clone();
    }
    let power = 10_f64.powf(raw.log10().floor());
    let step = power
        * [1.0, 2.0, 2.5, 5.0, 10.0]
            .into_iter()
            .find(|n| raw / power <= *n)
            .unwrap_or(10.0);
    let (min, max) = ((low / step).floor() * step, (high / step).ceil() * step);
    Axes {
        min: Some(min),
        max: Some(max),
        ticks: ((max - min) / step).round().max(1.0) as usize,
        ..axes.clone()
    }
}

/// How an axis writes its ticks: under `ValueFormat::Auto` (`auto`) every tick at the one
/// precision its step needs and, past seven digits, in the `compact` million, billion or
/// trillion; otherwise as `plain` writes them.
pub(super) fn axis_write<'a, F: Fn(f64) -> String + 'a>(
    svg: &str,
    mark: char,
    auto: bool,
    unit: Option<&'a str>,
    formats: &'a Formats,
    compact: &'a [&'static str; 3],
    plain: F,
) -> impl Fn(f64) -> String + use<'a, F> {
    let ticks = auto.then(|| Ticks::of(svg, mark, compact));
    move |value| match ticks {
        Some(ticks) => ticks.write(value, unit, formats),
        None => plain(value),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Ticks {
    decimals: usize,
    scale: f64,
    suffix: &'static str,
}

impl Ticks {
    fn of(svg: &str, mark: char, compact: &[&'static str; 3]) -> Self {
        let mut values = Vec::new();
        let mut rest = svg;
        while let Some(at) = rest.find(mark) {
            let label = &rest[at + mark.len_utf8()..];
            let end = label.find(['\n', '<', ' ']).unwrap_or(label.len());
            values.extend(scaled(label[..end].trim_end_matches('%')));
            rest = &label[end..];
        }
        let largest = values.iter().fold(0.0_f64, |most, v| most.max(v.abs()));
        let (scale, suffix) = match largest {
            l if l >= 1e12 => (1e12, compact[2]),
            l if l >= 1e9 => (1e9, compact[1]),
            l if l >= 1e7 => (1e6, compact[0]),
            _ => (1.0, ""),
        };
        let (low, high) = values.iter().fold((f64::MAX, f64::MIN), |(low, high), v| {
            (low.min(*v), high.max(*v))
        });
        let step = (high - low) / scale / values.len().saturating_sub(1).max(1) as f64;
        let decimals = match step > 0.0 && step.is_finite() {
            // The step's first digit, and one more for a step like 2.5 or 0.25.
            true => (0..=15)
                .find(|d| {
                    let shifted = step * 10_f64.powi(*d);
                    shifted >= 1.0 && (shifted - shifted.round()).abs() < shifted * 1e-4
                })
                .unwrap_or(15) as usize,
            false => 0,
        };
        // A step that never ends (66.67) stops at two decimals, as `ValueFormat::Auto` does:
        // three read `250,000` as a quarter million where the comma groups.
        let decimals = if step >= 1.0 {
            decimals.min(2)
        } else {
            decimals
        };
        Self {
            decimals,
            scale,
            suffix,
        }
    }

    fn write(&self, value: f64, unit: Option<&str>, formats: &Formats) -> String {
        let shown = format!("{:.*}", self.decimals, value / self.scale);
        // f32 noise round a zero tick must not read "-0".
        let shown = match shown
            .trim_start_matches('-')
            .chars()
            .all(|c| matches!(c, '0' | '.'))
        {
            true => shown.trim_start_matches('-').to_string(),
            false => shown,
        };
        // 0 needs no million.
        let suffix = match value == 0.0 {
            true => "",
            false => self.suffix,
        };
        let number = format!("{}{suffix}", formats.write_number(&shown));
        match unit.filter(|unit| !unit.is_empty()) {
            Some(unit) => format!("{number} {unit}"),
            None => number,
        }
    }
}

/// Writes every label behind `mark` again with `axis`.
pub(super) fn relabel(svg: &str, mark: char, axis: impl Fn(f64) -> String) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(at) = rest.find(mark) {
        out.push_str(&rest[..at]);
        let label = &rest[at + mark.len_utf8()..];
        let end = label.find(['\n', '<']).unwrap_or(label.len());
        let number = label[..end].split(' ').next().unwrap_or_default();
        match scaled(number.trim_end_matches('%')) {
            Some(value) => out.push_str(&escape(&axis(value))),
            None => out.push_str(&label[..end]),
        }
        rest = &label[end..];
    }
    out.push_str(rest);
    out
}

/// The renderer places a y label by its own text's width; written again in the locale it
/// would leave the axis. Pins each one by the right end the renderer gave it.
pub(super) fn pin_to_axis(svg: &str) -> String {
    pin_labels(svg, AXIS_MARK, Pin::<fn(f64) -> String>::End)
}

/// A lying bar chart's value labels, or a numeric x axis' behind `mark`, sit under their
/// ticks: pinned by their centre, as `axis` writes them, and kept inside the `width`.
pub(super) fn pin_to_ticks(
    svg: &str,
    mark: char,
    width: f32,
    axis: impl Fn(f64) -> String,
) -> String {
    pin_labels(svg, mark, Pin::Middle { width, axis })
}

/// Drops the tick labels behind `mark`, written by `axis`, that touch the one before; the
/// last tick stays and takes the place of the ones it touches.
pub(super) fn thin_tick_labels(svg: &str, mark: char, axis: impl Fn(f64) -> String) -> String {
    // Each label's element range and its span along x.
    let mut labels: Vec<(std::ops::Range<usize>, f32, f32)> = Vec::new();
    let mut at = 0;
    while let Some(open) = svg[at..].find("<text").map(|open| at + open) {
        let Some(close) = svg[open..].find('>').map(|close| open + close) else {
            break;
        };
        let end = svg[close..]
            .find("</text>")
            .map_or(close + 1, |end| close + end + 7);
        at = end;
        let attributes = attributes(&svg[open..=close]);
        let content = svg[close + 1..end].trim_end_matches("</text>").trim();
        let x = attribute(&attributes, "x").and_then(|x| x.parse::<f32>().ok());
        let (Some(x), Some(number)) = (x, content.strip_prefix(mark)) else {
            continue;
        };
        let written = match number.split(' ').next().and_then(scaled) {
            Some(value) => axis(value),
            None => number.to_string(),
        };
        let width = roomy(&attributes, &written);
        let left = match attribute(&attributes, "text-anchor") {
            Some("middle") => x - width / 2.0,
            Some("end") => x - width,
            _ => x,
        };
        labels.push((open..end, left, left + width));
    }
    let mut kept: Vec<usize> = Vec::new();
    for (i, (_, left, _)) in labels.iter().enumerate() {
        let touches = |last: &usize| *left < labels[*last].2 + EDGE;
        if i + 1 == labels.len() {
            while kept.last().is_some_and(touches) {
                kept.pop();
            }
        } else if kept.last().is_some_and(touches) {
            continue;
        }
        kept.push(i);
    }
    let mut out = String::with_capacity(svg.len());
    let mut from = 0;
    for (i, (range, _, _)) in labels.iter().enumerate() {
        if !kept.contains(&i) {
            out.push_str(&svg[from..range.start]);
            from = range.end;
        }
    }
    out.push_str(&svg[from..]);
    out
}

pub(super) enum Pin<F> {
    End,
    Middle { width: f32, axis: F },
}

pub(super) fn pin_labels<F: Fn(f64) -> String>(svg: &str, mark: char, pin: Pin<F>) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        let Some(close) = rest[open..].find('>').map(|close| open + close) else {
            break;
        };
        out.push_str(&rest[..open]);
        let tag = &rest[open..=close];
        let label = &rest[close + 1..];
        let label = label[..label.find('<').unwrap_or(label.len())].trim();
        let attributes = attributes(tag);
        let x = attribute(&attributes, "x").and_then(|x| x.parse::<f32>().ok());
        match x {
            Some(x) if label.starts_with(mark) => {
                let drawn = text_width(&attributes, label);
                let (x, anchor) = match &pin {
                    Pin::End => (x + drawn, "end"),
                    Pin::Middle { width, axis } => {
                        let number = label[mark.len_utf8()..].split(' ').next();
                        let half = number
                            .and_then(scaled)
                            .map_or(drawn, |value| text_width(&attributes, &axis(value)))
                            / 2.0;
                        let centre = x + drawn / 2.0;
                        (centre.min(width - EDGE - half).max(EDGE + half), "middle")
                    }
                };
                let kept: String = attributes
                    .iter()
                    .filter(|(attribute, _)| *attribute != "x")
                    .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
                    .collect();
                let _ = write!(out, "<text x=\"{x}\" text-anchor=\"{anchor}\"{kept}>");
            }
            _ => out.push_str(tag),
        }
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    out
}

/// The most `width` gives a label behind `mark`, handed its tag's attributes, its text and
/// its number.
pub(super) fn widest(
    svg: &str,
    mark: char,
    width: impl Fn(&[(&str, &str)], &str, &str) -> f32,
) -> Option<f32> {
    let mut most = 0.0f32;
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        let close = open + rest[open..].find('>')?;
        let attributes = attributes(&rest[open..=close]);
        let label = &rest[close + 1..];
        let label = label[..label.find('<').unwrap_or(label.len())].trim();
        if let Some(number) = label.strip_prefix(mark) {
            let number = number.split(' ').next().unwrap_or_default();
            most = most.max(width(&attributes, label, number));
        }
        rest = &rest[close + 1..];
    }
    Some(most)
}

/// The y axis' width once `axis` writes its labels, read in full from `exact`, when that is
/// more than the renderer kept for them in `svg`.
pub(super) fn y_width(
    svg: &str,
    exact: &str,
    mark: char,
    axis: impl Fn(f64) -> String,
) -> Option<f32> {
    let kept = widest(svg, mark, |attributes, label, _| {
        text_width(attributes, label)
    })?;
    let needed = widest(exact, mark, |attributes, _, number| {
        scaled(number).map_or(0.0, |value| text_width(attributes, &axis(value)))
    })?;
    // The renderer's own room: the label's width and 5.
    (needed > kept).then_some(needed + 5.0)
}
