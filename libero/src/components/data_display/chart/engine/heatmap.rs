//! A heatmap's spec and drawing, its cells' colours, labels and scale.

use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HeatmapSpec {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) x: Vec<String>,
    /// The rows' names, top first.
    pub(crate) y: Vec<String>,
    /// A row per `y`, a value per `x`; `None` is an empty cell.
    pub(crate) values: Vec<Vec<Option<f64>>>,
    /// The scale, low to high: two or more.
    pub(crate) colors: Vec<HexColor>,
    pub(crate) range: (f64, f64),
    /// Classes of the same width instead of a gradient; below 2, none.
    pub(crate) steps: usize,
    pub(crate) labels: bool,
    /// The colour scale drawn under the plot, with its values.
    pub(crate) scale: bool,
    /// The scale's key for an empty cell, when there is one.
    pub(crate) no_value: Option<String>,
    pub(crate) ink: ChartInk,
    pub(crate) format: ValueFormat,
    pub(crate) unit: Option<String>,
    pub(crate) formats: &'static Formats,
}

impl HeatmapSpec {
    /// A row's value at a column, if it has a finite one.
    pub(crate) fn value(&self, row: usize, column: usize) -> Option<f64> {
        self.values
            .get(row)?
            .get(column)
            .copied()
            .flatten()
            .filter(|value| value.is_finite())
    }
}

pub(crate) fn draw_heatmap(spec: &HeatmapSpec) -> Option<Drawn> {
    let (rows, columns) = (spec.y.len(), spec.x.len());
    if rows == 0 || columns == 0 {
        return None;
    }
    // The renderer's row 0 is the bottom one; ours is the top.
    let below = |row: usize| rows - 1 - row;
    let data: Vec<(usize, f32)> = (0..rows)
        .flat_map(|row| (0..columns).map(move |column| (row, column)))
        .filter_map(|(row, column)| {
            let value = spec.value(row, column)?;
            Some((below(row) * columns + column, value as f32))
        })
        .collect();
    let y_drawn: Vec<String> = spec.y.iter().rev().cloned().collect();
    let mut chart = charts_rs::HeatmapChart::new(data, spec.x.clone(), y_drawn);
    let room = match spec.scale {
        true => SCALE_ROOM.min(spec.height / 3.0),
        false => 0.0,
    };
    chart.width = spec.width;
    chart.height = spec.height - room;
    chart.background_color = InkColor::transparent();
    chart.title.text = String::new();
    chart.legend.show = Some(false);
    chart.tooltip.show = true;
    chart.animation = None;
    chart.x_axis.stroke_color = ink(spec.ink.axis);
    chart.x_axis.font.color = ink(spec.ink.label);
    chart.x_axis.label_overflow = AxisLabelOverflow::Ellipsis;
    let (low, high) = spec.range;
    let high = if high > low { high } else { low + 1.0 };
    let scale = &mut chart.heatmap_series;
    scale.min = low as f32;
    scale.max = high as f32;
    scale.colors = spec.colors.iter().map(|color| ink(*color)).collect();
    scale.steps = spec.steps;
    scale.min_font_color = (0, 0, 0).into();
    scale.max_font_color = (255, 255, 255).into();
    chart.series.label.formatter = format!("{LABEL_MARK}{{c}}");
    let svg = chart.svg().ok()?;
    let svg = fit_x_labels(&svg, &spec.x, spec.width, spec.height);
    let (svg, triggers) = inert(&svg, |value| value.to_string(), "", false);
    let svg = restyle_cells(&svg, &spec.ink);
    let expected: Vec<_> = (0..rows)
        .rev()
        .flat_map(|row| {
            (0..columns).filter_map(move |column| {
                Some((row, column, spec.value(row, column)?, spec.y[row].as_str()))
            })
        })
        .collect();
    let mut marks = match_marks(&expected, &triggers);
    let write = |value: f64| {
        spec.format
            .format(value, spec.unit.as_deref(), spec.formats)
    };
    let svg = write_cell_labels(&svg, &marks, spec.labels, write);
    marks.sort_by_key(|mark| (mark.series, mark.index));
    let plot = Plot::of(&svg, spec.width, spec.height);
    let svg = match spec.scale {
        true => with_scale(&svg, spec, plot, spec.height - room, write),
        false => svg,
    };
    Some(Drawn {
        svg: spec.ink.paint(svg),
        geometry: Geometry {
            width: spec.width,
            height: spec.height,
            plot,
            marks,
            hole: 0.0,
            share_of: None,
            cells: true,
        },
    })
}

/// Room under a heatmap's plot for its colour scale, the bar's height, its pieces.
pub(super) const SCALE_ROOM: f32 = 40.0;
pub(super) const SCALE_BAR: f32 = 10.0;
pub(super) const SCALE_PIECES: usize = 32;
/// An empty cell's outline, and its key's.
pub(super) const EMPTY_DASH: &str = "4,3";

/// The colour at `position` (0 to 1) of a gradient through `colors`, as the renderer mixes it.
pub(super) fn gradient(colors: &[HexColor], position: f64) -> HexColor {
    let (Some(first), Some(last)) = (colors.first(), colors.last()) else {
        return HexColor::new(0);
    };
    if !position.is_finite() || position <= 0.0 || colors.len() < 2 {
        return *first;
    }
    if position >= 1.0 {
        return *last;
    }
    let place = position * (colors.len() - 1) as f64;
    let index = (place as usize).min(colors.len() - 2);
    let share = place - index as f64;
    let (from, to) = (colors[index], colors[index + 1]);
    let mix = |a: u8, b: u8| (a as f64 + (b as f64 - a as f64) * share).round() as u32;
    HexColor::new(
        (mix(from.r(), to.r()) << 16) | (mix(from.g(), to.g()) << 8) | mix(from.b(), to.b()),
    )
}

/// The heatmap's SVG grown by its scale: a bar under the plot from the lowest value's colour
/// to the highest's, in classes with `steps`, its ends' values (and the classes') below it.
pub(super) fn with_scale(
    svg: &str,
    spec: &HeatmapSpec,
    plot: Plot,
    top: f32,
    write: impl Fn(f64) -> String,
) -> String {
    let (low, high) = spec.range;
    let bar_top = top + 6.0;
    let label = |x: f32, y: f32, anchor: &str, text: &str| {
        format!(
            "<text x=\"{x}\" y=\"{y}\" text-anchor=\"{anchor}\" font-size=\"12\" font-family=\"inherit\" fill=\"{}\">{}</text>",
            spec.ink.label,
            escape(text)
        )
    };
    let mut scale = String::from("<g>");
    // The empty cell's key at the end: an outline as the cells', its name after it.
    let key = spec.no_value.as_ref().map_or(0.0, |name| {
        let room = SCALE_BAR + 6.0 + text_width(&[("font-size", "12")], name);
        let x = plot.right - room;
        scale.push_str(&format!(
            "<rect x=\"{x}\" y=\"{bar_top}\" width=\"{SCALE_BAR}\" height=\"{SCALE_BAR}\" fill=\"none\" stroke=\"{}\" stroke-dasharray=\"{EMPTY_DASH}\"/>",
            spec.ink.axis
        ));
        scale.push_str(&label(x + SCALE_BAR + 6.0, bar_top + SCALE_BAR, "start", name));
        room + 16.0
    });
    let (left, width) = (plot.left, (plot.right - plot.left - key).max(0.0));
    let pieces = match spec.steps >= 2 {
        true => spec.steps,
        false => SCALE_PIECES,
    };
    let piece = width / pieces as f32;
    for i in 0..pieces {
        let position = match spec.steps >= 2 {
            true => i as f64 / (pieces - 1) as f64,
            false => (i as f64 + 0.5) / pieces as f64,
        };
        // A hair wider, so no seam shows between two pieces.
        scale.push_str(&format!(
            "<rect x=\"{}\" y=\"{bar_top}\" width=\"{}\" height=\"{SCALE_BAR}\" fill=\"{}\"/>",
            left + piece * i as f32,
            piece + 0.5,
            gradient(&spec.colors, position)
        ));
    }
    let text = |x: f32, anchor: &str, value: f64| {
        label(x, bar_top + SCALE_BAR + 14.0, anchor, &write(value))
    };
    scale.push_str(&text(left, "start", low));
    scale.push_str(&text(left + width, "end", high));
    if spec.steps >= 2 {
        let fits = (1..spec.steps).all(|i| {
            let value = low + (high - low) * i as f64 / spec.steps as f64;
            text_width(&[("font-size", "12")], &write(value)) < piece - 2.0 * EDGE
        });
        for i in (1..spec.steps).filter(|_| fits) {
            let value = low + (high - low) * i as f64 / spec.steps as f64;
            scale.push_str(&text(left + piece * i as f32, "middle", value));
        }
    }
    scale.push_str("</g>");
    appended(svg, spec.width, spec.height, &scale)
}

/// The renderer's y axis in our ink without its ticks (they fall between no rows), cells
/// parted by the surface, an empty one outlined in the grid's ink instead of a stripe.
pub(super) fn restyle_cells(svg: &str, ink: &ChartInk) -> String {
    const RENDERER_AXIS: &str = "#6E7079";
    let svg = svg
        .replace(
            &format!("stroke=\"{RENDERER_AXIS}\""),
            &format!("stroke=\"{}\"", ink.axis),
        )
        .replace(
            &format!("fill=\"{RENDERER_AXIS}\""),
            &format!("fill=\"{}\"", ink.label),
        );
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg.as_str();
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find('>')
            .map_or(rest.len(), |end| start + end + 1);
        let tag = &rest[start..end];
        rest = &rest[end..];
        let attributes = attributes(tag);
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        match tag_name(tag) {
            "line" => {
                let tick = matches!(
                    (number("x1"), number("x2"), number("y1"), number("y2")),
                    (Some(x1), Some(x2), Some(y1), Some(y2)) if y1 == y2 && (x2 - x1).abs() <= 5.0
                );
                if !tick {
                    out.push_str(tag);
                }
            }
            "rect" if attribute(&attributes, "data-index").is_some() => {
                let kept: String = ["x", "y", "width", "height"]
                    .iter()
                    .filter_map(|name| {
                        Some(format!(" {name}=\"{}\"", attribute(&attributes, name)?))
                    })
                    .collect();
                out.push_str(&format!(
                    "<rect{kept} fill=\"none\" stroke=\"{}\" stroke-dasharray=\"{EMPTY_DASH}\"/>",
                    ink.axis
                ));
            }
            "rect" => {
                let kept: String = attributes
                    .iter()
                    .filter(|(attribute, _)| *attribute != "stroke")
                    .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
                    .collect();
                out.push_str(&format!("<rect{kept} stroke=\"{}\"/>", ink.surface));
            }
            _ => out.push_str(tag),
        }
    }
    out.push_str(rest);
    out
}

/// A cell's label written by `write` at its middle, in the renderer's order of `marks`;
/// left out unless `show`, or when wider or taller than its cell.
pub(super) fn write_cell_labels(
    svg: &str,
    marks: &[Mark],
    show: bool,
    write: impl Fn(f64) -> String,
) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    let mut cells = marks.iter();
    while let Some(start) = rest.find("<text ") {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find('>')
            .map_or(rest.len(), |end| start + end + 1);
        let tag = &rest[start..end];
        let content = &rest[end..rest[end..].find('<').map_or(rest.len(), |at| end + at)];
        if !content.trim_start().starts_with(LABEL_MARK) {
            out.push_str(tag);
            rest = &rest[end..];
            continue;
        }
        rest = rest[end..]
            .find("</text>")
            .map_or("", |close| &rest[end + close + 7..]);
        let Some(Mark {
            value,
            shape:
                MarkShape::Rect {
                    x,
                    y,
                    width,
                    height,
                },
            ..
        }) = cells.next()
        else {
            continue;
        };
        let attributes = attributes(tag);
        let size = attribute(&attributes, "font-size")
            .and_then(|size| size.parse::<f32>().ok())
            .unwrap_or(14.0);
        let label = write(*value);
        if !show || text_width(&attributes, &label) > width - 2.0 * EDGE || size > *height {
            continue;
        }
        let kept: String = attributes
            .iter()
            .filter(|(attribute, _)| !matches!(*attribute, "x" | "y" | "dominant-baseline"))
            .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
            .collect();
        out.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\"{kept}>{}</text>",
            x + width / 2.0,
            y + height / 2.0 + size * 0.35,
            escape(&label)
        ));
    }
    out.push_str(rest);
    out
}
