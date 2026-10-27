use dioxus::prelude::*;

use super::{
    RadarAxis, Series,
    engine::{self, RadarSpec, RadarSpecSeries},
    export::ExportSource,
    frame::{Frame, Interaction, chart_unit, hex, is_named, use_plot_size},
    legend::use_hidden_series,
    overlay::{Layout, OverlaySeries},
    props::chart_props,
};
use crate::{
    components::common::{Input, use_name_warning},
    hooks::{use_accessibility, use_formats, use_localization, use_root_id, use_theme},
    localization::{ChartLabels, fill},
    theme::{ChartInk, PointSymbol, Theme, chart_palette},
    utils::warn,
};

chart_props! {
    pub struct RadarChartProps {
        /// The spokes, clockwise from the top; at least three.
        axes: Vec<RadarAxis>,
        /// One value per axis each, in the axes' order; a missing value is drawn at the centre.
        series: Vec<Series>,
        /// Each series' shape filled, translucent, inside its outline. Unset, the theme's.
        #[props(default)]
        filled: Option<bool>,
        /// The web's rings, its steps from the centre to the rim. Unset, the theme's.
        #[props(default)]
        rings: Option<usize>,
    }
}

/// Several measures of each series on spokes round a centre: how their profiles compare.
/// Each axis runs from 0 at the centre to its `max` at the rim.
///
/// Screen readers get the title and a summary (the axes, each series' highest and lowest).
/// The left and right keys walk a series round the axes, up and down move to the next
/// series on the same axis. Web and Android; enable the `charts` feature.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{RadarAxis, RadarChart, Series};
/// # fn app() -> Element {
/// rsx! {
///     RadarChart {
///         title: "Phones",
///         axes: vec![
///             RadarAxis::from(("Battery", 10.0)),
///             ("Camera", 10.0).into(),
///             ("Screen", 10.0).into(),
///             ("Price", 10.0).into(),
///         ],
///         series: vec![
///             Series::new("Model A", [8.0, 6.0, 7.0, 5.0]),
///             Series::new("Model B", [6.0, 9.0, 8.0, 3.0]),
///         ],
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/chart>
#[component]
pub fn RadarChart(props: RadarChartProps) -> Element {
    let theme = use_theme();
    let labels = use_localization().chart;
    let formats = use_formats();
    let id = use_root_id(&props.attributes);
    use_name_warning(
        is_named(&props.title, &props.attributes),
        "RadarChart: no `title` - name the chart: it is how a screen reader finds it.",
    );
    let count = props.axes.len();
    let uneven = props.series.iter().any(|s| s.values.len() != count);
    use_hook(move || {
        if count < 3 {
            warn(
                "RadarChart: fewer than three `axes` draw no web; the chart shows its empty state.",
            );
        }
        if uneven {
            warn(
                "RadarChart: a series has more or fewer values than `axes`: the missing ones are drawn at the centre, the extra ones left out.",
            );
        }
    });

    let ratio = props
        .aspect_ratio
        .unwrap_or(theme.chart.pie_aspect_ratio)
        .max(0.1);
    let size = use_plot_size();
    let (width, height) = size.get(ratio);

    // Padded or cut to one value per axis.
    let series: Vec<Series> = props
        .series
        .iter()
        .map(|series| {
            let mut series = series.clone();
            series.values.resize(count, None);
            for value in &mut series.values {
                *value = value.filter(|v| v.is_finite());
            }
            series
        })
        .collect();
    let empty = count < 3 || series.iter().all(|s| s.values.iter().all(Option::is_none));
    let axes: Vec<(String, f64)> = props
        .axes
        .iter()
        .enumerate()
        .map(|(i, axis)| {
            let largest = series
                .iter()
                .filter_map(|s| s.values[i])
                .fold(0.0, f64::max);
            let max = axis.max.filter(|max| max.is_finite() && *max > 0.0);
            (axis.name.clone(), max.unwrap_or_else(|| round_up(largest)))
        })
        .collect();

    let forced = use_accessibility().forced_colors();
    let all_specs = specs(theme, forced, &series);
    let names: Vec<String> = all_specs.iter().map(|s| s.name.clone()).collect();
    let hidden = use_hidden_series(
        &props.hidden,
        &props.default_hidden,
        props.onhiddenchange,
        &names,
    );
    let shown: Vec<usize> = (0..all_specs.len())
        .filter(|i| !hidden.get().contains(&all_specs[*i].name))
        .collect();
    let unit = chart_unit(&props.unit, &series);
    let spec = RadarSpec {
        width,
        height,
        axes: axes.clone(),
        series: shown.iter().map(|i| all_specs[*i].clone()).collect(),
        filled: props.filled.unwrap_or(theme.chart.radar_filled),
        rings: props.rings.unwrap_or(theme.chart.radar_rings),
        grid: props.grid.unwrap_or(theme.chart.grid),
        radius: theme.chart.stroke_width + 2.0,
        ink: ChartInk::of(theme, forced),
    };
    // The file keeps the normal palette under forced colours.
    let export = {
        let specs = specs(theme, false, &series);
        let spec = RadarSpec {
            series: shown.iter().map(|i| specs[*i].clone()).collect(),
            ink: ChartInk::of(theme, false),
            ..spec.clone()
        };
        ExportSource::of(theme, props.title.clone(), move || {
            engine::draw_radar(&spec)
        })
    };
    let drawn = use_memo(use_reactive!(|spec, shown| {
        engine::draw_radar(&spec).map(|drawn| drawn.with_series(&shown))
    }));

    let format = props.value_format;
    let write = |series: &Series, value: f64| {
        let unit = series.unit.as_deref().or(unit.as_deref());
        format.format(value, unit, formats)
    };
    let overlay: Vec<OverlaySeries> = series
        .iter()
        .zip(&all_specs)
        .map(|(series, spec)| OverlaySeries {
            name: spec.name.clone(),
            color: spec.color.to_string(),
            dashed: false,
            symbol: Some(spec.symbol),
            line: false,
            unit: series.unit.clone(),
            errors: Vec::new(),
            xs: Vec::new(),
            points: Vec::new(),
        })
        .collect();
    let x: Vec<String> = axes.iter().map(|(name, _)| name.clone()).collect();

    let summary = match (props.loading, empty) {
        (true, _) => vec![labels.loading.to_string()],
        (false, true) => Vec::new(),
        (false, false) => summary(&labels, &x, &series, write),
    };

    Frame {
        id,
        labels,
        title: props.title.clone(),
        description: props.description.clone(),
        loading: props.loading,
        empty,
        empty_slot: props.empty.clone(),
        drawn,
        center: None,
        legend: Frame::legend(&props.legend, theme, props.series.len()),
        legend_label: labels.legend,
        series: overlay,
        hidden,
        x,
        summary,
        plot: size,
        aspect_ratio: Some(ratio),
        interaction: Interaction {
            format,
            unit,
            formats,
            tooltip: props.tooltip.unwrap_or(theme.chart.tooltip),
            tooltip_content: props.tooltip_content,
            onpointclick: props.onpointclick,
            onactivechange: props.onactivechange,
            swap_axes: false,
            layout: Layout::Spokes,
            onrange: None,
        },
        export,
        handle: props.handle,
        zoom: None,
    }
    .render(
        &props.class,
        &props.sx,
        &props.parts,
        &props.states,
        props.attributes,
    )
}

/// Each series' values, colour and shape: its own, else circle, triangle, square and
/// diamond by its place.
fn specs(theme: &Theme, forced: bool, series: &[Series]) -> Vec<RadarSpecSeries> {
    const SHAPES: [PointSymbol; 4] = [
        PointSymbol::Circle,
        PointSymbol::Triangle,
        PointSymbol::Square,
        PointSymbol::Diamond,
    ];
    let palette = chart_palette(theme, forced);
    series
        .iter()
        .enumerate()
        .map(|(i, series)| RadarSpecSeries {
            name: series.name.clone(),
            values: series.values.clone(),
            color: hex(theme, forced, &palette, i, series.color),
            symbol: series.symbol.unwrap_or(SHAPES[i % SHAPES.len()]),
        })
        .collect()
}

/// `value` rounded up to 1, 2, 2.5 or 5 times a power of ten; 1 for nothing above 0.
fn round_up(value: f64) -> f64 {
    if value <= 0.0 || !value.is_finite() {
        return 1.0;
    }
    let power = 10f64.powf(value.log10().floor());
    [1.0, 2.0, 2.5, 5.0, 10.0]
        .into_iter()
        .map(|step| step * power)
        .find(|rim| *rim >= value)
        .unwrap_or(10.0 * power)
}

/// The axes, then each series' highest and lowest axis and those it has no value on.
fn summary(
    labels: &ChartLabels,
    axes: &[String],
    series: &[Series],
    write: impl Fn(&Series, f64) -> String,
) -> Vec<String> {
    let mut sentences = vec![fill(
        labels.radar_axes,
        &[
            ("kind", &labels.radar),
            ("count", &axes.len().to_string()),
            ("names", &axes.join(", ")),
        ],
    )];
    for one in series {
        let values = one
            .values
            .iter()
            .enumerate()
            .filter_map(|(i, value)| value.map(|value| (i, value)));
        // The first of equal values.
        let high = values.clone().reduce(|a, b| if b.1 > a.1 { b } else { a });
        let low = values.reduce(|a, b| if b.1 < a.1 { b } else { a });
        let (Some((high, max)), Some((low, min))) = (high, low) else {
            sentences.push(fill(labels.series_empty, &[("name", &one.name)]));
            continue;
        };
        sentences.push(fill(
            labels.radar_series,
            &[
                ("name", &one.name),
                ("high", &axes[high]),
                ("max", &write(one, max)),
                ("low", &axes[low]),
                ("min", &write(one, min)),
            ],
        ));
        let missing: Vec<&str> = (0..axes.len())
            .filter(|i| one.values[*i].is_none())
            .map(|i| axes[i].as_str())
            .collect();
        if !missing.is_empty() {
            sentences.push(fill(
                labels.radar_missing,
                &[("name", &one.name), ("axes", &missing.join(", "))],
            ));
        }
    }
    sentences
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LiberoProvider;

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    /// A named figure, its web, a summary of the axes and each series' highest and lowest,
    /// and a legend whose keys show each series' shape.
    #[test]
    fn the_first_render_is_a_named_figure_with_its_summary() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    RadarChart {
                        title: "Phones",
                        axes: vec!["Battery".into(), "Camera".into(), ("Screen", 10.0).into()],
                        series: vec![
                            Series::new("A", [8.0, 6.0, 7.0]),
                            Series::new("B", [Some(6.0), None, Some(9.0)]),
                        ],
                    }
                }
            }
        });
        assert!(html.contains("role=\"figure\""), "{html}");
        assert!(html.contains("<svg"), "{html}");
        assert!(
            html.contains("Radar chart over 3 axes: Battery, Camera, Screen."),
            "{html}"
        );
        assert!(
            html.contains("A: highest on Battery, 8; lowest on Camera, 6."),
            "{html}"
        );
        assert!(
            html.contains("B: no value on Camera, drawn at the centre."),
            "{html}"
        );
        assert!(html.contains("role=\"application\""), "{html}");
        assert_eq!(html.matches("aria-pressed=\"true\"").count(), 2, "{html}");
    }

    /// Under three axes there is no web: the empty state.
    #[test]
    fn two_axes_are_empty() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    RadarChart {
                        title: "Two",
                        axes: vec!["A".into(), "B".into()],
                        series: vec![Series::new("S", [1.0, 2.0])],
                    }
                }
            }
        });
        assert!(html.contains("No data"), "{html}");
        assert!(!html.contains("<svg"), "{html}");
    }

    #[test]
    fn an_unset_rim_is_a_round_number_over_the_largest_value() {
        assert_eq!(round_up(7.0), 10.0);
        assert_eq!(round_up(18.0), 20.0);
        assert_eq!(round_up(240.0), 250.0);
        assert_eq!(round_up(500.0), 500.0);
        assert_eq!(round_up(0.0), 1.0);
    }
}
