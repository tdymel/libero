use dioxus::prelude::*;

use super::{
    ChartColor, ChartPart, Series, XValues,
    engine::{self, LineSpec, SeriesSpec},
    props::chart_props,
};
use crate::{
    components::{
        accessibility::visually_hidden_sx,
        common::{
            HtmlTag, Input, Part, States, Variables, names_itself, use_name_warning, variables,
        },
        feedback::Skeleton,
        layout::use_box,
    },
    hooks::{
        use_element, use_formats, use_localization, use_resize_fallback, use_root_id, use_theme,
    },
    localization::{ChartLabels, fill},
    platform::ElementApi,
    sx::{StaticSx, sx},
    theme::{
        CHART_ASPECT_RATIO, CHART_LEGEND_GAP, CHART_SWATCH_SIZE, ChartInk, ChartTable, Curve,
        LegendPosition, Theme, chart_hex,
    },
};

/// Before the first measure, and on the server: the SVG scales to its box from here.
const FIRST_WIDTH: f32 = 640.0;
/// Sizes snap to this, so a drag of the window redraws every few pixels, not every one.
const SIZE_STEP: f32 = 8.0;

static CHART_BASE_SX: StaticSx = StaticSx::new(|| {
    let column = |direction| sx().flex_direction(direction);
    let side = |direction| {
        column(direction).selector(
            "& > [data-slot='legend']",
            sx().flex_direction("column").justify_content("center"),
        )
    };
    sx().display("flex")
        .flex_direction("column")
        .gap("xs")
        // The hidden summary and table sit inside, not at the page's corner.
        .position("relative")
        .min_width("0")
        .selector(ChartPart::Title.selector(), sx().font_weight("600"))
        .selector(ChartPart::Description.selector(), sx().color("text-dimmed"))
        .selector(
            ChartPart::Body.selector(),
            sx().display("flex")
                .flex_direction("column")
                .gap(CHART_LEGEND_GAP.value())
                .min_width("0"),
        )
        .selector(
            ChartPart::Plot.selector(),
            sx().position("relative")
                .flex("1 1 auto")
                .min_width("0")
                .with("aspect-ratio", CHART_ASPECT_RATIO.value())
                .selector("& > [aria-hidden]", sx().width("100%").height("100%"))
                .selector("& svg", sx().display("block")),
        )
        .selector(
            ChartPart::Legend.selector(),
            sx().display("flex")
                .flex_wrap("wrap")
                .justify_content("center")
                .gap("xs")
                .column_gap("md")
                .margin("0")
                .padding("0")
                .with("list-style", "none")
                .font_size("sm"),
        )
        .selector(
            ChartPart::LegendItem.selector(),
            sx().display("inline-flex").align_items("center").gap("xs"),
        )
        .selector(
            "& [data-chart-swatch]",
            sx().flex_shrink("0")
                .width(CHART_SWATCH_SIZE.value())
                .height(CHART_SWATCH_SIZE.value())
                .border_radius("2px")
                // The swatch is the series' only key: forced colours would blank it.
                .forced_color_adjust("none"),
        )
        .selector(
            ChartPart::Empty.selector(),
            sx().position("absolute")
                .with("inset", "0")
                .display("flex")
                .align_items("center")
                .justify_content("center")
                .color("text-dimmed"),
        )
        .selector("& > [data-chart-summary]", visually_hidden_sx())
        .selector(ChartPart::Table.selector(), visually_hidden_sx())
        .when(
            "table-visible",
            sx().selector(
                ChartPart::Table.selector(),
                sx().position("static")
                    .width("auto")
                    .height("auto")
                    .margin("0")
                    .overflow("auto")
                    .clip("auto")
                    .white_space("normal")
                    .opacity("1"),
            ),
        )
        .selector(
            format!("{} table", ChartPart::Table.selector()),
            sx().border_collapse("collapse").font_size("sm"),
        )
        .selector(
            format!("{} :is(th, td)", ChartPart::Table.selector()),
            sx().padding("4px 8px").text_align("end"),
        )
        // The x labels' column reads as text; the values line up on their ends.
        .selector(
            format!("{} :is(th, td):first-child", ChartPart::Table.selector()),
            sx().text_align("start"),
        )
        .selector(
            format!("{} caption", ChartPart::Table.selector()),
            sx().text_align("start").font_weight("600"),
        )
        .when(
            "legend-top",
            sx().selector(ChartPart::Body.selector(), column("column-reverse")),
        )
        // `row-reverse` puts the legend at the start edge in either direction.
        .when(
            "legend-start",
            sx().selector(ChartPart::Body.selector(), side("row-reverse")),
        )
        .when(
            "legend-end",
            sx().selector(ChartPart::Body.selector(), side("row")),
        )
});

chart_props! {
    pub struct LineChartProps {
        /// The x axis, one value per point.
        #[props(into)]
        x: XValues,
        /// The lines, each with one value per x value.
        series: Vec<Series>,
        /// Unset, the theme's.
        #[props(default, into)]
        curve: Input<Curve>,
        /// A dot on every point. Unset, the theme's; never past its `points_limit` points.
        #[props(default)]
        points: Option<bool>,
    }
}

/// One or more series of values over a shared x axis: a trend, a comparison over time.
///
/// Screen readers get the title, a summary (range, lowest and highest of each line) and
/// the values as a table. Web and Android; enable the `charts` feature.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{LineChart, Series};
/// # fn app() -> Element {
/// rsx! {
///     LineChart {
///         title: "Weekly visitors",
///         x: ["Mon", "Tue", "Wed", "Thu", "Fri"],
///         series: vec![
///             Series::new("This week", [120, 132, 101, 134, 90]),
///             Series::new("Last week", [98, 110, 115, 120, 104]).dashed(),
///         ],
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/line-chart>
#[component]
pub fn LineChart(props: LineChartProps) -> Element {
    let theme = use_theme();
    let labels = use_localization().chart;
    let decimal = use_formats().decimal_separator;
    let id = use_root_id(&props.attributes);
    use_name_warning(
        props.title.is_some() || names_itself(&props.attributes),
        "LineChart: no `title` - name the chart: it is how a screen reader finds it.",
    );

    let ratio = props
        .aspect_ratio
        .unwrap_or(theme.chart.aspect_ratio)
        .max(0.1);
    let size = use_plot_size();
    let (width, height) = size
        .get()
        .unwrap_or((FIRST_WIDTH, snap(FIRST_WIDTH / ratio)));

    let x = props.x.labels(&props.value_format, decimal);
    let empty = props
        .series
        .iter()
        .all(|series| series.values.iter().all(Option::is_none));
    let points = props.points.unwrap_or(theme.chart.points) && x.len() <= theme.chart.points_limit;
    let spec = LineSpec {
        width,
        height,
        x: x.clone(),
        series: series_specs(theme, &props.series),
        curve: props.curve.copied_or(theme.chart.curve),
        grid: props.grid.unwrap_or(theme.chart.grid),
        points,
        stroke_width: theme.chart.stroke_width,
        ink: ChartInk::of(theme),
        format: props.value_format,
        unit: props.unit.clone(),
        decimal,
    };
    let drawn = use_memo(use_reactive!(|spec| engine::draw_line(&spec)));

    let legend = match props.legend.as_ref() {
        Some(position) => *position,
        None if props.series.len() > 1 => theme.chart.legend,
        None => LegendPosition::None,
    };
    let table = props.table.copied_or_default();
    let write = |value: f64| {
        props
            .value_format
            .format(value, props.unit.as_deref(), decimal)
    };
    let summary = match (props.loading, empty) {
        (true, _) => vec![labels.loading.to_string()],
        (false, true) => Vec::new(),
        (false, false) => summary(&labels, labels.line, &x, &props.series, write),
    };
    let cells = x.len() * (props.series.len() + 1);
    let rows = (table != ChartTable::Off
        && !props.loading
        && !empty
        && cells <= theme.chart.table_cell_cap)
        .then(|| table_rows(&x, &props.series, write));

    let states: Input<States> = props
        .states
        .clone()
        .unwrap_or_default()
        .with(legend_state(legend), legend != LegendPosition::None)
        .with("table-visible", table == ChartTable::Visible)
        .into();
    let vars: Input<Variables> = variables()
        .with(
            CHART_ASPECT_RATIO,
            props.aspect_ratio.map(|ratio| ratio.to_string()),
        )
        .into();

    let title_id = props.title.as_ref().map(|_| format!("{id}-title"));
    let description_id = props
        .description
        .as_ref()
        .map(|_| format!("{id}-description"));
    let summary_id = (!summary.is_empty()).then(|| format!("{id}-summary"));
    let described_by = [description_id, summary_id]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");
    let caption = props
        .title
        .clone()
        .unwrap_or_else(|| labels.table.to_string());
    let swatches: Vec<(String, String)> = spec
        .series
        .iter()
        .map(|series| (series.name.clone(), series.color.to_string()))
        .collect();
    let plot_element = size.element;
    let svg = drawn.read().as_ref().map(|drawn| drawn.svg.clone());

    let children = rsx! {
        if let Some(title) = props.title.clone() {
            div { "data-slot": ChartPart::Title.slot(), id: "{id}-title", "{title}" }
        }
        if let Some(description) = props.description.clone() {
            div { "data-slot": ChartPart::Description.slot(), id: "{id}-description", "{description}" }
        }
        div { "data-slot": ChartPart::Body.slot(),
            div {
                "data-slot": ChartPart::Plot.slot(),
                onmounted: plot_element.mount(),
                ..plot_element.attributes(),
                if props.loading {
                    Skeleton { height: "100%" }
                } else if empty {
                    div { "data-slot": ChartPart::Empty.slot(),
                        match props.empty.clone() {
                            Some(empty) => empty,
                            None => rsx! { "{labels.no_data}" },
                        }
                    }
                } else if let Some(svg) = svg {
                    // The figure's summary and table speak for the drawing.
                    div { "aria-hidden": "true", dangerous_inner_html: "{svg}" }
                }
            }
            if legend != LegendPosition::None && !props.loading && !empty {
                // The summary names every series: the legend is for the eye.
                ul { "data-slot": ChartPart::Legend.slot(), "aria-hidden": "true",
                    for (name, color) in swatches {
                        li { key: "{name}", "data-slot": ChartPart::LegendItem.slot(),
                            span { "data-chart-swatch": true, style: "background:{color}" }
                            "{name}"
                        }
                    }
                }
            }
        }
        if !summary.is_empty() {
            div { "data-chart-summary": true, id: "{id}-summary",
                for (i, sentence) in summary.into_iter().enumerate() {
                    p { key: "{i}", "{sentence}" }
                }
            }
        }
        if let Some((head, rows)) = rows {
            div {
                "data-slot": ChartPart::Table.slot(),
                // Visible, it scrolls: keyboard users need a stop to scroll it.
                role: (table == ChartTable::Visible).then_some("region"),
                tabindex: (table == ChartTable::Visible).then_some("0"),
                "aria-labelledby": (table == ChartTable::Visible).then(|| format!("{id}-caption")),
                table {
                    caption { id: "{id}-caption", "{caption}" }
                    thead {
                        tr {
                            th { scope: "col", "{labels.category}" }
                            for (i, name) in head.into_iter().enumerate() {
                                th { key: "{i}", scope: "col", "{name}" }
                            }
                        }
                    }
                    tbody {
                        for (i, (x, cells)) in rows.into_iter().enumerate() {
                            tr { key: "{i}",
                                th { scope: "row", "{x}" }
                                for (j, cell) in cells.into_iter().enumerate() {
                                    td { key: "{j}", "{cell}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    use_box()
        .framework_sx(&CHART_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .variables(&vars)
        .prepare()
        // `figure`, not `img`: an image's children are presentational, the table with them.
        .attr("role", "figure")
        .attr("id", id())
        .attr("aria-labelledby", title_id)
        .attr(
            "aria-describedby",
            (!described_by.is_empty()).then_some(described_by),
        )
        .attr("aria-busy", props.loading.then_some("true"))
        .render(HtmlTag::Div, props.attributes, children)
}

fn legend_state(legend: LegendPosition) -> &'static str {
    match legend {
        LegendPosition::Top => "legend-top",
        LegendPosition::Start => "legend-start",
        LegendPosition::End => "legend-end",
        LegendPosition::Bottom | LegendPosition::None => "legend-bottom",
    }
}

fn snap(length: f32) -> f32 {
    ((length / SIZE_STEP).round() * SIZE_STEP).max(SIZE_STEP)
}

/// Each series with its hex: its own colour, else the theme palette's for its place.
fn series_specs(theme: &Theme, series: &[Series]) -> Vec<SeriesSpec> {
    let palette = theme.chart.palette;
    series
        .iter()
        .enumerate()
        .map(|(i, series)| {
            let (color, shade) = palette[i % palette.len()];
            let color = match series.color {
                Some(ChartColor::Hex(hex)) => hex,
                Some(ChartColor::Palette(color, shade)) => chart_hex(theme, color, shade),
                None => chart_hex(theme, color, shade),
            };
            SeriesSpec {
                name: series.name.clone(),
                values: series.values.clone(),
                color,
                dashed: series.dashed,
            }
        })
        .collect()
}

/// The range of x, then each series' lowest and highest value.
fn summary(
    labels: &ChartLabels,
    kind: &str,
    x: &[String],
    series: &[Series],
    write: impl Fn(f64) -> String,
) -> Vec<String> {
    let mut sentences = Vec::new();
    if let (Some(first), Some(last)) = (x.first(), x.last()) {
        sentences.push(fill(
            labels.range,
            &[("kind", &kind), ("first", first), ("last", last)],
        ));
    }
    for series in series {
        let values = series
            .values
            .iter()
            .enumerate()
            .filter_map(|(i, value)| value.map(|value| (i, value)));
        let min = values.clone().map(|(_, value)| value).reduce(f64::min);
        // The first of equal highs.
        let max = values.reduce(|high, next| if next.1 > high.1 { next } else { high });
        sentences.push(match (min, max) {
            (Some(min), Some((at, max))) => fill(
                labels.series,
                &[
                    ("name", &series.name),
                    ("min", &write(min)),
                    ("max", &write(max)),
                    ("x", &x.get(at).cloned().unwrap_or_default()),
                ],
            ),
            _ => fill(labels.series_empty, &[("name", &series.name)]),
        });
    }
    sentences
}

/// The series' names, then per x value its label and each series' value; a gap is empty.
fn table_rows(
    x: &[String],
    series: &[Series],
    write: impl Fn(f64) -> String,
) -> (Vec<String>, Vec<(String, Vec<String>)>) {
    let head = series.iter().map(|series| series.name.clone()).collect();
    let rows = x
        .iter()
        .enumerate()
        .map(|(i, x)| {
            let cells = series
                .iter()
                .map(|series| {
                    series
                        .values
                        .get(i)
                        .copied()
                        .flatten()
                        .map(&write)
                        .unwrap_or_default()
                })
                .collect();
            (x.clone(), cells)
        })
        .collect();
    (head, rows)
}

/// The plot's size, snapped, once it is measured: by its resizes, and at mount in case
/// the first resize comes late.
#[derive(Clone, Copy)]
struct PlotSize {
    element: crate::hooks::ElementHandle,
    size: Signal<Option<(f32, f32)>>,
}

impl PlotSize {
    fn get(&self) -> Option<(f32, f32)> {
        *self.size.read()
    }
}

fn use_plot_size() -> PlotSize {
    let element = use_element();
    let mut size = use_signal(|| None);
    let mut measured = move |width: f64, height: f64| {
        if width <= 0.0 || height <= 0.0 {
            return;
        }
        let next = Some((snap(width as f32), snap(height as f32)));
        if *size.peek() != next {
            size.set(next);
        }
    };
    use_resize_fallback(element, move |event| {
        if let Ok(border) = event.get_border_box_size() {
            measured(border.width, border.height);
        }
    });
    use_effect(move || {
        if !element.is_mounted() {
            return;
        }
        let dimensions = element.dimensions();
        spawn(async move {
            if let Ok(rect) = dimensions.await
                && size.peek().is_none()
            {
                measured(rect.width, rect.height);
            }
        });
    });
    PlotSize { element, size }
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

    /// The server's HTML has the drawing, the name and what a screen reader reads instead.
    #[test]
    fn the_first_render_is_a_named_figure_with_its_summary_and_table() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    LineChart {
                        title: "Visitors",
                        x: ["Mon", "Tue", "Wed"],
                        series: vec![
                            Series::new("This week", [1.0, 3.0, 2.0]),
                            Series::new("Last week", [Some(2.0), None, Some(1.5)]),
                        ],
                    }
                }
            }
        });

        assert!(html.contains("role=\"figure\""), "{html}");
        assert!(html.contains("<svg"), "{html}");
        assert!(html.contains("aria-labelledby="), "{html}");
        assert!(html.contains("Line chart from Mon to Wed."), "{html}");
        assert!(
            html.contains("This week: 1 to 3, highest at Tue."),
            "{html}"
        );
        assert!(html.contains("<caption"), "{html}");
        assert!(
            html.contains("<th scope=\"row\">Tue</th><td>3</td><td></td>"),
            "{html}"
        );
        // Two series: a legend, for the eye only.
        assert!(
            html.contains("aria-hidden=\"true\" data-slot=\"legend\""),
            "{html}"
        );
    }

    #[test]
    fn a_chart_without_values_says_so_and_has_no_table() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    LineChart { title: "Nothing", x: ["a"], series: vec![Series::new("A", [None::<f64>])] }
                }
            }
        });

        assert!(html.contains("No data"), "{html}");
        assert!(!html.contains("<svg") && !html.contains("<table"), "{html}");
    }

    #[test]
    fn a_loading_chart_is_busy() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    LineChart { title: "Later", loading: true, x: ["a"], series: vec![Series::new("A", [1.0])] }
                }
            }
        });

        assert!(html.contains("aria-busy=\"true\""), "{html}");
        assert!(html.contains("Loading chart"), "{html}");
        assert!(!html.contains("<table"), "{html}");
    }

    /// Past the cell cap only the summary speaks: a 10 000-cell table is no help.
    #[test]
    fn a_table_past_the_cap_is_left_out() {
        let html = render(|| {
            let x: Vec<String> = (0..1_500).map(|i| i.to_string()).collect();
            rsx! {
                LiberoProvider {
                    LineChart { title: "Many", x, series: vec![Series::new("A", (0..1_500).map(f64::from))] }
                }
            }
        });

        assert!(!html.contains("<table"), "{html}");
        assert!(html.contains("A: 0 to 1499, highest at 1499."), "{html}");
    }
}
