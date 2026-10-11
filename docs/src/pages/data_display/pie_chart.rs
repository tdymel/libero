// page-snippets: charts
use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::use_theme;
use libero::{
    components::{Code, PieChart, Slice, Text},
    theme::{ChartTable, LegendPosition, PieLabels},
};

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

#[component]
pub fn PieChartPage() -> Element {
    let theme = use_theme();
    rsx! {
        DocPage {
            title: "PieChart",
            source: "libero/src/components/data_display/chart/pie_chart.rs",
            markdown: "/md/pie_chart.md",
            properties: vec![
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
                .key(["ArrowRight", "ArrowDown"], "The next slice, clockwise.")
                .key(["ArrowLeft", "ArrowUp"], "The previous slice.")
                .key(["Home", "End"], "The first or last slice.")
                .key(["Enter", "Space"], "Calls `onpointclick` with the slice.")
                .key(["Escape"], "Clears the slice and its tooltip.")
                .handles([
                    "The chart is a `role=\"figure\"` named by `title`. The drawing is hidden from screen readers.",
                    "A hidden summary gives the total, then each slice's value and share.",
                    "A hidden table holds each slice's value and share.",
                    "The plot is one tab stop, a `role=\"application\"` described as a chart, so a screen reader hands it the arrows. Each slice the keys reach is announced, as \"Chrome: 62, 62%.\"",
                    "The legend's entries are toggle buttons: pressed while their slice shows, struck through when hidden. The last shown slice's button is `aria-disabled`.",
                    "On a touch screen a tap pins a slice's tooltip, a second tap on it clicks; a vertical swipe still scrolls the page.",
                    "Each slice's palette colour reaches 3:1 on the page and on a card, in every shipped theme and color scheme; one too close to an earlier one, also for colour-blind eyes, is skipped.",
                    "In forced colours the slices keep their colours on the system background; axes and labels take the system text colour.",
                ])
                .must([
                    "Give a `title`, or name the chart with `aria_label`.",
                    "Keep `labels` or the legend: colour alone does not tell slices apart.",
                    "Keep to a few slices; fold the small ones into one \"Other\".",
                    "Name every slice differently: the legend hides them by name.",
                ])
                .limits([
                    "A hex colour (`.color(\"#1e88e5\")`) is drawn as given: check its 3:1 on your page.",
                    "A hidden slice leaves the drawing, so the shown ones' shares grow; the summary and the table keep every slice.",
                    "The tooltip is for the eye; screen readers hear the announcement instead, so `tooltip_content` reaches only sighted readers.",
                    "`grid` draws nothing on a pie.",
                ]),
            lead: rsx! {
                Text {
                    "Parts of a whole as slices of a circle, or of a ring. Each "
                    Code { source: "Slice" }
                    " is a name and a value. Needs the "
                    Code { source: "charts" }
                    " feature. Web and Android."
                }
            },
            Demo {
                component: "PieChart",
                children_text: "",
                controls: vec![
                    Control::select("labels", ["percent", "value", "name", "none"])
                        .default(theme.chart.pie_labels.as_str())
                        .code(|control, values| {
                            let mut set = vec![
                                "title: \"Browsers\"".to_string(),
                                "slices: vec![Slice::new(\"Chrome\", 62.0), Slice::new(\"Safari\", 20.0), Slice::new(\"Firefox\", 10.0), Slice::new(\"Other\", 8.0)]".to_string(),
                            ];
                            let value = values.str("labels");
                            if value != control.default {
                                set.push(format!("labels: {value:?}"));
                            }
                            if values.str("donut") == "true" {
                                set.push("center: rsx! { \"1,204 users\" }".to_string());
                            }
                            set
                        }),
                    Control::switch("donut"),
                    Control::select("legend", ["bottom", "top", "start", "end", "none"]),
                    Control::toggle("table", ["hidden", "visible", "off"]),
                    Control::switch("tooltip").default(on(theme.chart.tooltip)),
                    Control::switch("loading"),
                ],
                render: move |values: DemoValues| rsx! {
                    PieChart {
                        title: "Browsers",
                        slices: slices(),
                        donut: values.str("donut") == "true",
                        labels: PieLabels::from(values.str("labels").as_str()),
                        center: rsx! { "1,204 users" },
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
