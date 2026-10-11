//! Props to `charts-rs` and back: it draws the static SVG, we harvest where each mark
//! landed and strip its own tooltips, so the overlay and keyboard layers can be ours.

use charts_rs::{Color as InkColor, LineStep, Symbol};

use super::data::ValueFormat;
use crate::theme::{ChartInk, Curve, HexColor};

/// Ahead of each y-axis label, so [`sanitize`] can write it again in the locale.
const AXIS_MARK: char = '\u{2060}';
const DASH: &str = "6,4";

/// Where every mark of a drawn chart sits, in the SVG's own units.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Geometry {
    pub(crate) width: f32,
    pub(crate) height: f32,
    /// Series by series, in data order; a gap has no mark.
    pub(crate) marks: Vec<Mark>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Mark {
    pub(crate) series: usize,
    pub(crate) index: usize,
    pub(crate) value: f64,
    pub(crate) shape: MarkShape,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MarkShape {
    /// A line's point.
    Point { x: f32, y: f32 },
    /// A bar.
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    /// A pie slice, as an SVG path.
    Sector { path: String },
}

/// A drawn chart: the SVG to show and where its marks are.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Drawn {
    pub(crate) svg: String,
    pub(crate) geometry: Geometry,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SeriesSpec {
    pub(crate) name: String,
    pub(crate) values: Vec<Option<f64>>,
    pub(crate) color: HexColor,
    pub(crate) dashed: bool,
}

/// Everything a line chart's SVG depends on, so a memo redraws only on a change.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LineSpec {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) x: Vec<String>,
    pub(crate) series: Vec<SeriesSpec>,
    pub(crate) curve: Curve,
    pub(crate) grid: bool,
    pub(crate) points: bool,
    pub(crate) stroke_width: f32,
    pub(crate) ink: ChartInk,
    pub(crate) format: ValueFormat,
    pub(crate) unit: Option<String>,
    pub(crate) decimal: &'static str,
}

fn ink(hex: HexColor) -> InkColor {
    InkColor::from((hex.r(), hex.g(), hex.b()))
}

pub(crate) fn draw_line(spec: &LineSpec) -> Option<Drawn> {
    let series = spec
        .series
        .iter()
        .map(|series| {
            let values = series.values.iter().map(|v| v.map(|v| v as f32)).collect();
            let mut drawn = charts_rs::Series::new_nullable(series.name.clone(), values);
            drawn.colors = Some(vec![Some(ink(series.color))]);
            drawn.stroke_dash_array = series.dashed.then(|| DASH.to_string());
            drawn.symbol = Some(match spec.points {
                true => Symbol::Circle(spec.stroke_width + 1.0, Some(ink(series.color))),
                false => Symbol::None,
            });
            match spec.curve {
                Curve::Linear => {}
                Curve::Smooth => drawn.smooth = Some(true),
                Curve::Step => drawn.step = Some(LineStep::End),
            }
            drawn
        })
        .collect();
    let mut chart = charts_rs::LineChart::new(series, spec.x.clone());
    chart.width = spec.width;
    chart.height = spec.height;
    chart.background_color = InkColor::transparent();
    chart.title.text = String::new();
    chart.legend.show = Some(false);
    // Its triggers are the marks we harvest; `sanitize` drops its tips.
    chart.tooltip.show = true;
    chart.animation = None;
    chart.series.stroke_width = spec.stroke_width;
    chart.series.colors = spec.series.iter().map(|s| ink(s.color)).collect();
    chart.grid.stroke_color = match spec.grid {
        true => ink(spec.ink.grid),
        false => InkColor::transparent(),
    };
    chart.x_axis.stroke_color = ink(spec.ink.axis);
    chart.x_axis.font.color = ink(spec.ink.label);
    let unit = spec
        .unit
        .as_deref()
        .map(|unit| format!(" {unit}"))
        .unwrap_or_default();
    if let Some(y) = chart.y_axis_configs.first_mut() {
        y.stroke_color = ink(spec.ink.axis);
        y.font.color = ink(spec.ink.label);
        y.formatter = Some(format!("{AXIS_MARK}{{c}}{unit}"));
    }
    let svg = chart.svg().ok()?;

    let expected: Vec<_> = expected_marks(&spec.series);
    let (svg, triggers) = sanitize(&svg, |label| {
        spec.format
            .format(label, spec.unit.as_deref(), spec.decimal)
    });
    let marks = match_marks(&expected, &triggers);
    Some(Drawn {
        svg,
        geometry: Geometry {
            width: spec.width,
            height: spec.height,
            marks,
        },
    })
}

/// `(series, index, value, name)` of every mark the renderer draws, in its order.
fn expected_marks(series: &[SeriesSpec]) -> Vec<(usize, usize, f64, &str)> {
    series
        .iter()
        .enumerate()
        .flat_map(|(s, series)| {
            series
                .values
                .iter()
                .enumerate()
                .filter_map(move |(i, value)| {
                    value.map(|value| (s, i, value, series.name.as_str()))
                })
        })
        .collect()
}

/// Pairs the renderer's triggers with the marks we expect; on any mismatch, no marks
/// rather than wrong ones.
fn match_marks(expected: &[(usize, usize, f64, &str)], triggers: &[Trigger]) -> Vec<Mark> {
    let matches = expected.len() == triggers.len()
        && expected
            .iter()
            .zip(triggers)
            .all(|((.., name), trigger)| trigger.series == *name);
    if !matches {
        return Vec::new();
    }
    expected
        .iter()
        .zip(triggers)
        .map(|(&(series, index, value, _), trigger)| Mark {
            series,
            index,
            value,
            shape: trigger.shape.clone(),
        })
        .collect()
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
            if name != "circle" {
                out.push_str(&self_closing(name, &attributes));
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
    (relabel(&out, axis), triggers)
}

fn tag_name(tag: &str) -> &str {
    let inner = tag.trim_start_matches('<');
    let end = inner
        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .unwrap_or(inner.len());
    &inner[..end]
}

/// A tag's attributes, values still escaped.
fn attributes(tag: &str) -> Vec<(&str, &str)> {
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

fn attribute<'a>(attributes: &[(&str, &'a str)], name: &str) -> Option<&'a str> {
    attributes
        .iter()
        .find(|(attribute, _)| *attribute == name)
        .map(|(_, value)| *value)
}

fn trigger(name: &str, attributes: &[(&str, &str)]) -> Option<Trigger> {
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
        _ => return None,
    };
    Some(Trigger {
        series: unescape(attribute(attributes, "data-series")?),
        shape,
    })
}

/// A mark without the renderer's hover hooks.
fn self_closing(name: &str, attributes: &[(&str, &str)]) -> String {
    let kept: String = attributes
        .iter()
        .filter(|(attribute, _)| *attribute != "class" && !attribute.starts_with("data-"))
        .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
        .collect();
    format!("<{name}{kept}/>")
}

/// The root at its container's size, out of the accessibility tree and the tab order.
fn root(attributes: &[(&str, &str)]) -> String {
    let kept: String = attributes
        .iter()
        .filter(|(attribute, _)| !matches!(*attribute, "width" | "height"))
        .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
        .collect();
    format!("<svg width=\"100%\" height=\"100%\" aria-hidden=\"true\" focusable=\"false\"{kept}>")
}

/// Writes every marked y label again with `axis`.
fn relabel(svg: &str, axis: impl Fn(f64) -> String) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(mark) = rest.find(AXIS_MARK) {
        out.push_str(&rest[..mark]);
        let label = &rest[mark + AXIS_MARK.len_utf8()..];
        let end = label.find(['\n', '<']).unwrap_or(label.len());
        let number = label[..end].split(' ').next().unwrap_or_default();
        match number.parse::<f64>() {
            Ok(value) => out.push_str(&escape(&axis(value))),
            Err(_) => out.push_str(&label[..end]),
        }
        rest = &label[end..];
    }
    out.push_str(rest);
    out
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Color, ColorShade, Theme, chart_hex};

    fn line(series: Vec<SeriesSpec>, x: &[&str]) -> LineSpec {
        LineSpec {
            width: 320.0,
            height: 180.0,
            x: x.iter().map(|x| x.to_string()).collect(),
            series,
            curve: Curve::Linear,
            grid: true,
            points: true,
            stroke_width: 2.0,
            ink: ChartInk::of(&Theme::DEFAULT),
            format: ValueFormat::Auto,
            unit: Some("kg".to_string()),
            decimal: ",",
        }
    }

    fn spec(name: &str, values: &[Option<f64>]) -> SeriesSpec {
        SeriesSpec {
            name: name.to_string(),
            values: values.to_vec(),
            color: chart_hex(&Theme::DEFAULT, Color::Primary, ColorShade::S6),
            dashed: false,
        }
    }

    /// Contract with the pinned `charts-rs`: a line's points come back in data order,
    /// gaps skipped, and the SVG keeps none of its own tooltips.
    #[test]
    fn a_line_s_points_are_harvested_and_its_tooltips_dropped() {
        let drawn = draw_line(&line(
            vec![
                spec("A & B", &[Some(1.0), None, Some(3.0)]),
                spec("C", &[Some(2.0), Some(2.5), Some(1.0)]),
            ],
            &["Mon", "Tue", "Wed"],
        ))
        .expect("drawn");
        let marks = &drawn.geometry.marks;

        let places: Vec<_> = marks.iter().map(|m| (m.series, m.index)).collect();
        assert_eq!(places, [(0, 0), (0, 2), (1, 0), (1, 1), (1, 2)]);
        let ys: Vec<_> = marks
            .iter()
            .map(|m| match m.shape {
                MarkShape::Point { y, .. } => y,
                _ => panic!("{m:?}"),
            })
            .collect();
        // Higher values sit higher up.
        assert!(ys[1] < ys[0] && ys[3] < ys[2], "{ys:?}");
        for gone in [
            "ct-trigger",
            "ct-tip",
            "<title",
            "<style",
            "Roboto",
            "\u{2060}",
        ] {
            assert!(!drawn.svg.contains(gone), "{gone}: {}", drawn.svg);
        }
        assert!(
            drawn
                .svg
                .starts_with("<svg width=\"100%\" height=\"100%\" aria-hidden=\"true\"")
        );
        assert!(drawn.svg.contains("2,5 kg"), "{}", drawn.svg);
    }

    /// The series' hex is the one drawn: no `var()` reaches the renderer.
    #[test]
    fn a_line_is_drawn_in_its_hex() {
        let drawn =
            draw_line(&line(vec![spec("A", &[Some(1.0), Some(2.0)])], &["a", "b"])).expect("drawn");
        let hex = chart_hex(&Theme::DEFAULT, Color::Primary, ColorShade::S6).to_string();
        assert!(
            drawn.svg.to_uppercase().contains(&hex.to_uppercase()),
            "{hex}: {}",
            drawn.svg
        );
    }

    #[test]
    fn a_mismatch_harvests_no_marks() {
        let triggers = [Trigger {
            series: "B".to_string(),
            shape: MarkShape::Point { x: 0.0, y: 0.0 },
        }];
        assert!(match_marks(&[(0, 0, 1.0, "A")], &triggers).is_empty());
    }

    /// Contract: a bar is a trigger `rect`, kept as the mark, its hooks stripped.
    #[test]
    fn a_bar_is_harvested_as_its_rect() {
        let mut chart = charts_rs::BarChart::new(
            vec![charts_rs::Series::new("A".to_string(), vec![1.0, 2.0])],
            vec!["x".to_string(), "y".to_string()],
        );
        chart.tooltip.show = true;
        chart.background_color = InkColor::transparent();
        let (svg, triggers) = sanitize(&chart.svg().expect("svg"), |v| v.to_string());

        assert_eq!(triggers.len(), 2);
        assert!(
            triggers
                .iter()
                .all(|t| matches!(t.shape, MarkShape::Rect { height, .. } if height > 0.0))
        );
        assert_eq!(svg.matches("<rect").count(), 2, "{svg}");
        assert!(
            !svg.contains("ct-trigger") && !svg.contains("<title"),
            "{svg}"
        );
    }

    /// Contract: a slice is a trigger `path`, one per series.
    #[test]
    fn a_pie_slice_is_harvested_as_its_path() {
        let mut chart = charts_rs::PieChart::new(vec![
            charts_rs::Series::new("A".to_string(), vec![1.0]),
            charts_rs::Series::new("B".to_string(), vec![3.0]),
        ]);
        chart.tooltip.show = true;
        let (_, triggers) = sanitize(&chart.svg().expect("svg"), |v| v.to_string());

        let names: Vec<_> = triggers.iter().map(|t| t.series.as_str()).collect();
        assert_eq!(names, ["A", "B"]);
        assert!(
            triggers
                .iter()
                .all(|t| matches!(&t.shape, MarkShape::Sector { path } if path.starts_with('M')))
        );
    }
}
