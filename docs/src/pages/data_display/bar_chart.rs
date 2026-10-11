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
                    prop("tooltip", "Option<bool>").default(on(theme.chart.tooltip)).doc("A tooltip at the pointer's or the keys' bar."),
                    prop("tooltip_content", "Option<Callback<ChartPoint, Element>>").doc("The tooltip's content instead of the category and each series' value there."),
                    prop("onpointclick", "Option<EventHandler<ChartPoint>>").doc("A click on a bar, a second tap on a pinned one, or Enter on the keys' bar."),
                    prop("onactivechange", "Option<EventHandler<Option<ChartPoint>>>").doc("The bar under the pointer or the keys; `None` when it leaves."),
                    prop("hidden", "Option<Vec<String>>").doc("The names of the series the legend hides; set, they are controlled."),
                    prop("default_hidden", "Vec<String>").default("[]").doc("Seeds the hidden series once. Ignored when `hidden` is set."),
                    prop("onhiddenchange", "Option<EventHandler<Vec<String>>>").doc("The hidden series a legend button asks for. The last shown series stays."),
                ]),
            ],
            accessibility: a11y()
                .key(["Tab"], "Into the plot, then on to the legend's buttons.")
                .key(["ArrowRight", "ArrowLeft"], "The next or previous category's bar of the series; `horizontal`, the next or previous series' bar in the category.")
                .key(["ArrowDown", "ArrowUp"], "The next or previous series' bar in the category; `horizontal`, the next or previous category, top to bottom.")
                .key(["Home", "End"], "The series' first or last bar.")
                .key(["Enter", "Space"], "Calls `onpointclick` with the bar.")
                .key(["Escape"], "Clears the bar and its tooltip.")
                .handles([
                    "The chart is a `role=\"figure\"` named by `title`. The drawing is hidden from screen readers.",
                    "A hidden summary gives the category range and each series' lowest and highest value, and where the highest is.",
                    "A hidden table holds every value, up to the theme's `table_cell_cap` cells; a `percent` chart's table keeps the values, not the shares.",
                    "The plot is one tab stop, a `role=\"application\"` described as a chart, so a screen reader hands it the arrows. Each bar the keys reach is announced, as \"2026, Q3: 150\".",
                    "The legend's entries are toggle buttons: pressed while their series shows, struck through when hidden. The last shown series' button is `aria-disabled`.",
                    "On a touch screen a tap pins a bar's tooltip, a second tap on it clicks; a vertical swipe still scrolls the page.",
                    "Each bar's palette colour reaches 3:1 on the page and on a card, in every shipped theme and color scheme; one too close to an earlier one, also for colour-blind eyes, is skipped.",
                    "In forced colours the bars keep their colours on the system background; axes and labels take the system text colour.",
                ])
                .must([
                    "Give a `title`, or name the chart with `aria_label`.",
                    "Say in `description` what the reader should see, as \"Q3 grew the most\".",
                    "Keep the legend, or `value_labels`, when series are told apart by colour alone.",
                    "Name every series differently: the legend hides them by name.",
                ])
                .limits([
                    "A hex colour (`.color(\"#1e88e5\")`) is drawn as given: check its 3:1 on your page.",
                    "The tooltip is for the eye; screen readers hear the announcement instead, so `tooltip_content` reaches only sighted readers.",
                    "The drawing runs left to right in a right-to-left page too, and the arrows follow it unflipped.",
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
                    Control::switch("tooltip").default(on(theme.chart.tooltip)),
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
                        tooltip: values.str("tooltip") == "true",
                        loading: values.str("loading") == "true",
                    }
                },
            }
        }
    }
}
