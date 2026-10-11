// page-snippets: charts
use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::use_theme;
use libero::{
    components::{Code, LineChart, Series, Text},
    theme::{ChartTable, Curve, LegendPosition},
};

const X: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const THIS_WEEK: [f64; 7] = [120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0];
const LAST_WEEK: [f64; 7] = [98.0, 110.0, 115.0, 120.0, 104.0, 170.0, 180.0];
// Uneven samples: the gaps show only on a number axis.
const HOURS: [f64; 7] = [0.0, 1.0, 2.0, 6.0, 12.0, 18.0, 24.0];
const TEMPERATURE: [f64; 7] = [14.0, 13.5, 13.0, 15.5, 22.0, 20.5, 15.0];

fn on(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

#[component]
pub fn LineChartPage() -> Element {
    let theme = use_theme();
    rsx! {
        DocPage {
            title: "LineChart",
            source: "libero/src/components/data_display/chart/line_chart.rs",
            markdown: "/md/line_chart.md",
            properties: vec![
                props("LineChart", vec![
                    prop("x", "XValues").default("required").doc("The x axis, one value per point: labels, one evenly spaced slot each, or numbers, each point at its value."),
                    prop("series", "Vec<Series>").default("required").doc("The lines, each with one value per x value. `None` is a gap."),
                    prop("title", "Option<String>").doc("Shown above the chart, and its accessible name."),
                    prop("description", "Option<String>").doc("A sentence under the title: what to read from the chart."),
                    prop("curve", "Curve").default(theme.chart.curve.as_str()).doc("`linear`, `smooth` or `step`."),
                    prop("points", "Option<bool>").default(on(theme.chart.points)).doc("A dot on every point; never past the theme's `points_limit` points."),
                    prop("legend", "LegendPosition").default("bottom with two series or more").doc("`bottom`, `top`, `start`, `end` or `none`."),
                    prop("grid", "Option<bool>").default(on(theme.chart.grid)).doc("Horizontal guide lines."),
                    prop("value_format", "ValueFormat").default("Auto").doc("How values are written on the axis, in the summary and in the table."),
                    prop("unit", "Option<String>").doc("After every value, as `\"kg\"`."),
                    prop("aspect_ratio", "Option<f32>").default("16 / 9").doc("Width over height of the plot."),
                    prop("loading", "bool").default("false").doc("A placeholder instead of the plot, and `aria-busy` on the chart."),
                    prop("empty", "Option<Element>").doc("Shown in the plot when there is no value at all. Unset, \"No data\"."),
                    prop("table", "ChartTable").default(ChartTable::default().as_str()).doc("The data as a table: `hidden` for screen readers, `visible`, or `off`."),
                    prop("tooltip", "Option<bool>").default(on(theme.chart.tooltip)).doc("A tooltip and a crosshair at the pointer's or the keys' point."),
                    prop("tooltip_content", "Option<Callback<ChartPoint, Element>>").doc("The tooltip's content instead of the x value and each line's value there."),
                    prop("onpointclick", "Option<EventHandler<ChartPoint>>").doc("A click on a point, a second tap on a pinned one, or Enter on the keys' point."),
                    prop("onactivechange", "Option<EventHandler<Option<ChartPoint>>>").doc("The point under the pointer or the keys; `None` when it leaves."),
                    prop("hidden", "Option<Vec<String>>").doc("The names of the lines the legend hides; set, they are controlled."),
                    prop("default_hidden", "Vec<String>").default("[]").doc("Seeds the hidden lines once. Ignored when `hidden` is set."),
                    prop("onhiddenchange", "Option<EventHandler<Vec<String>>>").doc("The hidden lines a legend button asks for. The last shown line stays."),
                ]),
            ],
            accessibility: a11y()
                .key(["Tab"], "Into the plot, then on to the legend's buttons.")
                .key(["ArrowRight", "ArrowLeft"], "The next or previous point of the line, as drawn: left to right in either text direction.")
                .key(["ArrowDown", "ArrowUp"], "The next or previous line at the same x value, or its nearest point.")
                .key(["Home", "End"], "The line's first or last point.")
                .key(["Enter", "Space"], "Calls `onpointclick` with the point.")
                .key(["Escape"], "Clears the point and its tooltip.")
                .handles([
                    "The chart is a `role=\"figure\"` named by `title`. The drawing is hidden from screen readers.",
                    "A hidden summary gives the x range and each line's lowest and highest value, and where the highest is.",
                    "A hidden table holds every value, up to the theme's `table_cell_cap` cells.",
                    "The plot is one tab stop, a `role=\"application\"` described as a chart, so a screen reader hands it the arrows. Each point the keys reach is announced, as \"This week, Tue: 132\".",
                    "The legend's entries are toggle buttons: pressed while their line shows, struck through when hidden. The last shown line's button is `aria-disabled`.",
                    "On a touch screen a tap pins a point's tooltip, a second tap on it clicks; a vertical swipe still scrolls the page.",
                    "Lines take the theme's palette, drawn in the current color scheme.",
                ])
                .must([
                    "Give a `title`, or name the chart with `aria_label`.",
                    "Say in `description` what the reader should see, as \"Visitors peaked on Saturday\".",
                    "Tell lines apart by more than colour: `dashed()` on a series, or their names in the legend.",
                    "Name every series differently: the legend hides them by name.",
                ])
                .limits([
                    "The tooltip is for the eye; screen readers hear the announcement instead, so `tooltip_content` reaches only sighted readers.",
                    "The x axis runs left to right in a right-to-left page too.",
                ]),
            lead: rsx! {
                Text {
                    "One or more "
                    Code { source: "Series" }
                    " over a shared x axis, drawn as an SVG that fills its width. Needs the "
                    Code { source: "charts" }
                    " feature. Web and Android."
                }
                Text {
                    "Text labels sit in evenly spaced slots. Numbers ("
                    Code { source: "x: vec![0.0, 1.0, 6.0]" }
                    ") sit at their value on a number axis, so uneven samples keep their gaps."
                }
            },
            Demo {
                component: "LineChart",
                children_text: "",
                controls: vec![
                    Control::select("curve", ["linear", "smooth", "step"])
                        .default(theme.chart.curve.as_str())
                        .code(|control, values| {
                            let mut set = vec![
                                "title: \"Visitors\"".to_string(),
                                format!("x: {X:?}"),
                                format!(
                                    "series: vec![Series::new(\"This week\", {THIS_WEEK:?}), Series::new(\"Last week\", {LAST_WEEK:?}).dashed()]"
                                ),
                            ];
                            let value = values.str("curve");
                            if value != control.default {
                                set.push(format!("curve: {value:?}"));
                            }
                            set
                        }),
                    Control::select("legend", ["bottom", "top", "start", "end", "none"]),
                    Control::toggle("table", ["hidden", "visible", "off"]),
                    Control::switch("tooltip").default(on(theme.chart.tooltip)),
                    Control::switch("loading"),
                ],
                render: move |values: DemoValues| rsx! {
                    LineChart {
                        title: "Visitors",
                        x: X,
                        series: vec![
                            Series::new("This week", THIS_WEEK),
                            Series::new("Last week", LAST_WEEK).dashed(),
                        ],
                        curve: Curve::from(values.str("curve").as_str()),
                        legend: LegendPosition::from(values.str("legend").as_str()),
                        table: ChartTable::from(values.str("table").as_str()),
                        tooltip: values.str("tooltip") == "true",
                        loading: values.str("loading") == "true",
                    }
                },
            }
            Demo {
                component: "LineChart",
                title: "Numeric x",
                children_text: "",
                fixed: vec![
                    "title: \"Temperature\"".to_string(),
                    format!("x: vec!{HOURS:?}"),
                    format!("series: vec![Series::new(\"Outside\", {TEMPERATURE:?})]"),
                    "unit: \"°C\"".to_string(),
                ],
                controls: vec![Control::select("curve", ["linear", "smooth", "step"])
                    .default(theme.chart.curve.as_str())],
                render: move |values: DemoValues| rsx! {
                    LineChart {
                        title: "Temperature",
                        x: HOURS.to_vec(),
                        series: vec![Series::new("Outside", TEMPERATURE)],
                        unit: "°C",
                        curve: Curve::from(values.str("curve").as_str()),
                    }
                },
            }
        }
    }
}
