use dioxus::prelude::*;

use super::{
    Slice, ValueFormat,
    engine::{self, FunnelSpec, SeriesSpec},
    export::ExportSource,
    frame::{Frame, Interaction, hex, is_named, overlay_series, shown_places, use_plot_size},
    legend::use_hidden_series,
    overlay::Layout,
    props::chart_props,
};
use crate::{
    components::common::{Input, use_name_warning},
    hooks::{use_accessibility, use_formats, use_localization, use_root_id, use_theme},
    localization::{ChartLabels, fill},
    theme::{ChartInk, PieLabels, chart_palette},
    utils::warn,
};

chart_props! {
    pub struct FunnelChartProps {
        /// The stages, top to bottom, each as wide as its value over the largest. Drawn
        /// widest first: a stage larger than the one before it moves up, with a warning in
        /// development; the keys, summary and legend keep this order.
        stages: Vec<Slice>,
        /// What each stage has written on it; a percent is its share of the first stage.
        /// A label wider than its stage is left out. Unset, the theme's `pie_labels`.
        #[props(default, into)]
        labels: Input<PieLabels>,
    }
}

/// Stages that narrow down, as visits to orders: each drawn as wide as its value.
///
/// Screen readers get the title and a summary (each stage's value and its share of the
/// first). `grid` draws nothing here. Web and Android; enable the `charts` feature.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{FunnelChart, Slice};
/// # fn app() -> Element {
/// rsx! {
///     FunnelChart {
///         title: "Checkout",
///         stages: vec![Slice::new("Visits", 1200.0), Slice::new("Carts", 480.0), Slice::new("Orders", 210.0)],
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/chart>
#[component]
pub fn FunnelChart(props: FunnelChartProps) -> Element {
    let theme = use_theme();
    let labels = use_localization().chart;
    let formats = use_formats();
    let id = use_root_id(&props.attributes);
    use_name_warning(
        is_named(&props.title, &props.attributes),
        "FunnelChart: no `title` - name the chart: it is how a screen reader finds it.",
    );
    let rising = props
        .stages
        .windows(2)
        .find(|pair| width_of(pair[1].value) > width_of(pair[0].value))
        .map(|pair| pair[1].name.clone());
    use_hook(move || {
        if let Some(name) = rising {
            warn(&format!(
                "FunnelChart: stages rise at {name} - drawn widest first."
            ));
        }
    });

    let ratio = props
        .aspect_ratio
        .unwrap_or(theme.chart.pie_aspect_ratio)
        .max(0.1);
    let size = use_plot_size();
    let (width, height) = size.get(ratio);

    let empty = !props.stages.iter().any(|stage| width_of(stage.value) > 0.0);
    let first = props
        .stages
        .first()
        .map_or(0.0, |stage| width_of(stage.value));
    let forced = use_accessibility().forced_colors();
    let specs = |forced: bool| -> Vec<SeriesSpec> {
        let palette = chart_palette(theme, forced);
        props
            .stages
            .iter()
            .enumerate()
            .map(|(i, stage)| SeriesSpec {
                name: stage.name.clone(),
                values: vec![Some(width_of(stage.value))],
                color: hex(theme, forced, &palette, i, stage.color),
                dashed: false,
                extras: Default::default(),
            })
            .collect()
    };
    let all_stages = specs(forced);
    let names: Vec<String> = all_stages.iter().map(|s| s.name.clone()).collect();
    let hidden = use_hidden_series(
        &props.hidden,
        &props.default_hidden,
        props.onhiddenchange,
        &names,
    );
    let shown = shown_places(&all_stages, &hidden.get());
    let spec = FunnelSpec {
        width,
        height,
        stages: shown.iter().map(|i| all_stages[*i].clone()).collect(),
        labels: props.labels.copied_or(theme.chart.pie_labels),
        first,
        ink: ChartInk::of(theme, forced),
        format: props.value_format,
        unit: props.unit.clone(),
        formats,
    };
    // The file keeps the normal palette under forced colours.
    let export = {
        let stages = specs(false);
        let spec = FunnelSpec {
            stages: shown.iter().map(|i| stages[*i].clone()).collect(),
            ink: ChartInk::of(theme, false),
            ..spec.clone()
        };
        ExportSource::of(theme, props.title.clone(), move || {
            engine::draw_funnel(&spec)
        })
    };
    let drawn = use_memo(use_reactive!(|spec, shown| {
        let drawable = spec
            .stages
            .iter()
            .any(|s| s.values[0].is_some_and(|v| v > 0.0));
        drawable
            .then(|| engine::draw_funnel(&spec))
            .flatten()
            .map(|drawn| drawn.with_indices(&shown))
    }));

    let write = |value: f64| {
        props
            .value_format
            .format(value, props.unit.as_deref(), formats)
    };
    let share = |value: f64| ValueFormat::Percent.format(width_of(value) / first, None, formats);
    let summary = match (props.loading, empty) {
        (true, _) => vec![labels.loading.to_string()],
        (false, true) => Vec::new(),
        (false, false) => summary(&labels, &props.stages, first > 0.0, write, share),
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
        legend: Frame::legend(&props.legend, theme, props.stages.len()),
        legend_label: labels.stages,
        series: overlay_series(&all_stages, &[], false),
        hidden,
        x: names,
        summary,
        plot: size,
        aspect_ratio: Some(ratio),
        interaction: Interaction {
            format: props.value_format,
            unit: props.unit.clone(),
            formats,
            tooltip: props.tooltip.unwrap_or(theme.chart.tooltip),
            tooltip_content: props.tooltip_content,
            onpointclick: props.onpointclick,
            onactivechange: props.onactivechange,
            swap_axes: false,
            layout: Layout::Columns,
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

/// A stage's width: a negative or missing value has none.
fn width_of(value: f64) -> f64 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

/// The first and last stage, then every stage's value and its share of the first.
fn summary(
    labels: &ChartLabels,
    stages: &[Slice],
    shares: bool,
    write: impl Fn(f64) -> String,
    share: impl Fn(f64) -> String,
) -> Vec<String> {
    let mut sentences = match stages {
        [only] => vec![fill(
            labels.range_one,
            &[("kind", &labels.funnel), ("first", &only.name)],
        )],
        [first, .., last] => vec![fill(
            labels.range,
            &[
                ("kind", &labels.funnel),
                ("first", &first.name),
                ("last", &last.name),
            ],
        )],
        [] => Vec::new(),
    };
    sentences.extend(stages.iter().map(|stage| {
        if !stage.value.is_finite() {
            return fill(labels.slice_empty, &[("name", &stage.name)]);
        }
        let value = write(stage.value);
        match shares {
            true => fill(
                labels.stage,
                &[
                    ("name", &stage.name),
                    ("value", &value),
                    ("share", &share(stage.value)),
                ],
            ),
            false => fill(
                labels.stage_value,
                &[("name", &stage.name), ("value", &value)],
            ),
        }
    }));
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

    /// The server's HTML has a stage per value, the name and what a screen reader reads.
    #[test]
    fn the_first_render_is_a_named_figure_with_stages_and_a_summary() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    FunnelChart {
                        title: "Checkout",
                        stages: vec![Slice::new("Visits", 200.0), Slice::new("Carts", 50.0), Slice::new("Orders", f64::NAN)],
                    }
                }
            }
        });

        assert!(html.contains("role=\"figure\""), "{html}");
        assert!(html.contains("<polygon"), "{html}");
        assert!(
            html.contains("Funnel chart from Visits to Orders."),
            "{html}"
        );
        assert!(html.contains("Visits: 200, 100% of the first."), "{html}");
        assert!(html.contains("Carts: 50, 25% of the first."), "{html}");
        assert!(html.contains("Orders: no value."), "{html}");
        assert!(html.contains("aria-label=\"Stages\""), "{html}");
        assert!(!html.contains("NaN"), "{html}");
    }

    /// A first stage of nothing gives no shares to divide by: values only.
    #[test]
    fn an_empty_first_stage_says_values_only() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    FunnelChart {
                        title: "Checkout",
                        stages: vec![Slice::new("Visits", 0.0), Slice::new("Carts", 5.0)],
                    }
                }
            }
        });
        assert!(html.contains("Carts: 5."), "{html}");
        assert!(!html.contains("of the first"), "{html}");
    }

    /// Rising stages are drawn widest first, so development warns of the moved stage.
    #[test]
    fn rising_stages_warn() {
        let warnings = crate::utils::warnings_of(|| {
            rsx! {
                LiberoProvider {
                    FunnelChart {
                        title: "Checkout",
                        stages: vec![Slice::new("Visits", 10.0), Slice::new("Carts", 20.0)],
                    }
                }
            }
        });
        assert_eq!(
            warnings,
            ["FunnelChart: stages rise at Carts - drawn widest first."]
        );
    }

    #[test]
    fn a_funnel_of_zeros_says_so() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    FunnelChart { title: "Nothing", stages: vec![Slice::new("A", 0.0)] }
                }
            }
        });
        assert!(html.contains("No data"), "{html}");
        assert!(!html.contains("<svg"), "{html}");
    }
}
