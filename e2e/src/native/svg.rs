//! The box of an SVG mark. Blitz paints an `<svg>` as one picture and lays out none of its
//! children, so a `<path>` reads 0x0; the web's client rect is rebuilt from the attributes.

use blitz_dom::BaseDocument;
use blitz_traits::NodeId;

type Rect = (f64, f64, f64, f64);

/// A mark's client rect: its user-space box mapped through the `<svg>`'s `viewBox`. `None`
/// for anything but a laid-out-less mark in an `<svg>`.
pub(super) fn mark_rect(doc: &BaseDocument, id: NodeId) -> Option<Rect> {
    let node = doc.get_node(id)?;
    let size = node.final_layout().size;
    if &*node.element_data()?.name.local == "svg" || size.width != 0.0 || size.height != 0.0 {
        return None;
    }
    let mut svg = node.parent.and_then(|id| doc.get_node(id));
    while let Some(parent) = svg {
        if parent
            .element_data()
            .is_some_and(|data| &*data.name.local == "svg")
        {
            break;
        }
        svg = parent.parent.and_then(|id| doc.get_node(id));
    }
    let svg = svg?;
    let (left, top, right, bottom) = user_box(node)?;
    let (x, y, width, height) = super::client_rect(doc, svg.id)?;
    let view_box = attr(svg, "viewBox").and_then(|view| {
        let numbers: Vec<f64> = view
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter_map(|n| n.parse().ok())
            .collect();
        <[f64; 4]>::try_from(numbers).ok()
    });
    // preserveAspectRatio's default: scaled to fit, centred.
    let [vx, vy, vw, vh] = view_box.unwrap_or([0.0, 0.0, width, height]);
    let scale = (width / vw).min(height / vh);
    let (ox, oy) = (
        x + (width - vw * scale) / 2.0 - vx * scale,
        y + (height - vh * scale) / 2.0 - vy * scale,
    );
    Some((
        ox + left * scale,
        oy + top * scale,
        (right - left) * scale,
        (bottom - top) * scale,
    ))
}

fn attr<'a>(node: &'a blitz_dom::Node, name: &str) -> Option<&'a str> {
    node.attrs()?
        .iter()
        .find(|attr| &*attr.name.local == name)
        .map(|attr| &*attr.value)
}

/// `(left, top, right, bottom)` in user units, from the mark's own attributes.
fn user_box(node: &blitz_dom::Node) -> Option<(f64, f64, f64, f64)> {
    let number =
        |name| attr(node, name).and_then(|v| v.trim().trim_end_matches("px").parse::<f64>().ok());
    let or_zero = |name| number(name).unwrap_or(0.0);
    match &*node.element_data()?.name.local {
        "rect" => {
            let (x, y) = (or_zero("x"), or_zero("y"));
            Some((x, y, x + or_zero("width"), y + or_zero("height")))
        }
        "line" => {
            let xs = [or_zero("x1"), or_zero("x2")];
            let ys = [or_zero("y1"), or_zero("y2")];
            Some((
                xs[0].min(xs[1]),
                ys[0].min(ys[1]),
                xs[0].max(xs[1]),
                ys[0].max(ys[1]),
            ))
        }
        "circle" => {
            let (cx, cy, r) = (or_zero("cx"), or_zero("cy"), or_zero("r"));
            Some((cx - r, cy - r, cx + r, cy + r))
        }
        "ellipse" => {
            let (cx, cy) = (or_zero("cx"), or_zero("cy"));
            let (rx, ry) = (or_zero("rx"), or_zero("ry"));
            Some((cx - rx, cy - ry, cx + rx, cy + ry))
        }
        "polygon" | "polyline" => bounds(
            &numbers(attr(node, "points")?)?
                .chunks(2)
                .map(|p| (p[0], p[1]))
                .collect::<Vec<_>>(),
        ),
        "path" => bounds(&path_points(attr(node, "d")?)),
        "text" => {
            let (x, y) = (or_zero("x"), or_zero("y"));
            let size = number("font-size").unwrap_or(12.0);
            let width = node.text_content().chars().count() as f64 * size * 0.55;
            let left = match attr(node, "text-anchor") {
                Some("middle") => x - width / 2.0,
                Some("end") => x - width,
                _ => x,
            };
            Some((left, y - size, left + width, y))
        }
        _ => None,
    }
}

fn numbers(text: &str) -> Option<Vec<f64>> {
    let all: Vec<f64> = text
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|n| !n.is_empty())
        .map(|n| n.parse().ok())
        .collect::<Option<_>>()?;
    all.len().is_multiple_of(2).then_some(all)
}

fn bounds(points: &[(f64, f64)]) -> Option<(f64, f64, f64, f64)> {
    let first = points.first()?;
    Some(points.iter().fold(
        (first.0, first.1, first.0, first.1),
        |(l, t, r, b), &(x, y)| (l.min(x), t.min(y), r.max(x), b.max(y)),
    ))
}

/// The points a path visits, control points included; an arc contributes its end only.
fn path_points(d: &str) -> Vec<(f64, f64)> {
    let mut tokens = Vec::new();
    let mut number = String::new();
    let flush = |number: &mut String, tokens: &mut Vec<Result<f64, char>>| {
        if let Ok(n) = number.parse() {
            tokens.push(Ok(n));
        }
        number.clear();
    };
    for c in d.chars() {
        match c {
            'a'..='z' | 'A'..='Z' if c != 'e' && c != 'E' => {
                flush(&mut number, &mut tokens);
                tokens.push(Err(c));
            }
            ',' | ' ' | '\t' | '\n' => flush(&mut number, &mut tokens),
            '-' if !number.is_empty() && !number.ends_with(['e', 'E']) => {
                flush(&mut number, &mut tokens);
                number.push(c);
            }
            '.' if number.contains('.') => {
                flush(&mut number, &mut tokens);
                number.push(c);
            }
            _ => number.push(c),
        }
    }
    flush(&mut number, &mut tokens);

    let (mut points, mut current, mut start) = (Vec::new(), (0.0, 0.0), (0.0, 0.0));
    let mut tokens = tokens.into_iter().peekable();
    let mut command = 'M';
    loop {
        match tokens.peek() {
            None => break,
            Some(Err(c)) => {
                command = *c;
                tokens.next();
                if matches!(command, 'z' | 'Z') {
                    current = start;
                    continue;
                }
            }
            Some(Ok(_)) => {}
        }
        let relative = command.is_ascii_lowercase();
        // Arguments per repeat; the pairs that are points; the arc's flags sit before its end.
        let count = match command.to_ascii_uppercase() {
            'M' | 'L' | 'T' => 2,
            'H' | 'V' => 1,
            'C' => 6,
            'S' | 'Q' => 4,
            'A' => 7,
            _ => break,
        };
        let args: Vec<f64> = (0..count).filter_map(|_| tokens.next()?.ok()).collect();
        if args.len() < count {
            break;
        }
        let base = if relative { current } else { (0.0, 0.0) };
        match command.to_ascii_uppercase() {
            'H' => {
                current.0 = if relative {
                    current.0 + args[0]
                } else {
                    args[0]
                }
            }
            'V' => {
                current.1 = if relative {
                    current.1 + args[0]
                } else {
                    args[0]
                }
            }
            'A' => current = (base.0 + args[5], base.1 + args[6]),
            _ => {
                for pair in args.chunks(2) {
                    points.push((base.0 + pair[0], base.1 + pair[1]));
                }
                current = *points.last().unwrap_or(&current);
            }
        }
        if matches!(command.to_ascii_uppercase(), 'M') {
            start = current;
            // A moveto's further pairs are linetos.
            command = if relative { 'l' } else { 'L' };
        }
        points.push(current);
    }
    points
}
