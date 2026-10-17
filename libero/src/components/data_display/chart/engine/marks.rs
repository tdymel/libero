//! A series' mark lines and mark point pins, over a line or a vertical bar chart.

use super::*;
use std::fmt::Write as _;

/// Every series' mark lines inside its value axis' ends, where unset the data's extent and
/// 0, which the axis always spans; in the order the renderer draws them.
pub(super) fn kept_mark_lines<'a>(
    series: &'a [SeriesSpec],
    axes: &Axes,
) -> Vec<Vec<&'a (f64, String)>> {
    let ends = |secondary: bool| {
        let values = series
            .iter()
            .filter(|s| s.extras.secondary == secondary)
            .flat_map(|s| s.values.iter().flatten().copied());
        let fixed = |end: Option<f64>| end.filter(|_| !secondary);
        let low = fixed(axes.min).unwrap_or(values.clone().fold(0.0, f64::min));
        (low, fixed(axes.max).unwrap_or(values.fold(0.0, f64::max)))
    };
    let axis = [ends(false), ends(true)];
    series
        .iter()
        .map(|series| {
            let (low, high) = axis[series.extras.secondary as usize];
            let lines = series.extras.mark_lines.iter();
            lines
                .filter(|(value, _)| (low..=high).contains(value))
                .collect()
        })
        .collect()
}

pub(super) fn renderer_mark_line(&(value, _): &(f64, String)) -> charts_rs::MarkLine {
    charts_rs::MarkLine {
        category: MarkLineCategory::Value(value as f32),
        ..Default::default()
    }
}

/// Writes each mark line's own text, in drawing order, over the bare number the renderer
/// puts by it: in the label ink, haloed in the surface's so the data under it cannot blur it.
/// One wider than the plot from its `left` is left out; the summary keeps it. One over a
/// value label or a pin goes under its line instead, if that is free.
pub(super) fn label_mark_lines(
    svg: &str,
    texts: &[&str],
    ink: &ChartInk,
    left: f32,
    pins: &[MarkPin],
    bars: &[[f32; 4]],
) -> String {
    let is_mark = |tag: &str| tag.contains("dy=\"-6\"") && tag.contains("text-anchor=\"end\"");
    // Covering a text or a pin costs ten times covering a bar.
    let mut crowd: Vec<([f32; 4], f32)> = pins.iter().map(|pin| (pin.rect(), 10.0)).collect();
    crowd.extend(bars.iter().map(|bar| (*bar, 1.0)));
    let mut scan = svg;
    while let Some(open) = scan.find("<text ") {
        let Some(close) = scan[open..].find('>').map(|close| open + close) else {
            break;
        };
        let tag = &scan[open..=close];
        let label = &scan[close + 1..];
        let label = label[..label.find('<').unwrap_or(label.len())].trim();
        scan = &scan[close + 1..];
        let attributes = attributes(tag);
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        if let (Some(x), Some(y)) = (number("x"), number("y"))
            && !label.is_empty()
            && !is_mark(tag)
        {
            crowd.push((text_box(&attributes, label, x, y), 10.0));
        }
    }
    let mut texts = texts.iter();
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(start) = rest.find("<text ") {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find("</text>")
            .map_or(rest.len(), |end| start + end + "</text>".len());
        let element = &rest[start..end];
        rest = &rest[end..];
        let tag = &element[..element.find('>').map_or(element.len(), |i| i + 1)];
        let Some(text) = is_mark(tag).then(|| texts.next()).flatten() else {
            out.push_str(element);
            continue;
        };
        let attributes = attributes(tag);
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        let end = number("x");
        if end.is_some_and(|end| end - text_width(&attributes, text) < left) {
            continue;
        }
        // Where the renderer has it, above its line, or under it, or slid left along it:
        // whichever covers the least, moving as little as it can.
        let size = number("font-size").unwrap_or(14.0);
        let (above, below) = (-6.0, 6.0 + size * 0.8);
        let wide = text_width(&attributes, text);
        let (dy, shift) = match (end, number("y")) {
            (Some(x), Some(y)) => {
                let cost = |dy: f32, shift: f32| {
                    let (l, t) = (x - shift - wide, y + dy - size * 0.8);
                    let (r, b) = (x - shift, y + dy + size * 0.25);
                    let covered: f32 = crowd
                        .iter()
                        .map(|(c, weight)| {
                            (r.min(c[2]) - l.max(c[0])).max(0.0)
                                * (b.min(c[3]) - t.max(c[1])).max(0.0)
                                * weight
                        })
                        .sum();
                    covered + shift + if dy == above { 0.0 } else { 20.0 }
                };
                let slide = (0..=((x - wide - left).max(0.0) / 4.0) as u32).map(|n| n as f32 * 4.0);
                slide
                    .flat_map(|shift| [(above, shift), (below, shift)])
                    .min_by(|a, b| cost(a.0, a.1).total_cmp(&cost(b.0, b.1)))
                    .unwrap_or((above, 0.0))
            }
            _ => (above, 0.0),
        };
        let kept: String = attributes
            .iter()
            .map(|&(name, value)| match name {
                "fill" => format!(" fill=\"{}\"", ink.label),
                "dy" => format!(" dy=\"{dy}\""),
                "x" => format!(
                    " x=\"{}\"",
                    value
                        .parse::<f32>()
                        .map_or(value.to_string(), |x| (x - shift).to_string())
                ),
                _ => format!(" {name}=\"{value}\""),
            })
            .collect();
        let halo = format!(
            " stroke=\"{}\" stroke-width=\"3\" stroke-linejoin=\"round\" paint-order=\"stroke\"",
            ink.surface
        );
        let _ = write!(out, "<text{kept}{halo}>\n{}\n</text>", escape(text));
    }
    out.push_str(rest);
    out
}

pub(super) const PIN_HEIGHT: f32 = 20.0;
pub(super) const PIN_GAP: f32 = 6.0;
pub(super) const PIN_SIZE: f32 = 12.0;

/// A marked point's pill: its series' colour, its text and its place.
pub(super) struct MarkPin<'a> {
    color: HexColor,
    text: &'a str,
    left: f32,
    top: f32,
    width: f32,
}

impl MarkPin<'_> {
    fn rect(&self) -> [f32; 4] {
        [
            self.left,
            self.top,
            self.left + self.width,
            self.top + PIN_HEIGHT,
        ]
    }
}

/// Each marked point's pill, above the point or, with no room there, below it.
pub(super) fn pins_of<'a>(
    all: &'a [SeriesSpec],
    chart_width: f32,
    marks: &[Mark],
) -> Vec<MarkPin<'a>> {
    let mut pins = Vec::new();
    for (s, series) in all.iter().enumerate() {
        for (index, text) in &series.extras.mark_points {
            let place = marks.iter().find(|m| m.series == s && m.index == *index);
            let Some(MarkShape::Point { x, y }) = place.map(|mark| &mark.shape) else {
                continue;
            };
            let width = charts_rs::measure_text_width_family(DEFAULT_FONT_FAMILY, PIN_SIZE, text)
                .map_or(0.0, |text| text.width())
                + PIN_HEIGHT * 0.6;
            let left = (x - width / 2.0).clamp(1.0, (chart_width - width - 1.0).max(1.0));
            let top = match y - PIN_GAP - PIN_HEIGHT >= 1.0 {
                true => y - PIN_GAP - PIN_HEIGHT,
                false => y + PIN_GAP,
            };
            pins.push(MarkPin {
                color: series.color,
                text,
                left,
                top,
                width,
            });
        }
    }
    pins
}

/// Each marked point's value in a pill of its series' colour, above the point or, with no
/// room there, below it; in black or white, whichever reads on the colour.
pub(super) fn pin_mark_points(svg: &str, pins: &[MarkPin], ink: &ChartInk) -> String {
    let pins: String = pins
        .iter()
        .map(|pin| {
            format!(
                "<g class=\"ct-pin\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{PIN_HEIGHT}\" \
                 rx=\"{}\" fill=\"{}\"/><text x=\"{}\" y=\"{}\" font-size=\"{PIN_SIZE}\" \
                 font-family=\"inherit\" text-anchor=\"middle\" dominant-baseline=\"central\" \
                 fill=\"{}\">{}</text></g>",
                pin.left,
                pin.top,
                pin.width,
                PIN_HEIGHT / 2.0,
                pin.color,
                pin.left + pin.width / 2.0,
                pin.top + PIN_HEIGHT / 2.0,
                ink.on(pin.color),
                escape(pin.text),
            )
        })
        .collect();
    match svg.rfind("</svg>") {
        Some(end) => format!("{}{pins}{}", &svg[..end], &svg[end..]),
        None => svg.to_string(),
    }
}
