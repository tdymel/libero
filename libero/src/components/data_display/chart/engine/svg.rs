//! Text surgery on the renderer's SVG: its tags and attributes read, made inert, escaped.

use super::*;

/// `extra` drawn last, the viewBox grown to the full `width` and `height`.
pub(super) fn appended(svg: &str, width: f32, height: f32, extra: &str) -> String {
    let svg = svg.trim_end().trim_end_matches("</svg>");
    let svg = match svg.find("viewBox=\"") {
        Some(at) => {
            let start = at + "viewBox=\"".len();
            let end = svg[start..].find('"').map_or(start, |end| start + end);
            format!("{}0 0 {width} {height}{}", &svg[..start], &svg[end..])
        }
        None => svg.to_string(),
    };
    format!("{svg}{extra}</svg>")
}

/// One of the renderer's hover targets: a mark's shape and its series' name.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Trigger {
    pub(crate) series: String,
    pub(crate) shape: MarkShape,
}

/// The renderer's SVG made inert: no own tooltips, no `<title>`s, the page's font, its
/// y labels written by `axis`, sized by its container. Returns the triggers it dropped.
pub(crate) fn sanitize(svg: &str, axis: impl Fn(f64) -> String) -> (String, Vec<Trigger>) {
    inert(svg, axis, "", false)
}

/// [`sanitize`], with `extra` attributes on every kept mark; with `circles`, a circle trigger
/// is a scatter's point, kept, not a line's invisible target.
pub(super) fn inert(
    svg: &str,
    axis: impl Fn(f64) -> String,
    extra: &str,
    circles: bool,
) -> (String, Vec<Trigger>) {
    let mut out = String::with_capacity(svg.len());
    let mut triggers = Vec::new();
    let mut rest = svg;
    while let Some(open) = rest.find('<') {
        out.push_str(&rest[..open]);
        let Some(close) = rest[open..].find('>').map(|close| open + close) else {
            rest = &rest[open..];
            break;
        };
        let tag = &rest[open..=close];
        let name = tag_name(tag);
        let attributes = attributes(tag);
        let class = attribute(&attributes, "class");
        let end_of = |rest: &str| {
            let closing = format!("</{name}>");
            rest[close..]
                .find(&closing)
                .map(|end| close + end + closing.len())
                .unwrap_or(close + 1)
        };
        if name == "style" || class == Some("ct-tip") {
            rest = &rest[end_of(rest)..];
            continue;
        }
        if class == Some("ct-trigger") {
            if let Some(trigger) = trigger(name, &attributes) {
                triggers.push(trigger);
            }
            let end = end_of(rest);
            // A line's trigger is an invisible circle; a bar or a slice is the mark itself.
            if name != "circle" || circles {
                out.push_str(&self_closing(name, &attributes, extra));
            }
            rest = &rest[end..];
            continue;
        }
        if name == "svg" {
            out.push_str(&root(&attributes));
        } else {
            out.push_str(tag);
        }
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    let out = out.replace("font-family=\"Roboto\"", "font-family=\"inherit\"");
    (relabel(&out, AXIS_MARK, axis), triggers)
}

pub(super) fn tag_name(tag: &str) -> &str {
    let inner = tag.trim_start_matches('<');
    let end = inner
        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .unwrap_or(inner.len());
    &inner[..end]
}

/// A tag's attributes, values still escaped.
pub(super) fn attributes(tag: &str) -> Vec<(&str, &str)> {
    let mut found = Vec::new();
    let mut rest = &tag[1 + tag_name(tag).len()..];
    while let Some(eq) = rest.find("=\"") {
        let name = rest[..eq].trim();
        let after = &rest[eq + 2..];
        let Some(end) = after.find('"') else { break };
        found.push((name, &after[..end]));
        rest = &after[end + 1..];
    }
    found
}

pub(super) fn attribute<'a>(attributes: &[(&str, &'a str)], name: &str) -> Option<&'a str> {
    attributes
        .iter()
        .find(|(attribute, _)| *attribute == name)
        .map(|(_, value)| *value)
}

pub(super) fn trigger(name: &str, attributes: &[(&str, &str)]) -> Option<Trigger> {
    let number = |name| attribute(attributes, name)?.parse::<f32>().ok();
    let shape = match name {
        "circle" => MarkShape::Point {
            x: number("cx")?,
            y: number("cy")?,
        },
        "rect" => MarkShape::Rect {
            x: number("x")?,
            y: number("y")?,
            width: number("width")?,
            height: number("height")?,
        },
        "path" => MarkShape::Sector {
            path: attribute(attributes, "d")?.to_string(),
        },
        // A funnel's stage or a scatter's triangle or diamond, as the path of its corners.
        "polygon" => {
            let corners: Vec<&str> = attribute(attributes, "points")?
                .split_whitespace()
                .collect();
            let first = corners.first()?.replace(',', " ");
            let rest: String = corners[1..]
                .iter()
                .map(|corner| format!(" L {}", corner.replace(',', " ")))
                .collect();
            MarkShape::Sector {
                path: format!("M {first}{rest} Z"),
            }
        }
        _ => return None,
    };
    // A heatmap's cell names its row instead.
    let series = attribute(attributes, "data-series").or(attribute(attributes, "data-y"))?;
    Some(Trigger {
        series: unescape(series),
        shape,
    })
}

/// A mark without the renderer's hover hooks.
pub(super) fn self_closing(name: &str, attributes: &[(&str, &str)], extra: &str) -> String {
    let kept: String = attributes
        .iter()
        .filter(|(attribute, _)| *attribute != "class" && !attribute.starts_with("data-"))
        .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
        .collect();
    format!("<{name}{kept}{extra}/>")
}

/// The root at its container's size, out of the accessibility tree and the tab order.
pub(super) fn root(attributes: &[(&str, &str)]) -> String {
    let kept: String = attributes
        .iter()
        .filter(|(attribute, _)| !matches!(*attribute, "width" | "height"))
        .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
        .collect();
    format!("<svg width=\"100%\" height=\"100%\" aria-hidden=\"true\" focusable=\"false\"{kept}>")
}

/// Draws `chart`, then once more with a wider margin if a label left the SVG.
pub(super) fn fitted<C: std::ops::DerefMut<Target = charts_rs::ChartBase>>(
    chart: &mut C,
    width: f32,
    height: f32,
    draw: impl Fn(&mut C) -> Option<String>,
) -> Option<String> {
    let svg = draw(chart)?;
    let Some([left, top, right, bottom]) = overflow(&svg, width, height) else {
        return Some(svg);
    };
    chart.margin.left += left;
    chart.margin.top += top;
    chart.margin.right += right;
    chart.margin.bottom += bottom;
    draw(chart)
}

/// How far the texts reach past [`EDGE`] on each side, `[left, top, right, bottom]`;
/// `None` when they all keep inside.
pub(super) fn overflow(svg: &str, width: f32, height: f32) -> Option<[f32; 4]> {
    let mut over = [0.0f32; 4];
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        let Some(close) = rest[open..].find('>').map(|close| open + close) else {
            break;
        };
        let attributes = attributes(&rest[open..=close]);
        let label = &rest[close + 1..];
        let label = label[..label.find('<').unwrap_or(label.len())].trim();
        rest = &rest[close + 1..];
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        let (Some(x), Some(y)) = (number("x"), number("y")) else {
            continue;
        };
        if label.is_empty() || attribute(&attributes, "class") == Some("ct-tip") {
            continue;
        }
        let [left, top, right, bottom] = text_box(&attributes, label, x, y);
        let reach = [
            EDGE - left,
            EDGE - top,
            right - (width - EDGE),
            bottom - (height - EDGE),
        ];
        for (over, reach) in over.iter_mut().zip(reach) {
            *over = over.max(reach);
        }
    }
    over.iter().any(|over| *over > 0.0).then_some(over)
}

/// A number the renderer may have written with a `k`, `M`, `G` or `T` suffix: `1.5k`.
pub(super) fn scaled(text: &str) -> Option<f64> {
    let factor = match text.chars().last()? {
        'k' => 1e3,
        'M' => 1e6,
        'G' => 1e9,
        'T' => 1e12,
        _ => return text.parse().ok(),
    };
    let number: f64 = text[..text.len() - 1].parse().ok()?;
    Some(number * factor)
}

pub(super) fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub(super) fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}
