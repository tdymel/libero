//! A pie chart's spec and drawing, its slices' arcs and rounded corners.

use super::*;

/// Everything a pie chart's SVG depends on; `slices` hold one value each.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PieSpec {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) slices: Vec<SeriesSpec>,
    /// The ring's width over the radius; `None`, a full pie.
    pub(crate) donut: Option<f32>,
    pub(crate) labels: PieLabels,
    /// Degrees clockwise from 12 o'clock.
    pub(crate) start_angle: f32,
    /// Where the last slice ends, in the same degrees; `None`, a full turn on.
    pub(crate) end_angle: Option<f32>,
    /// Every slice the same angle, its radius by its value: a nightingale rose.
    pub(crate) rose: bool,
    /// Between two slices, in pixels.
    pub(crate) slice_gap: f32,
    /// Corner radius of a slice, in pixels; 0 keeps them square.
    pub(crate) slice_radius: f32,
    /// Labels on their slices rather than beside them.
    pub(crate) labels_inside: bool,
    /// A slice narrower than this many degrees gets no label.
    pub(crate) min_label_angle: f32,
    pub(crate) ink: ChartInk,
    pub(crate) format: ValueFormat,
    pub(crate) unit: Option<String>,
    pub(crate) formats: &'static Formats,
}

impl PieSpec {
    /// The degrees the slices share. An end at or before the start counts a turn on.
    pub(super) fn span(&self) -> f32 {
        match self.end_angle {
            Some(end) if end.is_finite() && end > self.start_angle => {
                (end - self.start_angle).min(360.0)
            }
            Some(end) if end.is_finite() => {
                let span = (end - self.start_angle).rem_euclid(360.0);
                if span > 0.0 { span } else { 360.0 }
            }
            _ => 360.0,
        }
    }
}

/// A point `radius` from `(cx, cy)` at `angle` degrees clockwise from 12 o'clock.
pub(super) fn on_circle((cx, cy): (f32, f32), radius: f32, angle: f32) -> (f32, f32) {
    let (sin, cos) = angle.to_radians().sin_cos();
    (cx + radius * sin, cy - radius * cos)
}

/// The box round the part of the unit circle from `start` over `span` degrees with a hole
/// of radius `hole`, `[left, top, right, bottom]`, as the renderer centres it.
pub(super) fn arc_box(start: f32, span: f32, hole: f32) -> [f32; 4] {
    let mut angles = vec![start, start + span];
    let mut cardinal = (start / 90.0).ceil() * 90.0;
    while cardinal < start + span {
        angles.push(cardinal);
        cardinal += 90.0;
    }
    let mut found = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
    for angle in angles {
        for radius in [hole, 1.0] {
            let (x, y) = on_circle((0.0, 0.0), radius, angle);
            found = [
                found[0].min(x),
                found[1].min(y),
                found[2].max(x),
                found[3].max(y),
            ];
        }
    }
    found
}

pub(crate) fn draw_pie(spec: &PieSpec) -> Option<Drawn> {
    let series = spec
        .slices
        .iter()
        .map(|slice| {
            let value = slice.values.first().copied().flatten().unwrap_or(0.0);
            charts_rs::Series::new(slice.name.clone(), vec![value.max(0.0) as f32])
        })
        .collect();
    let mut chart = charts_rs::PieChart::new(series);
    chart.width = spec.width;
    chart.height = spec.height;
    chart.background_color = InkColor::transparent();
    chart.title.text = String::new();
    chart.legend.show = Some(false);
    chart.tooltip.show = true;
    chart.animation = None;
    chart.rose_type = Some(spec.rose);
    chart.border_radius = Some(0.0);
    chart.start_angle = spec.start_angle;
    let span = spec.span();
    chart.end_angle = (span < 360.0).then_some(spec.start_angle + span);
    chart.series.colors = spec.slices.iter().map(|s| ink(s.color)).collect();
    chart.series_label_position = spec.labels_inside.then(|| "inside".to_string());
    chart.min_show_label_angle = spec.min_label_angle.max(0.0);
    chart.series.label.font.color = match spec.labels_inside {
        true => ON_BAR,
        false => ink(spec.ink.label),
    };
    chart.series.label.formatter = match spec.labels {
        PieLabels::Value => format!("{AXIS_MARK}{{c}}{}", unit_suffix(spec.unit.as_deref())),
        PieLabels::Percent => format!("{PERCENT_MARK}{{d}}"),
        PieLabels::Name => "{a}".to_string(),
        PieLabels::None => String::new(),
    };
    if spec.labels == PieLabels::None {
        // Past a full turn: no slice is wide enough for a label.
        chart.min_show_label_angle = 361.0;
    }
    let hole_share = spec.donut.map_or(0.0, |ring| 1.0 - ring.clamp(0.05, 1.0));
    let ring = |radius: f32| radius * hole_share;
    let partial = span < 360.0;
    // The renderer centres a part of the circle on what is drawn of it, in the whole SVG.
    let [left, top, right, bottom] = arc_box(spec.start_angle, span, hole_share);
    let middle = |radius: f32| match partial {
        true => (
            spec.width / 2.0 - (left + right) / 2.0 * radius,
            spec.height / 2.0 - (top + bottom) / 2.0 * radius,
        ),
        false => (spec.width / 2.0, spec.height / 2.0),
    };
    // A full pie on a square canvas around the middle, its radius the renderer's share of
    // it: the pie stays centred as it shrinks for labels that would leave the SVG.
    let mut draw = |radius: f32| {
        let side = radius * 2.0 / PIE_FILL;
        chart.radius = radius;
        chart.inner_radius = ring(radius);
        chart.margin.left = if partial {
            0.0
        } else {
            (spec.width - side) / 2.0
        };
        chart.margin.right = chart.margin.left;
        chart.margin.top = if partial {
            0.0
        } else {
            (spec.height - side) / 2.0
        };
        chart.margin.bottom = chart.margin.top;
        chart.svg().ok()
    };
    let full = match partial {
        true => (spec.width * PIE_FILL / (right - left).max(0.1))
            .min(spec.height * PIE_FILL / (bottom - top).max(0.1)),
        false => spec.width.min(spec.height) * PIE_FILL / 2.0,
    };
    let mut radius = full;
    let mut svg = draw(radius)?;
    // A label moves in by at most what the radius loses: a few rounds settle it.
    for _ in 0..4 {
        let Some(over) = overflow(&svg, spec.width, spec.height) else {
            break;
        };
        let next = (radius - over.into_iter().fold(0.0, f32::max)).max(full * PIE_MIN);
        if next >= radius {
            break;
        }
        radius = next;
        svg = draw(radius)?;
    }
    let hole = ring(radius);
    let centre = middle(radius);
    let svg = match spec.labels_inside {
        true => centre_inside_labels(&svg, centre, hole),
        false => svg,
    };
    // What `center` fills: the hole, or the half of it a ring of half a turn or less bends round.
    let (inner, inner_radius) = match span <= 180.0 {
        true => (
            on_circle(centre, hole / 2.0, spec.start_angle + span / 2.0),
            hole / 2.0,
        ),
        false => (centre, hole),
    };

    // A line of the page between two slices of a close colour; a lone slice has no seam.
    let shares = spec
        .slices
        .iter()
        .filter(|s| s.values.first().copied().flatten().is_some_and(|v| v > 0.0))
        .count();
    let gap = if shares > 1 {
        spec.slice_gap.max(0.0)
    } else {
        0.0
    };
    let edge = format!(" stroke=\"{}\" stroke-width=\"{gap}\"", spec.ink.surface);
    let (svg, triggers) = inert(
        &svg,
        |label| {
            spec.format
                .format(label, spec.unit.as_deref(), spec.formats)
        },
        &edge,
        CircleTriggers::Dropped,
    );
    let (svg, triggers) = round_slices(svg, triggers, centre, spec.slice_radius);
    let svg = relabel(&svg, PERCENT_MARK, |share| {
        ValueFormat::Percent.format(share / 100.0, None, spec.formats)
    });
    let svg = match spec.labels_inside {
        true => svg,
        false => {
            let values: Vec<f64> = spec
                .slices
                .iter()
                .map(|s| s.values.first().copied().flatten().unwrap_or(0.0))
                .collect();
            drop_colliding_labels(&cut_labels(&svg, spec.width), &values)
        }
    };
    // Every slice is drawn, an empty one too.
    let expected: Vec<_> = spec
        .slices
        .iter()
        .enumerate()
        .map(|(i, slice)| {
            let value = slice.values.first().copied().flatten().unwrap_or(0.0);
            (0, i, value, slice.name.as_str())
        })
        .collect();
    let marks = match_marks(&expected, &triggers);
    let svg = ink_slice_labels(
        &svg,
        &marks,
        |slice| spec.ink.on(spec.slices[slice].color),
        spec.ink.label,
    );
    Some(Drawn {
        svg: spec.ink.paint(svg),
        geometry: Geometry {
            width: spec.width,
            height: spec.height,
            plot: Plot {
                left: inner.0 - inner_radius,
                top: inner.1 - inner_radius,
                right: inner.0 + inner_radius,
                bottom: inner.1 + inner_radius,
            },
            marks,
            hole,
            share_of: None,
            cells: false,
        },
    })
}

/// Every slice's path drawn again with its corners rounded by `radius`, in the SVG and in
/// the triggers; the renderer's own rounding leaves a knob at each corner.
pub(super) fn round_slices(
    svg: String,
    mut triggers: Vec<Trigger>,
    centre: (f32, f32),
    radius: f32,
) -> (String, Vec<Trigger>) {
    if radius <= 0.0 {
        return (svg, triggers);
    }
    let mut svg = svg;
    for trigger in &mut triggers {
        let MarkShape::Sector { path } = &mut trigger.shape else {
            continue;
        };
        if let Some(rounded) = rounded_sector(path, centre, radius) {
            svg = svg.replace(&format!("d=\"{path}\""), &format!("d=\"{rounded}\""));
            *path = rounded;
        }
    }
    (svg, triggers)
}

/// A sector's path with its outer corners, and its inner ones when it is a ring's, filleted
/// by `corner` pixels at most: what the thickness and the slice's angle leave of it. The
/// sector's radii and angles are read off the path; `None` for a full circle or none.
pub(super) fn rounded_sector(path: &str, (cx, cy): (f32, f32), corner: f32) -> Option<String> {
    let spaced: String = path
        .chars()
        .flat_map(|c| match c.is_ascii_alphabetic() {
            true => vec![' ', c, ' '],
            false => vec![c],
        })
        .collect();
    let mut commands: Vec<(char, Vec<f32>)> = Vec::new();
    for token in spaced.split([' ', ',']).filter(|t| !t.is_empty()) {
        match token.parse::<f32>() {
            Ok(number) => commands.last_mut()?.1.push(number),
            Err(_) => commands.push((token.chars().next()?, Vec::new())),
        }
    }
    let radii: Vec<f32> = commands
        .iter()
        .filter(|(c, args)| *c == 'A' && args.first().is_some_and(|r| *r > 0.5))
        .map(|(_, args)| args[0])
        .collect();
    let outer = radii.iter().copied().fold(0.0, f32::max);
    let inner = radii
        .iter()
        .copied()
        .filter(|r| outer - r > 0.5)
        .fold(0.0, f32::max);
    let rim: Vec<(f32, f32)> = commands
        .iter()
        .filter_map(|(c, args)| match (c, args.len()) {
            ('M' | 'L', 2) => Some((args[0], args[1])),
            ('A', 7) => Some((args[5], args[6])),
            _ => None,
        })
        .filter(|(x, y)| ((x - cx).hypot(y - cy) - outer).abs() < 0.6)
        .collect();
    let angle = |(x, y): (f32, f32)| (x - cx).atan2(cy - y).to_degrees().rem_euclid(360.0);
    let start = angle(*rim.first()?);
    let mut span = angle(*rim.last()?) - start;
    if span <= 0.0 {
        span += 360.0;
    }
    if outer <= 0.0 || !(0.5..359.5).contains(&span) {
        return None;
    }
    // A fillet of radius `b` inside the outer arc sits `outer - b` from the centre, `b` off
    // the edge; round the ring's inside just so, outside the inner arc.
    let room = (span / 2.0 * 0.9).to_radians().sin();
    let mut b = corner
        .min((outer - inner) / 2.0)
        .min(outer * room / (1.0 + room));
    if inner > 0.0 {
        b = b.min(inner * room / (1.0 - room));
    }
    if b <= 0.1 {
        return None;
    }
    let b = (b * 100.0).floor() / 100.0;
    let at = |radius: f32, degrees: f32| {
        let (sin, cos) = degrees.to_radians().sin_cos();
        (cx + radius * sin, cy - radius * cos)
    };
    let point = |(x, y): (f32, f32)| {
        format!(
            "{},{}",
            (x * 100.0).round() / 100.0,
            (y * 100.0).round() / 100.0
        )
    };
    let end = start + span;
    // Where a fillet touches the edge, and how far round the arc it touches.
    let touch = |centre_radius: f32| {
        let turn = (b / centre_radius).asin().to_degrees();
        ((centre_radius.powi(2) - b * b).sqrt(), turn)
    };
    let (edge_o, turn_o) = touch(outer - b);
    let large = |turn: f32| u8::from(span - 2.0 * turn > 180.0);
    let d = match inner > 0.0 {
        true => {
            let (edge_i, turn_i) = touch(inner + b);
            format!(
                "M{} L{} A{b},{b} 0 0 1 {} A{outer},{outer} 0 {} 1 {} A{b},{b} 0 0 1 {} L{} A{b},{b} 0 0 1 {} A{inner},{inner} 0 {} 0 {} A{b},{b} 0 0 1 {} Z",
                point(at(edge_i, start)),
                point(at(edge_o, start)),
                point(at(outer, start + turn_o)),
                large(turn_o),
                point(at(outer, end - turn_o)),
                point(at(edge_o, end)),
                point(at(edge_i, end)),
                point(at(inner, end - turn_i)),
                large(turn_i),
                point(at(inner, start + turn_i)),
                point(at(edge_i, start)),
            )
        }
        false => format!(
            "M{} L{} A{b},{b} 0 0 1 {} A{outer},{outer} 0 {} 1 {} A{b},{b} 0 0 1 {} L{} Z",
            point((cx, cy)),
            point(at(edge_o, start)),
            point(at(outer, start + turn_o)),
            large(turn_o),
            point(at(outer, end - turn_o)),
            point(at(edge_o, end)),
            point((cx, cy)),
        ),
    };
    Some(d)
}
