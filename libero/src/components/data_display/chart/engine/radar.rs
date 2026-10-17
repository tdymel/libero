//! A radar chart's spec and drawing, its spokes and markers.

use super::*;

/// A radar series as drawn: one value per axis, `None` where it has none.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RadarSpecSeries {
    pub(crate) name: String,
    pub(crate) values: Vec<Option<f64>>,
    pub(crate) color: HexColor,
    pub(crate) symbol: PointSymbol,
}

/// Everything a radar chart's SVG depends on.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RadarSpec {
    pub(crate) width: f32,
    pub(crate) height: f32,
    /// Each axis' name and the value at its rim, above 0.
    pub(crate) axes: Vec<(String, f64)>,
    pub(crate) series: Vec<RadarSpecSeries>,
    pub(crate) filled: bool,
    pub(crate) rings: usize,
    pub(crate) grid: bool,
    /// A point's radius.
    pub(crate) radius: f32,
    pub(crate) ink: ChartInk,
}

/// The renderer's room between the web and the SVG's shorter side, for the axis names.
pub(super) const RADAR_ROOM: f32 = 40.0;
/// Between a spoke's end and its name.
pub(super) const RADAR_LABEL_GAP: f32 = 6.0;
pub(super) const RADAR_FONT: f32 = 14.0;

/// The point `reach` out from the centre on spoke `axis`, the spokes clockwise from the top.
pub(super) fn radar_point(spec: &RadarSpec, axis: usize, reach: f32) -> (f32, f32) {
    let angle = std::f32::consts::TAU * axis as f32 / spec.axes.len() as f32;
    let (cx, cy) = (spec.width / 2.0, spec.height / 2.0);
    (cx + reach * angle.sin(), cy - reach * angle.cos())
}

/// A point's marker in its shape, each covering about a circle's area, edged in the surface.
pub(super) fn radar_marker(
    symbol: PointSymbol,
    (x, y): (f32, f32),
    r: f32,
    color: HexColor,
    edge: HexColor,
) -> String {
    let paint = format!("fill=\"{color}\" stroke=\"{edge}\" stroke-width=\"1\"");
    let corners = |points: &[(f32, f32)]| {
        let points: Vec<String> = points.iter().map(|(px, py)| format!("{px},{py}")).collect();
        format!("<polygon points=\"{}\" {paint}/>", points.join(" "))
    };
    match symbol {
        PointSymbol::Circle => format!("<circle cx=\"{x}\" cy=\"{y}\" r=\"{r}\" {paint}/>"),
        PointSymbol::Square => {
            let s = r * 0.9;
            format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" {paint}/>",
                x - s,
                y - s,
                2.0 * s,
                2.0 * s
            )
        }
        PointSymbol::Triangle => {
            let s = r * 1.35;
            corners(&[
                (x, y - s),
                (x + s * 0.866, y + s / 2.0),
                (x - s * 0.866, y + s / 2.0),
            ])
        }
        PointSymbol::Diamond => {
            let s = r * 1.25;
            corners(&[(x, y - s), (x + s, y), (x, y + s), (x - s, y)])
        }
    }
}

pub(crate) fn draw_radar(spec: &RadarSpec) -> Option<Drawn> {
    let count = spec.axes.len();
    if count < 3 {
        return None;
    }
    // A missing value sits at the centre, as the renderer draws it.
    let series = spec
        .series
        .iter()
        .map(|series| {
            let values = (0..count)
                .map(|i| series.values.get(i).copied().flatten().map(|v| v as f32))
                .collect();
            charts_rs::Series::new_nullable(series.name.clone(), values)
        })
        .collect();
    let indicators = spec
        .axes
        .iter()
        .map(|(name, max)| charts_rs::RadarIndicator {
            name: name.clone(),
            max: *max as f32,
        })
        .collect();
    let mut chart = charts_rs::RadarChart::new(series, indicators);
    chart.width = spec.width;
    chart.height = spec.height;
    chart.background_color = InkColor::transparent();
    chart.title.text = String::new();
    chart.legend.show = Some(false);
    chart.animation = None;
    chart.margin = Default::default();
    chart.series.colors = spec.series.iter().map(|s| ink(s.color)).collect();
    chart.series.stroke_width = 2.0;
    chart.series.fill = spec.filled;
    chart.split_number = spec.rings.max(1);
    chart.grid.stroke_color = match spec.grid {
        true => ink(spec.ink.grid),
        false => InkColor::transparent(),
    };
    let svg = chart.svg().ok()?;
    // Its own names are placed in its font and its circles all alike: both drawn anew.
    let mut kept = String::with_capacity(svg.len());
    let mut rest = svg.as_str();
    while let Some(open) = rest.find('<') {
        kept.push_str(&rest[..open]);
        let tag = &rest[open..];
        let end = match tag_name(tag) {
            "text" => tag.find("</text>").map(|end| end + "</text>".len()),
            "circle" => tag.find('>').map(|end| end + 1),
            _ => None,
        };
        match end {
            Some(end) => rest = &tag[end..],
            None => {
                let close = tag.find('>').map_or(tag.len(), |close| close + 1);
                kept.push_str(&tag[..close]);
                rest = &tag[close..];
            }
        }
    }
    kept.push_str(rest);
    let (svg, _) = inert(&kept, |value| value.to_string(), "", false);

    let reach = (spec.width.min(spec.height) / 2.0 - RADAR_ROOM).max(1.0);
    let mut names = String::new();
    for (axis, (name, _)) in spec.axes.iter().enumerate() {
        let (x, y) = radar_point(spec, axis, reach + RADAR_LABEL_GAP);
        let angle = std::f32::consts::TAU * axis as f32 / count as f32;
        let (sin, cos) = (angle.sin(), angle.cos());
        let anchor = match sin {
            s if s.abs() < 0.1 => "middle",
            s if s > 0.0 => "start",
            _ => "end",
        };
        // Above the top spoke, below the bottom one, centred on the others.
        let y = match cos {
            c if c > 0.9 => y,
            c if c < -0.9 => y + RADAR_FONT * 0.8,
            _ => y + RADAR_FONT * 0.35,
        };
        names.push_str(&format!(
            "<text x=\"{x}\" y=\"{y}\" text-anchor=\"{anchor}\" font-size=\"{RADAR_FONT}\" font-family=\"inherit\" fill=\"{}\">\n{}\n</text>",
            spec.ink.label,
            escape(name)
        ));
    }
    let mut marks = Vec::new();
    let mut markers = String::new();
    for (s, series) in spec.series.iter().enumerate() {
        for (axis, (_, max)) in spec.axes.iter().enumerate() {
            let Some(value) = series.values.get(axis).copied().flatten() else {
                continue;
            };
            let share = (value / max).clamp(0.0, 1.0) as f32;
            let at = radar_point(spec, axis, reach * share);
            markers.push_str(&radar_marker(
                series.symbol,
                at,
                spec.radius,
                series.color,
                spec.ink.surface,
            ));
            marks.push(Mark {
                series: s,
                index: axis,
                value,
                shape: MarkShape::Point { x: at.0, y: at.1 },
            });
        }
    }
    let end = svg.rfind("</svg>").unwrap_or(svg.len());
    let svg = format!("{}{names}{markers}{}", &svg[..end], &svg[end..]);
    let svg = cut_labels(&svg, spec.width);
    let (cx, cy) = (spec.width / 2.0, spec.height / 2.0);
    Some(Drawn {
        svg: spec.ink.paint(svg),
        geometry: Geometry {
            width: spec.width,
            height: spec.height,
            plot: Plot {
                left: cx - reach,
                top: cy - reach,
                right: cx + reach,
                bottom: cy + reach,
            },
            marks,
            cells: false,
            hole: 0.0,
            share_of: None,
        },
    })
}
