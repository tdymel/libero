// page-snippets: charts
use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, prop, props};
use dioxus::prelude::*;
use libero::use_theme;
use libero::{
    components::{BarChart, Code, LineChart, Orientation, PieChart, Series, Slice, Text},
    theme::{BarLayout, ChartTable, Curve, LegendPosition, PieLabels},
};

const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const THIS_WEEK: [f64; 7] = [120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0];
const LAST_WEEK: [f64; 7] = [98.0, 110.0, 115.0, 120.0, 104.0, 170.0, 180.0];
const QUARTERS: [&str; 4] = ["Q1", "Q2", "Q3", "Q4"];
const SALES_2025: [f64; 4] = [120.0, 132.0, 101.0, 134.0];
const SALES_2026: [f64; 4] = [140.0, 128.0, 150.0, 162.0];

fn on(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn slices() -> Vec<Slice> {
    vec![
        Slice::new("Chrome", 62.0),
        Slice::new("Safari", 20.0),
        Slice::new("Firefox", 10.0),
        Slice::new("Other", 8.0),
    ]
}

fn is(kind: &'static str) -> fn(&DemoValues) -> bool {
    match kind {
        "bar" => |values| values.str("type") != "bar",
        "pie" => |values| values.str("type") != "pie",
        _ => |values| values.str("type") != "line",
    }
}

/// The type's component in place of the generated `Chart`.
fn component(values: &DemoValues, source: &str) -> String {
    let name = match values.str("type").as_str() {
        "bar" => "BarChart",
        "pie" => "PieChart",
        _ => "LineChart",
    };
    source.replacen("Chart", name, 1)
}

#[component]
pub fn ChartPage() -> Element {
    let theme = use_theme();
    let legend = ["bottom", "top", "start", "end", "none"];
    rsx! {
        DocPage {
            title: "Chart",
            source: "libero/src/components/data_display/chart",
            markdown: "/md/chart.md",
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
                props("PieChart", vec![
                    prop("slices", "Vec<Slice>").default("required").doc("The slices, each a share of their sum. A negative value takes no share."),
                    prop("title", "Option<String>").doc("Shown above the chart, and its accessible name."),
                    prop("description", "Option<String>").doc("A sentence under the title: what to read from the chart."),
                    prop("donut", "bool").default("false").doc("A ring instead of a full pie, `center` in its middle."),
                    prop("labels", "PieLabels").default(theme.chart.pie_labels.as_str()).doc("Beside each slice: `percent`, `value`, `name` or `none`."),
                    prop("center", "Option<Element>").doc("In a donut's middle, as a total."),
                    prop("start_angle", "f32").default("0").doc("Where the first slice starts, in degrees clockwise from 12 o'clock."),
                    prop("legend", "LegendPosition").default("bottom with two slices or more").doc("`bottom`, `top`, `start`, `end` or `none`."),
                    prop("value_format", "ValueFormat").default("Auto").doc("How values are written beside the slices, in the summary and in the table."),
                    prop("unit", "Option<String>").doc("After every value, as `\"kg\"`."),
                    prop("grid", "Option<bool>").doc("Shared with the other charts; a pie has no grid to draw."),
                    prop("aspect_ratio", "Option<f32>").default("16 / 9").doc("Width over height of the plot."),
                    prop("loading", "bool").default("false").doc("A placeholder instead of the plot, and `aria-busy` on the chart."),
                    prop("empty", "Option<Element>").doc("Shown in the plot when the slices sum to nothing. Unset, \"No data\"."),
                    prop("table", "ChartTable").default(ChartTable::default().as_str()).doc("The data as a table: `hidden` for screen readers, `visible`, or `off`."),
                    prop("tooltip", "Option<bool>").default(on(theme.chart.tooltip)).doc("A tooltip with the slice's value and share at the pointer's or the keys' slice."),
                    prop("tooltip_content", "Option<Callback<ChartPoint, Element>>").doc("The tooltip's content instead of the slice's name, value and share."),
                    prop("onpointclick", "Option<EventHandler<ChartPoint>>").doc("A click on a slice, a second tap on a pinned one, or Enter on the keys' slice. `index` is the slice's place in `slices`, `x` and `series_name` its name."),
                    prop("onactivechange", "Option<EventHandler<Option<ChartPoint>>>").doc("The slice under the pointer or the keys; `None` when it leaves."),
                    prop("hidden", "Option<Vec<String>>").doc("The names of the slices the legend hides; set, they are controlled."),
                    prop("default_hidden", "Vec<String>").default("[]").doc("Seeds the hidden slices once. Ignored when `hidden` is set."),
                    prop("onhiddenchange", "Option<EventHandler<Vec<String>>>").doc("The hidden slices a legend button asks for. The last shown slice stays."),
                ]),
            ],
            accessibility: a11y()
                .key(["Tab"], "Into the plot, then on to the legend's buttons.")
                .key(["ArrowRight", "ArrowLeft"], "Line: the next or previous point of the line, as drawn, left to right in either text direction. Bar: the next or previous category's bar of the series; `horizontal`, the next or previous series' bar in the category. Pie: the next or previous slice, clockwise.")
                .key(["ArrowDown", "ArrowUp"], "Line: the next or previous line at the same x value, or its nearest point. Bar: the next or previous series' bar in the category; `horizontal`, the next or previous category, top to bottom. Pie: the next or previous slice.")
                .key(["Home", "End"], "The line's or the series' first or last point or bar; a pie's first or last slice.")
                .key(["Enter", "Space"], "Calls `onpointclick` with the point, bar or slice.")
                .key(["Escape"], "Clears the point and its tooltip.")
                .handles([
                    "The chart is a `role=\"figure\"` named by `title`. The drawing is hidden from screen readers.",
                    "A hidden summary gives the x range and each series' lowest and highest value, and where the highest is; a pie's, the total, then each slice's value and share.",
                    "A hidden table holds every value, up to the theme's `table_cell_cap` cells; a `percent` bar chart's keeps the values, not the shares; a pie's has each slice's share too.",
                    "The plot is one tab stop, a `role=\"application\"` described as a chart, so a screen reader hands it the arrows. Each point the keys reach is announced, as \"This week, Tue: 132\" or \"Chrome: 62, 62%.\"",
                    "The legend's entries are toggle buttons: pressed while their series or slice shows, struck through when hidden. The last shown one's button is `aria-disabled`. A dashed line's key is a dash, as drawn.",
                    "On a touch screen a tap pins a point's tooltip, a second tap on it clicks; a vertical swipe still scrolls the page.",
                    "Each series' or slice's palette colour reaches 3:1 on the page and on a card, in every shipped theme and color scheme; one too close to an earlier one, also for colour-blind eyes, is skipped.",
                    "In forced colours the lines, bars and slices keep their colours on the system background; axes and labels take the system text colour.",
                ])
                .must([
                    "Give a `title`, or name the chart with `aria_label`.",
                    "Say in `description` what the reader should see, as \"Visitors peaked on Saturday\".",
                    "Tell series apart by more than colour: `dashed()` on a line, the legend, `value_labels` on bars, `labels` on a pie.",
                    "Name every series and slice differently: the legend hides them by name.",
                    "Keep a pie to a few slices; fold the small ones into one \"Other\".",
                ])
                .limits([
                    "A hex colour (`.color(\"#1e88e5\")`) is drawn as given: check its 3:1 on your page.",
                    "The tooltip is for the eye; screen readers hear the announcement instead, so `tooltip_content` reaches only sighted readers.",
                    "The drawing runs left to right in a right-to-left page too, and the arrows follow it unflipped.",
                    "A hidden slice leaves the drawing, so the shown ones' shares grow; the summary and the table keep every slice.",
                    "`grid` draws nothing on a pie.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "LineChart" }
                    ", "
                    Code { source: "BarChart" }
                    " and "
                    Code { source: "PieChart" }
                    ", each drawn as an SVG that fills its width, with a tooltip, the arrow keys, a legend that hides and shows, and a summary and table for screen readers. Needs the "
                    Code { source: "charts" }
                    " feature. Web and Android."
                }
                Text {
                    "Lines and bars take one or more "
                    Code { source: "Series" }
                    " over a shared x axis; a pie takes "
                    Code { source: "Slice" }
                    "s, each a name and a value."
                }
            },
            Demo {
                component: "Chart",
                children_text: "",
                wrap: Wrap(component),
                controls: vec![
                    Control::toggle("type", ["line", "bar", "pie"])
                        .default("line")
                        .labels(["LineChart", "BarChart", "PieChart"])
                        .code(|_, values| match values.str("type").as_str() {
                            "bar" => vec![
                                "title: \"Sales\"".to_string(),
                                format!("x: {QUARTERS:?}"),
                                format!(
                                    "series: vec![Series::new(\"2025\", {SALES_2025:?}), Series::new(\"2026\", {SALES_2026:?})]"
                                ),
                            ],
                            "pie" => vec![
                                "title: \"Browsers\"".to_string(),
                                "slices: vec![Slice::new(\"Chrome\", 62.0), Slice::new(\"Safari\", 20.0), Slice::new(\"Firefox\", 10.0), Slice::new(\"Other\", 8.0)]".to_string(),
                            ],
                            _ => vec![
                                "title: \"Visitors\"".to_string(),
                                format!("x: {DAYS:?}"),
                                format!(
                                    "series: vec![Series::new(\"This week\", {THIS_WEEK:?}), Series::new(\"Last week\", {LAST_WEEK:?}).dashed()]"
                                ),
                            ],
                        }),
                    Control::select("curve", ["linear", "smooth", "step"])
                        .default(theme.chart.curve.as_str())
                        .hidden_when(is("line")),
                    Control::select("layout", ["grouped", "stacked", "percent"])
                        .default(theme.chart.bar_layout.as_str())
                        .hidden_when(is("bar")),
                    Control::toggle("orientation", ["vertical", "horizontal"]).hidden_when(is("bar")),
                    Control::select("radius", ["xs", "sm", "md", "lg"])
                        .default(theme.chart.bar_radius.as_str())
                        .hidden_when(is("bar")),
                    Control::switch("value_labels")
                        .default(on(theme.chart.value_labels))
                        .hidden_when(is("bar")),
                    Control::select("labels", ["percent", "value", "name", "none"])
                        .default(theme.chart.pie_labels.as_str())
                        .hidden_when(is("pie")),
                    Control::switch("donut").hidden_when(is("pie")).code(|_, values| {
                        match values.str("donut").as_str() {
                            "true" => vec![
                                "donut: true".to_string(),
                                "center: rsx! { \"1,204 users\" }".to_string(),
                            ],
                            _ => vec![],
                        }
                    }),
                    Control::select("legend", legend),
                    Control::toggle("table", ["hidden", "visible", "off"]),
                    Control::switch("tooltip").default(on(theme.chart.tooltip)),
                    Control::switch("loading"),
                ],
                render: move |values: DemoValues| {
                    let legend = LegendPosition::from(values.str("legend").as_str());
                    let table = ChartTable::from(values.str("table").as_str());
                    let tooltip = values.str("tooltip") == "true";
                    let loading = values.str("loading") == "true";
                    match values.str("type").as_str() {
                        "bar" => rsx! {
                            BarChart {
                                title: "Sales",
                                x: QUARTERS,
                                series: vec![Series::new("2025", SALES_2025), Series::new("2026", SALES_2026)],
                                layout: BarLayout::from(values.str("layout").as_str()),
                                orientation: Orientation::from(values.str("orientation").as_str()),
                                radius: values.str("radius"),
                                value_labels: values.str("value_labels") == "true",
                                legend, table, tooltip, loading,
                            }
                        },
                        "pie" => rsx! {
                            PieChart {
                                title: "Browsers",
                                slices: slices(),
                                labels: PieLabels::from(values.str("labels").as_str()),
                                donut: values.str("donut") == "true",
                                center: rsx! { "1,204 users" },
                                legend, table, tooltip, loading,
                            }
                        },
                        _ => rsx! {
                            LineChart {
                                title: "Visitors",
                                x: DAYS,
                                series: vec![
                                    Series::new("This week", THIS_WEEK),
                                    Series::new("Last week", LAST_WEEK).dashed(),
                                ],
                                curve: Curve::from(values.str("curve").as_str()),
                                legend, table, tooltip, loading,
                            }
                        },
                    }
                },
            }
        }
    }
}
