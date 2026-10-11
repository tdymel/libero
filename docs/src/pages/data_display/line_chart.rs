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
                    prop("x", "XValues").default("required").doc("The x axis, one value per point: labels or numbers."),
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
                ]),
            ],
            accessibility: a11y()
                .handles([
                    "The chart is a `role=\"figure\"` named by `title`. The drawing is hidden from screen readers.",
                    "A hidden summary gives the x range and each line's lowest and highest value, and where the highest is.",
                    "A hidden table holds every value, up to the theme's `table_cell_cap` cells.",
                    "Lines take the theme's palette, drawn in the current color scheme.",
                ])
                .must([
                    "Give a `title`, or name the chart with `aria_label`.",
                    "Say in `description` what the reader should see, as \"Visitors peaked on Saturday\".",
                    "Tell lines apart by more than colour: `dashed()` on a series, or their names in the legend.",
                ])
                .limits([
                    "No tooltip and no keyboard path over the points yet: the summary and the table are the way to the values.",
                ]),
            lead: rsx! {
                Text {
                    "One or more "
                    Code { source: "Series" }
                    " over a shared x axis, drawn as an SVG that fills its width. Needs the "
                    Code { source: "charts" }
                    " feature. Web and Android."
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
                        loading: values.str("loading") == "true",
                    }
                },
            }
        }
    }
}
