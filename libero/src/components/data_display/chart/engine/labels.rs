//! Label placement: measured, fitted, moved off each other or dropped, and inked.

use super::*;

/// The renderer writes an upright bar's label past its top, inside a falling bar. Moves a
/// falling one's under its lower end, or just inside it in the bar's ink when the x axis is
/// too near; a label past its bar is haloed in `surface`, so a line under it cannot blur it.
pub(super) fn place_outside_labels(svg: &str, bottom: f32, surface: HexColor) -> String {
    const GAP: f32 = 8.0;
    let halo = format!(
        " stroke=\"{surface}\" stroke-width=\"3\" stroke-linejoin=\"round\" paint-order=\"stroke\""
    );
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        let Some(close) = rest[open..].find('>').map(|close| open + close) else {
            break;
        };
        out.push_str(&rest[..open]);
        let tag = &rest[open..=close];
        let label = rest[close + 1..].trim_start();
        rest = &rest[close + 1..];
        let attributes = attributes(tag);
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        let (Some(end), Some(value)) = (number("y"), label.strip_prefix(LABEL_MARK)) else {
            out.push_str(tag);
            continue;
        };
        let size = number("font-size").unwrap_or(14.0);
        let below = GAP + size * 0.8;
        let (dy, inside) = match value.starts_with('-') {
            false => (number("dy").unwrap_or(0.0), false),
            true if end + below + size * 0.25 <= bottom => (below, false),
            true => (-(GAP / 2.0 + size * 0.25), true),
        };
        let kept: String = attributes
            .iter()
            .filter(|(name, _)| *name != "dy" && !(inside && *name == "fill"))
            .map(|(name, value)| format!(" {name}=\"{value}\""))
            .collect();
        let ink = if inside {
            format!(" {ON_BAR_FILL}")
        } else {
            halo.clone()
        };
        out.push_str(&format!("<text dy=\"{dy}\"{kept}{ink}>"));
    }
    out.push_str(rest);
    out
}

/// Drops the value labels, as `text` writes them, that overlap a larger value's.
pub(super) fn drop_crowded_labels(svg: &str, text: impl Fn(f64) -> String) -> String {
    // Per label: its value, its box and its element's byte range.
    let mut labels: Vec<(f64, [f32; 4], std::ops::Range<usize>)> = Vec::new();
    let mut at = 0;
    while let Some(open) = svg[at..].find("<text ").map(|open| at + open) {
        let Some(close) = svg[open..].find('>').map(|close| open + close) else {
            break;
        };
        let end = svg[close..]
            .find("</text>")
            .map_or(close + 1, |end| close + end + 7);
        at = close + 1;
        let attributes = attributes(&svg[open..=close]);
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        let content = svg[close + 1..end].trim_end_matches("</text>").trim();
        let value = content
            .strip_prefix(LABEL_MARK)
            .and_then(|label| scaled(label.split(' ').next()?.trim_end_matches('%')));
        let (Some(value), Some(x), Some(y)) = (value, number("x"), number("y")) else {
            continue;
        };
        let width = text_width(&attributes, &unescape(&text(value)));
        let left = match attribute(&attributes, "text-anchor") {
            Some("middle") => x - width / 2.0,
            Some("end") => x - width,
            _ => x,
        };
        let size = number("font-size").unwrap_or(14.0);
        let y = y + number("dy").unwrap_or(0.0);
        labels.push((
            value,
            [left, y - size * 0.75, left + width, y + size * 0.25],
            open..end,
        ));
        at = end;
    }
    let mut order: Vec<usize> = (0..labels.len()).collect();
    order.sort_by(|a, b| labels[*b].0.abs().total_cmp(&labels[*a].0.abs()));
    let mut kept: Vec<[f32; 4]> = Vec::new();
    let mut dropped = Vec::new();
    for i in order {
        let [left, top, right, bottom] = labels[i].1;
        let overlaps = kept
            .iter()
            .any(|[l, t, r, b]| left < *r && *l < right && top < *b && *t < bottom);
        match overlaps {
            true => dropped.push(labels[i].2.clone()),
            false => kept.push(labels[i].1),
        }
    }
    dropped.sort_by_key(|range| range.start);
    let mut out = String::with_capacity(svg.len());
    let mut from = 0;
    for range in dropped {
        out.push_str(&svg[from..range.start]);
        from = range.end;
    }
    out.push_str(&svg[from..]);
    out
}

/// Anchors every label on a stage at its middle, on its baseline: the renderer writes it
/// from its left end round a central baseline, measured in its own font.
pub(super) fn centre_stage_labels(svg: &str) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(start) = rest.find("<text ") {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find('>')
            .map_or(rest.len(), |end| start + end + 1);
        let tag = &rest[start..end];
        rest = &rest[end..];
        let attributes = attributes(tag);
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        let label = rest[..rest.find('<').unwrap_or(rest.len())].trim();
        let (Some(x), Some(y), true) = (number("x"), number("y"), tag.contains(ON_BAR_FILL)) else {
            out.push_str(tag);
            continue;
        };
        let size = number("font-size").unwrap_or(14.0);
        let kept: String = attributes
            .iter()
            .filter(|(attribute, _)| !matches!(*attribute, "x" | "y" | "dominant-baseline"))
            .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
            .collect();
        out.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\"{kept}>",
            x + text_width(&attributes, label) / 2.0,
            y + size * 0.35
        ));
    }
    out.push_str(rest);
    out
}

/// Moves every label on a slice to the middle of its slice's band, between the hole and the
/// slice's own radius. The renderer puts it two thirds of that radius out from `centre`,
/// in a donut's hole or by a rose's small slice's centre; anchored at its middle, it stays
/// centred once written again.
pub(super) fn centre_inside_labels(svg: &str, centre: (f32, f32), hole: f32) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(start) = rest.find("<text ") {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find('>')
            .map_or(rest.len(), |end| start + end + 1);
        let tag = &rest[start..end];
        rest = &rest[end..];
        let attributes = attributes(tag);
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        let label = rest[..rest.find('<').unwrap_or(rest.len())].trim();
        let on_slice = tag.contains(ON_BAR_FILL) && attribute(&attributes, "text-anchor").is_none();
        let (Some(x), Some(y), true) = (number("x"), number("y"), on_slice) else {
            out.push_str(tag);
            continue;
        };
        let (dx, dy) = (
            x + text_width(&attributes, label) / 2.0 - centre.0,
            y - centre.1,
        );
        let distance = dx.hypot(dy);
        if distance < 0.5 {
            out.push_str(tag);
            continue;
        }
        let outer = distance * 1.5;
        let scale = (hole + outer) / 2.0 / distance;
        let size = number("font-size").unwrap_or(14.0);
        let (x, y) = (centre.0 + dx * scale, centre.1 + dy * scale + size * 0.35);
        let kept: String = attributes
            .iter()
            .filter(|(attribute, _)| !matches!(*attribute, "x" | "y"))
            .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
            .collect();
        out.push_str(&format!(
            "<text x=\"{x}\" y=\"{y}\" text-anchor=\"middle\"{kept}>"
        ));
    }
    out.push_str(rest);
    out
}

/// Drops the outside labels that overlap a larger slice's, with their leader lines: the
/// smaller slice's goes first. `values` are the slices' in drawing order.
pub(super) fn drop_colliding_labels(svg: &str, values: &[f64]) -> String {
    // Per label: its slice's value, its box, and the byte ranges of its leader and its text.
    let mut labels: Vec<(f64, [f32; 4], Vec<std::ops::Range<usize>>)> = Vec::new();
    let (mut slice, mut leader) = (None::<usize>, None);
    let mut at = 0;
    while let Some(found) = svg[at..].find('<') {
        let start = at + found;
        let end = svg[start..]
            .find('>')
            .map_or(svg.len(), |end| start + end + 1);
        let tag = &svg[start..end];
        at = end;
        if tag.starts_with("<path ") {
            match tag.contains("fill=\"none\"") {
                true => leader = Some(start..end),
                false => {
                    slice = Some(slice.map_or(0, |i| i + 1));
                    leader = None;
                }
            }
        } else if tag.starts_with("<text ") {
            let close = svg[end..]
                .find("</text>")
                .map_or(svg.len(), |close| end + close + 7);
            at = close;
            let attributes = attributes(tag);
            let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
            let (Some(x), Some(y), Some(index)) = (number("x"), number("y"), slice) else {
                continue;
            };
            let label = unescape(svg[end..close].trim_end_matches("</text>").trim());
            let width = text_width(&attributes, &label);
            let left = match attribute(&attributes, "text-anchor") {
                Some("middle") => x - width / 2.0,
                Some("end") => x - width,
                _ => x,
            };
            let size = number("font-size").unwrap_or(14.0);
            let ranges = leader
                .take()
                .into_iter()
                .chain(std::iter::once(start..close))
                .collect();
            let value = values.get(index).copied().unwrap_or(0.0);
            labels.push((
                value,
                [left, y - size * 0.75, left + width, y + size * 0.25],
                ranges,
            ));
        }
    }
    let mut order: Vec<usize> = (0..labels.len()).collect();
    order.sort_by(|a, b| labels[*b].0.total_cmp(&labels[*a].0));
    let mut kept: Vec<[f32; 4]> = Vec::new();
    let mut dropped: Vec<std::ops::Range<usize>> = Vec::new();
    for i in order {
        let [left, top, right, bottom] = labels[i].1;
        let overlaps = kept
            .iter()
            .any(|[l, t, r, b]| left < *r && *l < right && top < *b && *t < bottom);
        match overlaps {
            true => dropped.extend(labels[i].2.iter().cloned()),
            false => kept.push(labels[i].1),
        }
    }
    dropped.sort_by_key(|range| range.start);
    let mut out = String::with_capacity(svg.len());
    let mut from = 0;
    for range in dropped {
        out.push_str(&svg[from..range.start]);
        from = range.end;
    }
    out.push_str(&svg[from..]);
    out
}

/// Cuts a label that leaves the SVG at a side short with an ellipsis, inside [`EDGE`]; the
/// tooltip and the summary keep the whole text.
pub(super) fn cut_labels(svg: &str, width: f32) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(start) = rest.find("<text ") {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find('>')
            .map_or(rest.len(), |end| start + end + 1);
        let tag = &rest[start..end];
        rest = &rest[end..];
        let attributes = attributes(tag);
        let text_end = rest.find('<').unwrap_or(rest.len());
        let label = unescape(rest[..text_end].trim());
        let anchor = attribute(&attributes, "text-anchor");
        let Some(x) = attribute(&attributes, "x").and_then(|x| x.parse::<f32>().ok()) else {
            out.push_str(tag);
            continue;
        };
        let drawn = text_width(&attributes, &label);
        let left = match anchor {
            Some("middle") => x - drawn / 2.0,
            Some("end") => x - drawn,
            _ => x,
        };
        let (from, to) = (left.max(EDGE), (left + drawn).min(width - EDGE));
        let tip = attribute(&attributes, "class") == Some("ct-tip");
        if tip || label.is_empty() || (from <= left && to >= left + drawn) {
            out.push_str(tag);
            continue;
        }
        let cut = shorten(&attributes, &label, to - from);
        let x = match anchor {
            Some("middle") => (from + to) / 2.0,
            Some("end") => to,
            _ => from,
        };
        let kept: String = attributes
            .iter()
            .filter(|(attribute, _)| *attribute != "x")
            .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
            .collect();
        out.push_str(&format!("<text x=\"{x}\"{kept}>\n{}\n", escape(&cut)));
        rest = &rest[text_end..];
    }
    out.push_str(rest);
    out
}

/// `label` cut short with an ellipsis to at most `room` wide in the tag's font, or as is
/// where it fits.
pub(super) fn shorten(attributes: &[(&str, &str)], label: &str, room: f32) -> String {
    if text_width(attributes, label) <= room {
        return label.to_string();
    }
    let mut chars: Vec<char> = label.chars().collect();
    let mut cut = String::from("…");
    while chars.pop().is_some() {
        cut = chars.iter().collect::<String>().trim_end().to_string() + "…";
        if text_width(attributes, &cut) <= room || chars.is_empty() {
            break;
        }
    }
    cut
}

/// Fits the category `labels` under the plot each into its band, cut with an ellipsis; where
/// a band is too narrow for three letters and the ellipsis, only every n-th label is written.
pub(super) fn fit_x_labels(svg: &str, labels: &[String], width: f32, height: f32) -> String {
    if labels.is_empty() {
        return svg.to_string();
    }
    // Each text's tag start, its element's end, its x and y and its content.
    let mut texts = Vec::new();
    let mut at = 0;
    while let Some(open) = svg[at..].find("<text ").map(|open| at + open) {
        let Some(close) = svg[open..].find('>').map(|close| open + close) else {
            break;
        };
        let end = svg[close..]
            .find("</text>")
            .map_or(close + 1, |end| close + end + 7);
        let attributes = attributes(&svg[open..=close]);
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        let content = unescape(svg[close + 1..end].trim_end_matches("</text>").trim());
        if let (Some(x), Some(y)) = (number("x"), number("y")) {
            texts.push((open, end, x, y, content));
        }
        at = close + 1;
    }
    // The renderer writes each one whole or cut short with its own ellipsis.
    let drawn = |content: &str, label: &str| {
        content == label
            || content
                .strip_suffix('…')
                .is_some_and(|start| label.starts_with(start.trim_end()))
    };
    let Some(first) = (0..texts.len().saturating_sub(labels.len() - 1)).find(|&start| {
        let row = &texts[start..start + labels.len()];
        row.iter()
            .zip(labels)
            .all(|(text, label)| drawn(&text.4, label))
            && row.iter().all(|text| text.3 == row[0].3)
    }) else {
        return svg.to_string();
    };
    let row = &texts[first..first + labels.len()];
    let plot = Plot::of(svg, width, height);
    let band = (plot.right - plot.left) / labels.len() as f32;
    let tag = |open: usize| attributes(&svg[open..=open + svg[open..].find('>').unwrap_or(0)]);
    // Each label's first three letters and the ellipsis, or the whole of a shorter one.
    let least = labels.iter().fold(0.0_f32, |most, label| {
        let start: String = label.chars().take(3).collect();
        let shown = match start.len() < label.len() {
            true => start + "…",
            false => start,
        };
        most.max(roomy(&tag(row[0].0), &shown))
    });
    let fits = |least: f32| {
        (1..=labels.len())
            .find(|n| *n as f32 * band - EDGE >= least)
            .unwrap_or(labels.len())
    };
    // Thinned anyway, they are written whole while at least three are left.
    let widest = labels.iter().fold(0.0_f32, |most, label| {
        most.max(roomy(&tag(row[0].0), label))
    });
    let every = match (fits(least), fits(widest)) {
        (1, _) => 1,
        (some, whole) if labels.len().div_ceil(whole) >= 3 => whole.max(some),
        (some, _) => some,
    };
    let mut out = String::with_capacity(svg.len());
    let mut last = 0;
    // Where the label before ends.
    let mut written = f32::MIN;
    for (index, ((open, end, _, _, _), label)) in row.iter().zip(labels).enumerate() {
        out.push_str(&svg[last..*open]);
        last = *end;
        if index % every != 0 {
            continue;
        }
        let attributes = tag(*open);
        let band_centre = plot.left + band * (index as f32 + 0.5);
        let mut cut = shorten(
            &attributes,
            label,
            (every as f32 * band - EDGE) / PAGE_FONT_SLACK,
        );
        // Under its band, moved in from a side of the SVG, else cut to stay centred there.
        let half = roomy(&attributes, &cut) / 2.0;
        let mut centre = band_centre.clamp(EDGE + half, (width - EDGE - half).max(EDGE + half));
        if centre - half < written + EDGE {
            let side = (band_centre - EDGE).min(width - EDGE - band_centre);
            cut = shorten(&attributes, label, 2.0 * side / PAGE_FONT_SLACK);
            centre = band_centre;
        }
        written = centre + roomy(&attributes, &cut) / 2.0;
        let kept: String = attributes
            .iter()
            .filter(|(attribute, _)| !matches!(*attribute, "x" | "text-anchor"))
            .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
            .collect();
        out.push_str(&format!(
            "<text x=\"{centre}\" text-anchor=\"middle\"{kept}>\n{}\n</text>",
            escape(&cut)
        ));
    }
    out.push_str(&svg[last..]);
    out
}

/// Gives every label drawn on a slice the ink that reads on that slice; one off every
/// slice takes `off`.
pub(super) fn ink_slice_labels(
    svg: &str,
    marks: &[Mark],
    ink: impl Fn(usize) -> HexColor,
    off: HexColor,
) -> String {
    let outlines: Vec<(usize, _)> = marks
        .iter()
        .filter_map(|mark| match &mark.shape {
            MarkShape::Sector { path } => Some((mark.index, super::super::hit::outline(path))),
            _ => None,
        })
        .collect();
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(start) = rest.find("<text ") {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find('>')
            .map_or(rest.len(), |end| start + end + 1);
        let tag = &rest[start..end];
        rest = &rest[end..];
        if !tag.contains(ON_BAR_FILL) {
            out.push_str(tag);
            continue;
        }
        let attributes = attributes(tag);
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        let label = &rest[..rest.find('<').unwrap_or(rest.len())];
        if label.trim().is_empty() {
            rest = rest.find("</text>").map_or("", |close| &rest[close + 7..]);
            continue;
        }
        // The renderer writes a label on a slice from its left end; `centre_inside_labels`
        // anchors it at its middle.
        let middle = number("x").zip(number("y")).map(|(x, y)| {
            let size = number("font-size").unwrap_or(14.0);
            let x = match attribute(&attributes, "text-anchor") {
                Some("middle") => x,
                _ => x + text_width(&attributes, label.trim()) / 2.0,
            };
            (x, y - size * 0.35)
        });
        let on = middle.and_then(|(x, y)| {
            outlines
                .iter()
                .find(|(_, outline)| super::super::hit::contains(outline, x, y))
        });
        // A label wider or taller than its slice is dropped; the summary still has it.
        let half = text_width(&attributes, label.trim()) / 2.0;
        let rise = number("font-size").unwrap_or(14.0) * 0.35;
        // Glyphs leave a box's corners empty: those are checked a little in.
        let inset = half * 0.8;
        if let (Some((x, y)), Some((_, outline))) = (middle, on)
            && ![
                (-half, 0.0),
                (half, 0.0),
                (-inset, -rise),
                (inset, -rise),
                (inset, rise),
                (-inset, rise),
            ]
            .iter()
            .all(|(dx, dy)| super::super::hit::contains(outline, x + dx, y + dy))
        {
            rest = rest.find("</text>").map_or("", |close| &rest[close + 7..]);
            continue;
        }
        let hex = on.map_or(off, |(slice, _)| ink(*slice));
        out.push_str(&tag.replace(ON_BAR_FILL, &format!("fill=\"{hex}\"")));
    }
    out.push_str(rest);
    out
}

/// A label's width in the renderer's own measure, in the tag's font.
pub(super) fn text_width(attributes: &[(&str, &str)], text: &str) -> f32 {
    // `sanitize` hands the renderer's font to the page as `inherit`.
    let family = attribute(attributes, "font-family")
        .filter(|family| *family != "inherit")
        .unwrap_or(charts_rs::DEFAULT_FONT_FAMILY);
    let size = attribute(attributes, "font-size")
        .and_then(|size| size.parse().ok())
        .unwrap_or(14.0);
    charts_rs::measure_text_width_family(family, size, text).map_or(0.0, |text| text.width())
}

/// The page's font is wider than the renderer's Roboto; its width, with that margin.
pub(super) fn roomy(attributes: &[(&str, &str)], text: &str) -> f32 {
    text_width(attributes, text) * PAGE_FONT_SLACK
}

/// The box `[left, top, right, bottom]` of `label` written at the baseline point `x`, `y`
/// (before the tag's `dx` and `dy`): the ascent above the baseline, the descent below.
pub(super) fn text_box(attributes: &[(&str, &str)], label: &str, x: f32, y: f32) -> [f32; 4] {
    let number = |name| attribute(attributes, name)?.parse::<f32>().ok();
    let (x, y) = (
        x + number("dx").unwrap_or(0.0),
        y + number("dy").unwrap_or(0.0),
    );
    let size = number("font-size").unwrap_or(14.0);
    let drawn = text_width(attributes, label);
    let left = match attribute(attributes, "text-anchor") {
        Some("middle") => x - drawn / 2.0,
        Some("end") => x - drawn,
        _ => x,
    };
    [left, y - size * 0.8, left + drawn, y + size * 0.25]
}
