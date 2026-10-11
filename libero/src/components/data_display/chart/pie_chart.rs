use dioxus::prelude::*;

use super::{
    Slice, ValueFormat,
    engine::{self, PieSpec, SeriesSpec},
    frame::{Frame, Interaction, TableRows, hex, overlay_series, shown_places, use_plot_size},
    legend::use_hidden_series,
    props::chart_props,
};
use crate::{
    components::common::{Input, names_itself, use_name_warning},
    hooks::{use_accessibility, use_formats, use_localization, use_root_id, use_theme},
    localization::{ChartLabels, fill},
    theme::{ChartInk, PieLabels, chart_palette},
};

chart_props! {
    pub struct PieChartProps {
        /// The slices, each a share of their sum.
        slices: Vec<Slice>,
        /// A ring instead of a full pie, `center` in its middle.
        #[props(default)]
        donut: bool,
        /// What each slice has written beside it. Unset, the theme's.
        #[props(default, into)]
        labels: Input<PieLabels>,
        /// In a donut's middle, as a total.
        #[props(default)]
        center: Option<Element>,
        /// Where the first slice starts, in degrees clockwise from 12 o'clock.
        #[props(default)]
        start_angle: f32,
    }
}

/// Parts of a whole as slices of a circle, or of a ring with `donut`.
///
/// Screen readers get the title, a summary (the total, then each slice's value and share)
/// and the values as a table. `grid` draws nothing here. Web and Android; enable the
/// `charts` feature.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{PieChart, Slice};
/// # fn app() -> Element {
/// rsx! {
///     PieChart {
///         title: "Browsers",
///         donut: true,
///         slices: vec![Slice::new("Chrome", 62.0), Slice::new("Safari", 20.0), Slice::new("Other", 18.0)],
///         center: rsx! { "1,204 users" },
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/pie-chart>
#[component]
pub fn PieChart(props: PieChartProps) -> Element {
    let theme = use_theme();
    let labels = use_localization().chart;
    let formats = use_formats();
    let id = use_root_id(&props.attributes);
    use_name_warning(
        props.title.is_some() || names_itself(&props.attributes),
        "PieChart: no `title` - name the chart: it is how a screen reader finds it.",
    );

    let ratio = props
        .aspect_ratio
        .unwrap_or(theme.chart.aspect_ratio)
        .max(0.1);
    let size = use_plot_size();
    let (width, height) = size.get(ratio);

    let total: f64 = props.slices.iter().map(|slice| share_of(slice.value)).sum();
    let empty = total <= 0.0;
    let forced = use_accessibility().forced_colors();
    let palette = chart_palette(theme, forced);
    let all_slices: Vec<SeriesSpec> = props
        .slices
        .iter()
        .enumerate()
        .map(|(i, slice)| SeriesSpec {
            name: slice.name.clone(),
            values: vec![Some(share_of(slice.value))],
            color: hex(theme, forced, &palette, i, slice.color),
            dashed: false,
        })
        .collect();
    let names: Vec<String> = all_slices.iter().map(|s| s.name.clone()).collect();
    let hidden = use_hidden_series(
        &props.hidden,
        &props.default_hidden,
        props.onhiddenchange,
        &names,
    );
    let shown = shown_places(&all_slices, &hidden.get());
    let spec = PieSpec {
        width,
        height,
        slices: shown.iter().map(|i| all_slices[*i].clone()).collect(),
        donut: props.donut.then_some(theme.chart.donut_thickness),
        labels: props.labels.copied_or(theme.chart.pie_labels),
        start_angle: props.start_angle,
        ink: ChartInk::of(theme, forced),
        format: props.value_format,
        unit: props.unit.clone(),
        formats,
    };
    let drawn = use_memo(use_reactive!(|spec, shown| {
        // The shown slices may sum to nothing while hidden ones do not: a blank plot.
        let drawable = spec
            .slices
            .iter()
            .any(|s| s.values[0].is_some_and(|v| v > 0.0));
        drawable
            .then(|| engine::draw_pie(&spec))
            .flatten()
            .map(|drawn| drawn.with_indices(&shown))
    }));

    let write = |value: f64| {
        props
            .value_format
            .format(value, props.unit.as_deref(), formats)
    };
    let share = |value: f64| ValueFormat::Percent.format(share_of(value) / total, None, formats);
    let summary = match (props.loading, empty) {
        (true, _) => vec![labels.loading.to_string()],
        (false, true) => Vec::new(),
        (false, false) => summary(&labels, &props.slices, write(total), write, share),
    };
    let table = props.table.copied_or_default();
    let rows = Frame::rows(
        table,
        !props.loading && !empty,
        props.slices.len() * 3,
        theme,
        || table_rows(&labels, &props.slices, write, share),
    );

    Frame {
        id,
        labels,
        title: props.title.clone(),
        description: props.description.clone(),
        loading: props.loading,
        empty,
        empty_slot: props.empty.clone(),
        drawn,
        center: props.center.clone().filter(|_| props.donut),
        legend: Frame::legend(&props.legend, theme, props.slices.len()),
        legend_label: labels.slices,
        series: overlay_series(&all_slices, &[]),
        hidden,
        x: names,
        summary,
        table,
        rows,
        plot: size,
        aspect_ratio: props.aspect_ratio,
        interaction: Interaction {
            format: props.value_format,
            unit: props.unit.clone(),
            formats,
            tooltip: props.tooltip.unwrap_or(theme.chart.tooltip),
            tooltip_content: props.tooltip_content,
            onpointclick: props.onpointclick,
            onactivechange: props.onactivechange,
            swap_axes: false,
        },
    }
    .render(
        &props.class,
        &props.sx,
        &props.parts,
        &props.states,
        props.attributes,
    )
}

/// A slice's part of the pie: a negative or missing value takes none.
fn share_of(value: f64) -> f64 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

/// The total, then every slice's value and share.
fn summary(
    labels: &ChartLabels,
    slices: &[Slice],
    total: String,
    write: impl Fn(f64) -> String,
    share: impl Fn(f64) -> String,
) -> Vec<String> {
    let mut sentences = vec![fill(
        labels.total,
        &[("kind", &labels.pie), ("total", &total)],
    )];
    sentences.extend(slices.iter().map(|slice| {
        fill(
            labels.slice,
            &[
                ("name", &slice.name),
                ("value", &write(slice.value)),
                ("share", &share(slice.value)),
            ],
        )
    }));
    sentences
}

/// Per slice its name, value and share.
fn table_rows(
    labels: &ChartLabels,
    slices: &[Slice],
    write: impl Fn(f64) -> String,
    share: impl Fn(f64) -> String,
) -> TableRows {
    let head = vec![labels.value.to_string(), labels.share.to_string()];
    let rows = slices
        .iter()
        .map(|slice| {
            (
                slice.name.clone(),
                vec![write(slice.value), share(slice.value)],
            )
        })
        .collect();
    (head, rows)
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

    /// The server's HTML has a slice per value, the name and what a screen reader reads.
    #[test]
    fn the_first_render_is_a_named_figure_with_slices_a_summary_and_a_table() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    PieChart {
                        title: "Browsers",
                        slices: vec![Slice::new("Chrome", 3.0), Slice::new("Safari", 1.0)],
                    }
                }
            }
        });

        assert!(html.contains("role=\"figure\""), "{html}");
        assert!(html.contains("<svg"), "{html}");
        assert!(html.contains("Pie chart, total 4."), "{html}");
        assert!(html.contains("Chrome: 3, 75%."), "{html}");
        assert!(
            html.contains("<th scope=\"row\">Safari</th><td>1</td><td>25%</td>"),
            "{html}"
        );
        // Labels in percent by default.
        assert!(html.contains(">75%<") || html.contains("75%\n"), "{html}");
    }

    /// A hidden slice leaves the pie, so the others' shares grow; the summary keeps all.
    #[test]
    fn a_default_hidden_slice_is_not_drawn_and_the_plot_is_walkable() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    PieChart {
                        title: "Browsers",
                        slices: vec![Slice::new("Chrome", 1.0), Slice::new("Safari", 1.0), Slice::new("Other", 2.0)],
                        default_hidden: vec!["Other".to_string()],
                    }
                }
            }
        });

        let drawing = html
            .split("data-chart-drawing")
            .nth(1)
            .and_then(|rest| rest.split("</svg>").next())
            .unwrap_or_default();
        assert!(
            drawing.contains("50%") && !drawing.contains("25%"),
            "{html}"
        );
        assert!(html.contains("Other: 2, 50%."), "{html}");
        assert!(html.contains("aria-label=\"Slices\""), "{html}");
        assert!(html.contains("aria-pressed=\"false\""), "{html}");
        assert!(
            html.contains("role=\"application\" tabindex=\"0\""),
            "{html}"
        );
    }

    #[test]
    fn a_donut_shows_its_center() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    PieChart {
                        title: "Users",
                        donut: true,
                        labels: "none",
                        slices: vec![Slice::new("A", 1.0), Slice::new("B", 1.0)],
                        center: rsx! { "2 users" },
                    }
                }
            }
        });

        assert!(html.contains("data-slot=\"center\""), "{html}");
        assert!(html.contains("2 users"), "{html}");
    }

    #[test]
    fn a_pie_of_zeros_says_so() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    PieChart { title: "Nothing", slices: vec![Slice::new("A", 0.0)] }
                }
            }
        });

        assert!(html.contains("No data"), "{html}");
        assert!(!html.contains("<svg") && !html.contains("<table"), "{html}");
    }
}
