// page-snippets: charts
use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::use_theme;
use libero::{
    components::{BarChart, Code, Orientation, Series, Text},
    theme::{BarLayout, ChartTable, LegendPosition},
};

const X: [&str; 4] = ["Q1", "Q2", "Q3", "Q4"];
const LAST_YEAR: [f64; 4] = [120.0, 132.0, 101.0, 134.0];
const THIS_YEAR: [f64; 4] = [140.0, 128.0, 150.0, 162.0];

fn on(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

#[component]
pub fn BarChartPage() -> Element {
    let theme = use_theme();
    rsx! {
        DocPage {
            title: "BarChart",
            source: "libero/src/components/data_display/chart/bar_chart.rs",
            markdown: "/md/bar_chart.md",
            properties: vec![
                props("BarChart", vec![
                    prop("x", "XValues").default("required").doc("The categories, one per bar or group of bars."),
                    prop("series", "Vec<Series>").default("required").doc("The bars' series, each with one value per x value. `None` leaves no bar."),
                    prop("title", "Option<String>").doc("Shown above the chart, and its accessible name."),
                    prop("description", "Option<String>").doc("A sentence under the title: what to read from the chart."),
                    prop("layout", "BarLayout").default(theme.chart.bar_layout.as_str()).doc("`grouped` side by side, `stacked`, or `percent`: stacked to 100%."),
                    prop("orientation", "Orientation").default("vertical").doc("`horizontal` lays the categories down the side, top to bottom."),
                    prop("radius", "Size").default(theme.chart.bar_radius.as_str()).doc("Corner radius of every bar, on the theme's radius scale."),
                    prop("value_labels", "Option<bool>").default(on(theme.chart.value_labels)).doc("Each bar's value written on it."),
                    prop("legend", "LegendPosition").default("bottom with two series or more").doc("`bottom`, `top`, `start`, `end` or `none`."),
                    prop("grid", "Option<bool>").default(on(theme.chart.grid)).doc("Guide lines across the values."),
                    prop("value_format", "ValueFormat").default("Auto").doc("How values are written on the axis, the bars, in the summary and in the table."),
                    prop("unit", "Option<String>").doc("After every value, as `\"kg\"`."),
                    prop("aspect_ratio", "Option<f32>").default("16 / 9").doc("Width over height of the plot."),
                    prop("loading", "bool").default("false").doc("A placeholder instead of the plot, and `aria-busy` on the chart."),
                    prop("empty", "Option<Element>").doc("Shown in the plot when there is no value at all. Unset, \"No data\"."),
                    prop("table", "ChartTable").default(ChartTable::default().as_str()).doc("The data as a table: `hidden` for screen readers, `visible`, or `off`."),
                    prop("tooltip", "Option<bool>").doc("Shared with `LineChart`; no tooltip on bars yet."),
                    prop("tooltip_content", "Option<Callback<ChartPoint, Element>>").doc("Shared with `LineChart`; not used on bars yet."),
                    prop("onpointclick", "Option<EventHandler<ChartPoint>>").doc("Shared with `LineChart`; bars do not call it yet."),
                    prop("onactivechange", "Option<EventHandler<Option<ChartPoint>>>").doc("Shared with `LineChart`; bars do not call it yet."),
                    prop("hidden", "Option<Vec<String>>").doc("Shared with `LineChart`; the legend does not hide bars yet."),
                    prop("default_hidden", "Vec<String>").default("[]").doc("Shared with `LineChart`; not used on bars yet."),
                    prop("onhiddenchange", "Option<EventHandler<Vec<String>>>").doc("Shared with `LineChart`; bars do not call it yet."),
                ]),
            ],
            accessibility: a11y()
                .handles([
                    "The chart is a `role=\"figure\"` named by `title`. The drawing is hidden from screen readers.",
                    "A hidden summary gives the category range and each series' lowest and highest value, and where the highest is.",
                    "A hidden table holds every value, up to the theme's `table_cell_cap` cells; a `percent` chart's table keeps the values, not the shares.",
                    "Bars take the theme's palette, drawn in the current color scheme.",
                ])
                .must([
                    "Give a `title`, or name the chart with `aria_label`.",
                    "Say in `description` what the reader should see, as \"Q3 grew the most\".",
                    "Keep the legend, or `value_labels`, when series are told apart by colour alone.",
                ])
                .limits([
                    "No tooltip and no keyboard path over the bars yet: the summary and the table are the way to the values.",
                    "The legend only names the series: the props `LineChart` uses to hide them and report points are taken but not wired yet.",
                ]),
            lead: rsx! {
                Text {
                    "Values per category as bars, side by side or stacked, drawn as an SVG that fills its width. Takes the same "
                    Code { source: "Series" }
                    " as "
                    Code { source: "LineChart" }
                    ". Needs the "
                    Code { source: "charts" }
                    " feature. Web and Android."
                }
            },
            Demo {
                component: "BarChart",
                children_text: "",
                controls: vec![
                    Control::select("layout", ["grouped", "stacked", "percent"])
                        .default(theme.chart.bar_layout.as_str())
                        .code(|control, values| {
                            let mut set = vec![
                                "title: \"Sales\"".to_string(),
                                format!("x: {X:?}"),
                                format!(
                                    "series: vec![Series::new(\"2025\", {LAST_YEAR:?}), Series::new(\"2026\", {THIS_YEAR:?})]"
                                ),
                            ];
                            let value = values.str("layout");
                            if value != control.default {
                                set.push(format!("layout: {value:?}"));
                            }
                            set
                        }),
                    Control::toggle("orientation", ["vertical", "horizontal"]),
                    Control::select("radius", ["xs", "sm", "md", "lg"]).default(theme.chart.bar_radius.as_str()),
                    Control::switch("value_labels"),
                    Control::select("legend", ["bottom", "top", "start", "end", "none"]),
                    Control::toggle("table", ["hidden", "visible", "off"]),
                    Control::switch("loading"),
                ],
                render: move |values: DemoValues| rsx! {
                    BarChart {
                        title: "Sales",
                        x: X,
                        series: vec![Series::new("2025", LAST_YEAR), Series::new("2026", THIS_YEAR)],
                        layout: BarLayout::from(values.str("layout").as_str()),
                        orientation: Orientation::from(values.str("orientation").as_str()),
                        radius: values.str("radius"),
                        value_labels: values.str("value_labels") == "true",
                        legend: LegendPosition::from(values.str("legend").as_str()),
                        table: ChartTable::from(values.str("table").as_str()),
                        loading: values.str("loading") == "true",
                    }
                },
            }
        }
    }
}
