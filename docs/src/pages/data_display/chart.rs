// page-snippets: charts
use crate::{
    Route,
    components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, a11y, prop, props},
};
use dioxus::prelude::*;
use libero::use_theme;
use libero::{
    components::{
        Anchor, BarChart, ChartRange, Code, LineChart, MarkLine, MarkPoint, Orientation, PieChart,
        Series, Slice, Table, Text, XValues, column, use_chart,
    },
    theme::{
        BarLabelPosition, BarLayout, Curve, LegendPosition, PieLabelPosition, PieLabels,
        PointSymbol,
    },
};

const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const THIS_WEEK: [f64; 7] = [120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0];
const LAST_WEEK: [f64; 7] = [98.0, 110.0, 115.0, 120.0, 104.0, 170.0, 180.0];
const LAST_WEEK_RANGE: [(f64, f64); 7] = [
    (88.0, 108.0),
    (99.0, 121.0),
    (104.0, 126.0),
    (108.0, 132.0),
    (94.0, 114.0),
    (153.0, 187.0),
    (162.0, 198.0),
];
const CONVERSION: [f64; 7] = [2.1, 2.4, 1.8, 2.6, 1.5, 3.4, 3.1];
const QUARTERS: [&str; 4] = ["Q1", "Q2", "Q3", "Q4"];
const SALES_2025: [f64; 4] = [120.0, 132.0, 101.0, 134.0];
const SALES_2026: [f64; 4] = [140.0, 128.0, 150.0, 162.0];
const PROFIT_2025: [f64; 4] = [40.0, 55.0, -25.0, 62.0];
const PROFIT_2026: [f64; 4] = [30.0, -15.0, 20.0, 48.0];

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
        "axes" => |values| values.str("type") == "pie",
        _ => |values| values.str("type") != "line",
    }
}

/// The value axis' fixed ends, `y_min` and `y_max`, for the chart type.
fn y_range(values: &DemoValues) -> Option<(f64, f64)> {
    match (values.str("y_range").as_str(), values.str("type").as_str()) {
        ("auto", _) => None,
        (_, "bar") => Some((-50.0, 200.0)),
        _ if values.str("area") == "stacked" => Some((0.0, 450.0)),
        _ => Some((50.0, 250.0)),
    }
}

/// `x_title` and `y_title` for the chart type.
fn axis_titles(values: &DemoValues) -> Option<(&'static str, &'static str)> {
    match (
        values.str("axis_titles") == "true",
        values.str("type").as_str(),
    ) {
        (false, _) => None,
        (true, "bar") => Some(("Quarter", "Profit")),
        (true, _) => Some(("Day", "Visitors")),
    }
}

/// The line chart's second series, with its error band as the switch says.
fn last_week(values: &DemoValues) -> (String, Series) {
    let series = Series::new("Last week", LAST_WEEK)
        .dashed()
        .symbol(PointSymbol::Square);
    match values.str("error_band") == "true" {
        true => (
            format!(".errors({LAST_WEEK_RANGE:?})"),
            series.errors(LAST_WEEK_RANGE),
        ),
        false => (String::new(), series),
    }
}

/// The line chart's series on the right axis, in code and as built.
fn conversion(values: &DemoValues) -> Option<(String, Series)> {
    (values.str("type") == "line" && values.str("secondary_axis") == "true").then(|| {
        (
            format!(", Series::new(\"Conversion\", {CONVERSION:?}).unit(\"%\").symbol(PointSymbol::Triangle).secondary_axis()"),
            Series::new("Conversion", CONVERSION)
                .unit("%")
                .symbol(PointSymbol::Triangle)
                .secondary_axis(),
        )
    })
}

/// The first series' mark line, in code and as built, for the chart type.
fn mark_line(values: &DemoValues) -> Option<(String, MarkLine)> {
    let target = match values.str("type").as_str() {
        "bar" => 50.0,
        _ => 200.0,
    };
    match values.str("mark_line").as_str() {
        "average" => Some((
            ".mark_line(MarkLine::average())".to_string(),
            MarkLine::average(),
        )),
        "target" => Some((
            format!(".mark_line(MarkLine::value({target:?}).label(\"Target\"))"),
            MarkLine::value(target).label("Target"),
        )),
        _ => None,
    }
}

/// A line chart's marked points, as the control says.
fn mark_points(values: &DemoValues) -> Vec<MarkPoint> {
    match (
        values.str("type").as_str(),
        values.str("mark_point").as_str(),
    ) {
        ("line", "max") => vec![MarkPoint::Max],
        ("line", "min") => vec![MarkPoint::Min],
        ("line", "both") => vec![MarkPoint::Max, MarkPoint::Min],
        _ => vec![],
    }
}

/// The first series' marks, as code.
fn marks_code(values: &DemoValues) -> String {
    let line = mark_line(values).map(|m| m.0).unwrap_or_default();
    let points: String = mark_points(values)
        .iter()
        .map(|point| format!(".mark_point(MarkPoint::{point:?})"))
        .collect();
    line + &points
}

/// The first series, marked as the controls say.
fn marked(values: &DemoValues, series: Series) -> Series {
    let series = match mark_line(values) {
        Some((_, line)) => series.mark_line(line),
        None => series,
    };
    mark_points(values)
        .into_iter()
        .fold(series, Series::mark_point)
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
    let chart = use_chart();
    let mut selected = use_signal(|| None::<ChartRange>);
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
                    prop("area", "Option<bool>").default(on(theme.chart.area)).doc("The area under each line filled in its colour, at the theme's `area_opacity`."),
                    prop("stacked", "bool").default("false").doc("Each line drawn on top of the ones before it, so the top one is the total; with `area`, each fills down to the line below. The tooltip and the keys keep each line's own value. A line on the right y axis is not stacked."),
                    prop("y_min", "Option<f64>").doc("The y axis' lowest value. Unset, 0 or the lowest value below it."),
                    prop("y_max", "Option<f64>").doc("The y axis' highest value. Unset, a round number at or over the highest value."),
                    prop("y_ticks", "Option<usize>").default(theme.chart.y_ticks.to_string()).doc("The steps the y axis is cut into: 4 writes 5 labels."),
                    prop("x_title", "Option<String>").doc("Under the x axis: what it counts."),
                    prop("y_title", "Option<String>").doc("Beside the y axis, read upward: what it measures."),
                    prop("y2_title", "Option<String>").doc("Beside the right y axis, read downward: what its `Series::secondary_axis()` series measure."),
                    prop("legend", "LegendPosition").default("bottom with two series or more").doc("`bottom`, `top`, `start`, `end` or `none`."),
                    prop("grid", "Option<bool>").default(on(theme.chart.grid)).doc("Horizontal guide lines."),
                    prop("value_format", "ValueFormat").default("Auto").doc("How values are written on the axis, in the tooltip and in the summary."),
                    prop("unit", "Option<String>").doc("After every value, as `\"kg\"`."),
                    prop("aspect_ratio", "Option<f32>").default("16 / 9").doc("Width over height of the plot."),
                    prop("loading", "bool").default("false").doc("A placeholder instead of the plot, and `aria-busy` on the chart."),
                    prop("empty", "Option<Element>").doc("Shown in the plot when there is no value at all. Unset, \"No data\"."),
                    prop("tooltip", "Option<bool>").default(on(theme.chart.tooltip)).doc("A tooltip and a crosshair at the pointer's or the keys' point."),
                    prop("tooltip_content", "Option<Callback<ChartPoint, Element>>").doc("The tooltip's content instead of the x value and each line's value there."),
                    prop("onpointclick", "Option<EventHandler<ChartPoint>>").doc("A click on a point, a second tap on a pinned one, or Enter on the keys' point."),
                    prop("onactivechange", "Option<EventHandler<Option<ChartPoint>>>").doc("The point under the pointer or the keys; `None` when it leaves."),
                    prop("hidden", "Option<Vec<String>>").doc("The names of the lines the legend hides; set, they are controlled."),
                    prop("default_hidden", "Vec<String>").default("[]").doc("Seeds the hidden lines once. Ignored when `hidden` is set."),
                    prop("onhiddenchange", "Option<EventHandler<Vec<String>>>").doc("The hidden lines a legend button asks for. The last shown line stays."),
                    prop("zoom", "Option<bool>").default(on(theme.chart.zoom)).doc("Zoom and pan the x axis: Ctrl or Cmd with the wheel, a pinch, a drag (Shift or Alt with it once `onrange` is set), the keys, and a Reset zoom button."),
                    prop("onrange", "Option<EventHandler<Option<ChartRange>>>").doc("Set, a drag or Shift with an arrow brushes a band of x values; Shift or Alt with a drag pans instead. `None` when Escape clears it."),
                    prop("handle", "Option<ChartHandle>").doc("From `use_chart()`: reads the drawing as SVG or PNG, saves it, resets the zoom."),
                    prop("export_button", "Option<bool>").default(on(theme.chart.export_button)).doc("Save as SVG and Save as PNG buttons above the plot."),
                ]),
                props("BarChart", vec![
                    prop("x", "XValues").default("required").doc("The categories, one per bar or group of bars."),
                    prop("series", "Vec<Series>").default("required").doc("The bars' series, each with one value per x value. `None` leaves no bar."),
                    prop("title", "Option<String>").doc("Shown above the chart, and its accessible name."),
                    prop("description", "Option<String>").doc("A sentence under the title: what to read from the chart."),
                    prop("layout", "BarLayout").default(theme.chart.bar_layout.as_str()).doc("`grouped` side by side, `stacked`, or `percent`: stacked to 100%."),
                    prop("orientation", "Orientation").default("vertical").doc("`horizontal` lays the categories down the side, top to bottom."),
                    prop("radius", "Size").default(theme.chart.bar_radius.as_str()).doc("Corner radius of a bar's free end, the one away from its axis, on the theme's radius scale."),
                    prop("value_labels", "Option<bool>").default(on(theme.chart.value_labels)).doc("Each bar's value written on it."),
                    prop("y_min", "Option<f64>").doc("The value axis' lowest value. Unset, 0 or the lowest value below it. Ignored by `percent`, which runs 0 to 100%."),
                    prop("y_max", "Option<f64>").doc("The value axis' highest value. Unset, a round number at or over the highest value. Ignored by `percent`."),
                    prop("y_ticks", "Option<usize>").default(theme.chart.y_ticks.to_string()).doc("The steps the value axis is cut into: 4 writes 5 labels."),
                    prop("x_title", "Option<String>").doc("Along the categories: what they are. Under them, or beside them when `horizontal`."),
                    prop("y_title", "Option<String>").doc("Along the values: what they measure. Beside them, or under them when `horizontal`."),
                    prop("value_label_position", "BarLabelPosition").default("chart-wide: inside on a `horizontal` chart or once any shown series has error ranges, past the end otherwise").doc("`inside` the bar, in the ink that reads on it, or `outside` past its end. A stack always writes inside."),
                    prop("bar_width", "Option<f32>").default("5 px each side of a band").doc("The share of a category's band its bars fill, clamped to `0.1..=1`."),
                    prop("bar_gap", "Option<f32>").default(theme.chart.bar_gap.to_string()).doc("Between two bars of a group, in pixels."),
                    prop("legend", "LegendPosition").default("bottom with two series or more").doc("`bottom`, `top`, `start`, `end` or `none`."),
                    prop("grid", "Option<bool>").default(on(theme.chart.grid)).doc("Guide lines across the values."),
                    prop("value_format", "ValueFormat").default("Auto").doc("How values are written on the axis, the bars, in the tooltip and in the summary. On `percent` the axis and the bars write shares in percent."),
                    prop("unit", "Option<String>").doc("After every value, as `\"kg\"`."),
                    prop("aspect_ratio", "Option<f32>").default("16 / 9").doc("Width over height of the plot."),
                    prop("loading", "bool").default("false").doc("A placeholder instead of the plot, and `aria-busy` on the chart."),
                    prop("empty", "Option<Element>").doc("Shown in the plot when there is no value at all. Unset, \"No data\"."),
                    prop("tooltip", "Option<bool>").default(on(theme.chart.tooltip)).doc("A tooltip at the pointer's or the keys' bar."),
                    prop("tooltip_content", "Option<Callback<ChartPoint, Element>>").doc("The tooltip's content instead of the category and each series' value there."),
                    prop("onpointclick", "Option<EventHandler<ChartPoint>>").doc("A click on a bar, a second tap on a pinned one, or Enter on the keys' bar."),
                    prop("onactivechange", "Option<EventHandler<Option<ChartPoint>>>").doc("The bar under the pointer or the keys; `None` when it leaves."),
                    prop("hidden", "Option<Vec<String>>").doc("The names of the series the legend hides; set, they are controlled."),
                    prop("default_hidden", "Vec<String>").default("[]").doc("Seeds the hidden series once. Ignored when `hidden` is set."),
                    prop("onhiddenchange", "Option<EventHandler<Vec<String>>>").doc("The hidden series a legend button asks for. The last shown series stays."),
                    prop("zoom", "Option<bool>").default(on(theme.chart.zoom)).doc("Zoom and pan along the categories: Ctrl or Cmd with the wheel, a pinch, a drag (Shift or Alt with it once `onrange` is set), the keys, and a Reset zoom button."),
                    prop("onrange", "Option<EventHandler<Option<ChartRange>>>").doc("Set, a drag or Shift with an arrow brushes a band of categories; Shift or Alt with a drag pans instead. `None` when Escape clears it."),
                    prop("handle", "Option<ChartHandle>").doc("From `use_chart()`: reads the drawing as SVG or PNG, saves it, resets the zoom."),
                    prop("export_button", "Option<bool>").default(on(theme.chart.export_button)).doc("Save as SVG and Save as PNG buttons above the plot."),
                ]),
                props("PieChart", vec![
                    prop("slices", "Vec<Slice>").default("required").doc("The slices, each a share of their sum. A negative value takes no share."),
                    prop("title", "Option<String>").doc("Shown above the chart, and its accessible name."),
                    prop("description", "Option<String>").doc("A sentence under the title: what to read from the chart."),
                    prop("donut", "bool").default("false").doc("A ring instead of a full pie, `center` in its middle."),
                    prop("labels", "PieLabels").default(theme.chart.pie_labels.as_str()).doc("Beside each slice: `percent`, `value`, `name` or `none`. One too long for the room ends in an ellipsis; the tooltip and the summary write it whole."),
                    prop("center", "Option<Element>").doc("In a donut's hole, as a total, kept off the ring; its text shrinks in a small hole, the rest is clipped."),
                    prop("start_angle", "f32").default("0").doc("Where the first slice starts, in degrees clockwise from 12 o'clock."),
                    prop("end_angle", "Option<f32>").default("a full turn").doc("Where the last slice ends, in the same degrees: `start_angle: -90.0, end_angle: 90.0` is a half pie, a gauge with `donut`. At or before the start, it counts a turn on: `90.0` to `-90.0` is the lower half."),
                    prop("rose", "bool").default("false").doc("A nightingale rose: every slice the same angle, its radius by its value."),
                    prop("label_position", "PieLabelPosition").default(theme.chart.pie_label_position.as_str()).doc("`outside` beside each slice, or `inside` on it in the ink that reads there. Inside, a label wider than its slice is left out; the summary keeps it."),
                    prop("min_label_angle", "Option<f32>").default(theme.chart.min_label_angle.to_string()).doc("A slice narrower than this many degrees gets no label; the summary still names it."),
                    prop("slice_gap", "Option<f32>").default(theme.chart.slice_gap.to_string()).doc("Between two slices, in pixels, in the page's colour."),
                    prop("legend", "LegendPosition").default("bottom with two slices or more").doc("`bottom`, `top`, `start`, `end` or `none`."),
                    prop("value_format", "ValueFormat").default("Auto").doc("How values are written beside the slices, in the tooltip and in the summary."),
                    prop("unit", "Option<String>").doc("After every value, as `\"kg\"`."),
                    prop("grid", "Option<bool>").doc("Shared with the other charts; a pie has no grid to draw."),
                    prop("aspect_ratio", "Option<f32>").default("16 / 9").doc("Width over height of the plot."),
                    prop("loading", "bool").default("false").doc("A placeholder instead of the plot, and `aria-busy` on the chart."),
                    prop("empty", "Option<Element>").doc("Shown in the plot when the slices sum to nothing. Unset, \"No data\"."),
                    prop("tooltip", "Option<bool>").default(on(theme.chart.tooltip)).doc("A tooltip with the slice's value and share at the pointer's or the keys' slice."),
                    prop("tooltip_content", "Option<Callback<ChartPoint, Element>>").doc("The tooltip's content instead of the slice's name, value and share."),
                    prop("onpointclick", "Option<EventHandler<ChartPoint>>").doc("A click on a slice, a second tap on a pinned one, or Enter on the keys' slice. `index` is the slice's place in `slices`, `x` and `series_name` its name."),
                    prop("onactivechange", "Option<EventHandler<Option<ChartPoint>>>").doc("The slice under the pointer or the keys; `None` when it leaves."),
                    prop("hidden", "Option<Vec<String>>").doc("The names of the slices the legend hides; set, they are controlled."),
                    prop("default_hidden", "Vec<String>").default("[]").doc("Seeds the hidden slices once. Ignored when `hidden` is set."),
                    prop("onhiddenchange", "Option<EventHandler<Vec<String>>>").doc("The hidden slices a legend button asks for. The last shown slice stays."),
                    prop("handle", "Option<ChartHandle>").doc("From `use_chart()`: reads the drawing as SVG or PNG, saves it, resets the zoom."),
                    prop("export_button", "Option<bool>").default(on(theme.chart.export_button)).doc("Save as SVG and Save as PNG buttons above the plot."),
                ]),
            ],
            accessibility: a11y()
                .key(["Tab"], "Into the plot and the legend's buttons in the order drawn: a legend at the top or the start comes first.")
                .key(["ArrowRight", "ArrowLeft"], "Line: the next or previous point of the line, as drawn, left to right in either text direction. Bar: the next or previous category's bar of the series; `horizontal`, the next or previous series' bar in the category. Pie: the next or previous slice, clockwise.")
                .key(["ArrowDown", "ArrowUp"], "Line: the next or previous line at the same x value, or its nearest point. Bar: the next or previous series' bar in the category; `horizontal`, the next or previous category, top to bottom. Pie: the next or previous slice.")
                .key(["Home", "End"], "The line's or the series' first or last point or bar; a pie's first or last slice.")
                .key(["Enter", "Space"], "Calls `onpointclick` with the point, bar or slice.")
                .key(["Escape"], "Clears the brushed band, else the point and its tooltip.")
                .key(["+", "-"], "With `zoom`: zooms the x axis, a lying bar chart's categories, in or out around the keys' point; the arrows carry the window along at its edge.")
                .key(["0"], "With `zoom`: shows every x value or category again.")
                .key(["Shift+ArrowRight", "Shift+ArrowLeft"], "With `onrange`: brushes from the keys' point to the next or previous x value.")
                .handles([
                    "The chart is a `role=\"figure\"` named by `title`. The drawing is hidden from screen readers.",
                    "A hidden summary gives the x range and each series' lowest and highest value, and where the highest is, then each mark line's name and value and each marked point's place and value, each line's error band and each upright grouped series' error bars with their outer ends, the series on the right y axis, the axes' titles and, stacked, that the top line is the total; a pie's, the total, then each slice's value and share.",
                    "The plot is one tab stop, a `role=\"application\"` described as a chart, so a screen reader hands it the arrows. Each point the keys reach is announced, as \"This week, Tue: 132\" or \"Chrome: 62, 62%.\"",
                    "The legend's entries are toggle buttons: pressed while their series or slice shows, struck through when hidden. The last shown one's button is `aria-disabled`. Should `hidden` or `default_hidden` name every one, the plot and the summary say \"All series are hidden.\" and the legend stays. A dashed line's key is a dash, as drawn; once a line sets a `symbol`, each key is its line's point shape.",
                    "On a touch screen a tap pins a point's tooltip, a second tap on it clicks; a vertical swipe still scrolls the page.",
                    "With `zoom`, the shown range is announced once a zoom or pan settles, as \"Showing Tue to Fri\"; the Reset zoom button stays focusable while there is nothing to reset. Ctrl or Cmd with the wheel zooms only a zoomable plot, so the page zoom works everywhere else.",
                    "A drag with `onrange` set brushes; Shift or Alt with it pans. The band is outlined as well as tinted, and announced as \"Selected Tue to Thu\".",
                    "A saved SVG is self-contained: its colours resolved, its font named, the chart's `title` as its `<title>`. The save buttons announce what they saved.",
                    "Each series' or slice's palette colour reaches 3:1 on the page and on a card, in every shipped theme and color scheme; one too close to an earlier one, also for colour-blind eyes, is skipped.",
                    "In forced colours the lines, bars and slices keep their colours on the system background; axes and labels take the system text colour.",
                    "A bar's or a point's error range is written after its value in the tooltip and the announcement, as \"120 (110–130)\".",
                    "A label on a bar or a slice takes black or white, whichever reads on it.",
                ])
                .must([
                    "Give a `title`, or name the chart with `aria_label`.",
                    "Say in `description` what the reader should see, as \"Visitors peaked on Saturday\".",
                    "Tell series apart by more than colour: `dashed()` or `symbol()` on a line, the legend, `value_labels` on bars, `labels` on a pie.",
                    "Name every series and slice differently: the legend hides them by name.",
                    "Keep a pie to a few slices; fold the small ones into one \"Other\".",
                    "Where every value matters, show the data in a `Table` beside the chart too.",
                ])
                .limits([
                    "A hex colour (`.color(\"#1e88e5\")`) is drawn as given: check its 3:1 on your page.",
                    "The tooltip is for the eye; screen readers hear the announcement instead, so `tooltip_content` reaches only sighted readers.",
                    "The drawing runs left to right in a right-to-left page too, and the arrows follow it unflipped.",
                    "A hidden slice leaves the drawing, so the shown ones' shares grow; the summary keeps every slice.",
                    "`grid` draws nothing on a pie.",
                    "`.colors` and `.negative_color` add no meaning of their own: a loss already hangs below the axis. The legend keeps the series' colour.",
                    "Error bars draw on upright grouped bars only; a stack or a lying chart reads the range in its tooltip alone. A line draws its ranges as a band, only around its points and not when `stacked`.",
                    "A rose's radius and a slice under `min_label_angle` are read by eye or not at all; the summary gives every value.",
                    "`zoom` and `onrange` are for lines and bars; a pie exports only.",
                    "An export keeps the normal palette in forced colours. `png()` is `None` on Blitz and on the server, and the PNG button says it could not save.",
                    "A mark line is drawn on a line chart that is not `stacked` and on upright, `grouped` bars, and only inside the y axis' ends; the summary leaves out the ones not drawn, except past the ends. A marked point is drawn on a line chart only.",
                    "A right y axis is drawn on a line chart only; `y_min` and `y_max` set the left one, and a `stacked` chart stacks only the left one's lines. Give its series a `symbol` or `dashed()` and a `y2_title`, so the reader can tell which axis a line is read on.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "LineChart" }
                    ", "
                    Code { source: "BarChart" }
                    " and "
                    Code { source: "PieChart" }
                    ", each drawn as an SVG that fills its width, with a tooltip, the arrow keys, a legend that hides and shows, and a summary for screen readers. Needs the "
                    Code { source: "charts" }
                    " feature. Web and Android."
                }
                Text {
                    "Lines and bars take one or more "
                    Code { source: "Series" }
                    " over a shared x axis; a pie takes "
                    Code { source: "Slice" }
                    "s, each a name and a value. A line of another size or unit goes on a right y axis with "
                    Code { source: ".secondary_axis()" }
                    "."
                }
                Text {
                    "Lines and bars zoom and pan their x axis with "
                    Code { source: "zoom" }
                    " and brush a range for "
                    Code { source: "onrange" }
                    ". Every chart exports as SVG or PNG through a "
                    Code { source: "use_chart()" }
                    " handle or the "
                    Code { source: "export_button" }
                    "s."
                }
            },
            // snippet: let mut selected = use_signal(|| None::<ChartRange>);
            // snippet: let chart = use_chart();
            Demo {
                component: "Chart",
                children_text: "",
                wrap: Wrap(component),
                controls: vec![
                    Control::toggle("type", ["line", "bar", "pie"])
                        .default("line")
                        .labels(["LineChart", "BarChart", "PieChart"])
                        .code(|_, values| {
                            let mark = marks_code(values);
                            match values.str("type").as_str() {
                            "bar" => vec![
                                "title: \"Profit\"".to_string(),
                                format!("x: {QUARTERS:?}"),
                                format!(
                                    "series: vec![Series::new(\"2025\", {PROFIT_2025:?}){mark}, Series::new(\"2026\", {PROFIT_2026:?})]"
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
                                    "series: vec![Series::new(\"This week\", {THIS_WEEK:?}){mark}, Series::new(\"Last week\", {LAST_WEEK:?}).dashed().symbol(PointSymbol::Square){}{}]",
                                    last_week(values).0,
                                    conversion(values).map(|c| c.0).unwrap_or_default()
                                ),
                            ],
                        }}),
                    Control::select("curve", ["linear", "smooth", "step"])
                        .default(theme.chart.curve.as_str())
                        .hidden_when(is("line")),
                    Control::toggle("area", ["filled", "stacked", "none"])
                        .default("filled")
                        .hidden_when(is("line"))
                        .code(|_, values| match values.str("area").as_str() {
                            "filled" => vec!["area: true".to_string()],
                            "stacked" => vec!["area: true".to_string(), "stacked: true".to_string()],
                            _ => vec![],
                        }),
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
                    // A position shows its labels without the switch.
                    Control::select("value_label_position", ["auto", "inside", "outside"])
                        .default("auto")
                        .hidden_when(is("bar"))
                        .code(|_, values| match values.str("value_label_position").as_str() {
                            "auto" => vec![],
                            position => {
                                let mut lines = vec![format!("value_label_position: \"{position}\"")];
                                if values.str("value_labels") != "true" {
                                    lines.insert(0, "value_labels: true".to_string());
                                }
                                lines
                            }
                        }),
                    Control::slider("bar_width", ["auto", "0.4", "0.6", "0.8", "1.0"])
                        .default("auto")
                        .hidden_when(is("bar"))
                        .code(|_, values| match values.str("bar_width").as_str() {
                            "auto" => vec![],
                            share => vec![format!("bar_width: {share}")],
                        }),
                    Control::select("labels", ["percent", "value", "name", "none"])
                        .default(theme.chart.pie_labels.as_str())
                        .hidden_when(is("pie")),
                    Control::toggle("label_position", ["outside", "inside"])
                        .default(theme.chart.pie_label_position.as_str())
                        .hidden_when(is("pie")),
                    Control::slider("min_label_angle", ["0", "15", "30", "45"])
                        .default("0")
                        .hidden_when(is("pie"))
                        .code(|_, values| match values.str("min_label_angle").as_str() {
                            "0" => vec![],
                            angle => vec![format!("min_label_angle: {angle}.0")],
                        }),
                    Control::switch("rose").hidden_when(is("pie")),
                    Control::select("end_angle", ["full", "half"])
                        .default("full")
                        .labels(["Full turn", "Half: -90 to 90"])
                        .hidden_when(is("pie"))
                        .code(|_, values| match values.str("end_angle").as_str() {
                            "half" => vec![
                                "start_angle: -90.0".to_string(),
                                "end_angle: 90.0".to_string(),
                            ],
                            _ => vec![],
                        }),
                    Control::slider("slice_gap", ["0", "1", "2", "4", "8"])
                        .default("1")
                        .hidden_when(is("pie"))
                        .code(|_, values| match values.str("slice_gap").as_str() {
                            "1" => vec![],
                            gap => vec![format!("slice_gap: {gap}.0")],
                        }),
                    Control::switch("donut").hidden_when(is("pie")).code(|_, values| {
                        match values.str("donut").as_str() {
                            "true" => vec![
                                "donut: true".to_string(),
                                "center: rsx! { \"1,204 users\" }".to_string(),
                            ],
                            _ => vec![],
                        }
                    }),
                    Control::toggle("mark_point", ["max", "min", "both", "none"])
                        .default("max")
                        .hidden_when(is("line"))
                        .code(|_, _| vec![]),
                    Control::toggle("mark_line", ["average", "target", "none"])
                        .default("average")
                        .hidden_when(is("axes"))
                        .code(|_, _| vec![]),
                    Control::switch("error_band")
                        .default("true")
                        .hidden_when(is("line"))
                        .code(|_, _| vec![]),
                    Control::switch("secondary_axis")
                        .default("true")
                        .hidden_when(is("line"))
                        .code(|_, _| vec![]),
                    Control::select("y_range", ["fixed", "auto"]).hidden_when(is("axes")).code(
                        |_, values| match y_range(values) {
                            Some((min, max)) => vec![format!("y_min: {min:?}"), format!("y_max: {max:?}")],
                            None => vec![],
                        },
                    ),
                    Control::slider("y_ticks", ["2", "3", "4", "5", "6", "8", "10"])
                        .default(theme.chart.y_ticks.to_string())
                        .hidden_when(is("axes"))
                        .code(|control, values| match values.str("y_ticks") {
                            ticks if ticks == control.default => vec![],
                            ticks => vec![format!("y_ticks: {ticks}")],
                        }),
                    Control::switch("axis_titles")
                        .default("true")
                        .hidden_when(is("axes"))
                        .code(|_, values| match axis_titles(values) {
                            Some((x, y)) => {
                                let mut lines = vec![format!("x_title: {x:?}"), format!("y_title: {y:?}")];
                                if conversion(values).is_some() {
                                    lines.push("y2_title: \"Conversion\"".to_string());
                                }
                                lines
                            }
                            None => vec![],
                        }),
                    Control::select("legend", legend),
                    Control::switch("tooltip").default(on(theme.chart.tooltip)),
                    Control::switch("zoom").default("true").hidden_when(|values| values.str("type") == "pie").code(|_, values| {
                        match (values.str("zoom").as_str(), values.str("type").as_str()) {
                            ("true", "line" | "bar") => vec![
                                "zoom: true".to_string(),
                                "onrange: move |range| selected.set(range)".to_string(),
                            ],
                            _ => vec![],
                        }
                    }),
                    Control::switch("export_button").default("true").code(|_, values| {
                        match values.str("export_button").as_str() {
                            "true" => vec!["export_button: true".to_string(), "handle: chart".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("loading"),
                ],
                render: move |values: DemoValues| {
                    let legend = LegendPosition::from(values.str("legend").as_str());
                    let tooltip = values.str("tooltip") == "true";
                    let loading = values.str("loading") == "true";
                    let range = y_range(&values);
                    let (y_min, y_max) = (range.map(|r| r.0), range.map(|r| r.1));
                    let y_ticks = values.str("y_ticks").parse::<usize>().ok();
                    let titles = axis_titles(&values);
                    let y2_title = titles.and(conversion(&values)).map(|_| "Conversion".to_string());
                    let (x_title, y_title) = (
                        titles.map(|t| t.0.to_string()),
                        titles.map(|t| t.1.to_string()),
                    );
                    let zoom = values.str("zoom") == "true";
                    let export_button = values.str("export_button") == "true";
                    let onrange = zoom.then_some(EventHandler::new(move |range: Option<ChartRange>| selected.set(range)));
                    match values.str("type").as_str() {
                        "bar" => brushed(zoom, selected, rsx! {
                            BarChart {
                                title: "Profit",
                                x: QUARTERS,
                                series: vec![marked(&values, Series::new("2025", PROFIT_2025)), Series::new("2026", PROFIT_2026)],
                                layout: BarLayout::from(values.str("layout").as_str()),
                                orientation: Orientation::from(values.str("orientation").as_str()),
                                radius: values.str("radius"),
                                value_labels: values.str("value_labels") == "true"
                                    || values.str("value_label_position") != "auto",
                                value_label_position: match values.str("value_label_position").as_str() {
                                    "auto" => None,
                                    position => Some(BarLabelPosition::from(position)),
                                },
                                bar_width: values.str("bar_width").parse::<f32>().ok(),
                                y_min, y_max, y_ticks, x_title, y_title,
                                                                legend, tooltip, loading, zoom, onrange, export_button, handle: chart,
                            }
                        }),
                        "pie" => {
                            let half = values.str("end_angle") == "half";
                            rsx! {
                                PieChart {
                                    title: "Browsers",
                                    slices: slices(),
                                    labels: PieLabels::from(values.str("labels").as_str()),
                                    label_position: PieLabelPosition::from(values.str("label_position").as_str()),
                                    min_label_angle: values.str("min_label_angle").parse::<f32>().ok(),
                                    donut: values.str("donut") == "true",
                                    rose: values.str("rose") == "true",
                                    start_angle: if half { -90.0 } else { 0.0 },
                                    end_angle: half.then_some(90.0),
                                    slice_gap: values.str("slice_gap").parse::<f32>().ok(),
                                    center: rsx! { "1,204 users" },
                                    legend, tooltip, loading, export_button, handle: chart,
                                }
                            }
                        }
                        _ => brushed(zoom, selected, rsx! {
                            LineChart {
                                title: "Visitors",
                                x: DAYS,
                                series: vec![
                                    marked(&values, Series::new("This week", THIS_WEEK)),
                                    last_week(&values).1,
                                ].into_iter().chain(conversion(&values).map(|c| c.1)).collect::<Vec<_>>(),
                                curve: Curve::from(values.str("curve").as_str()),
                                area: values.str("area") != "none",
                                stacked: values.str("area") == "stacked",
                                y_min, y_max, y_ticks, x_title, y_title, y2_title,
                                                                legend, tooltip, loading, zoom, onrange, export_button, handle: chart,
                            }
                        }),
                    }
                },
            }
            DocSection { title: "Bar colours and error ranges",
                Text {
                    "A series can colour its bars one by one with "
                    Code { source: ".colors([..])" }
                    ", or its losses with "
                    Code { source: ".negative_color(..)" }
                    "; the bar's place below the axis says the same. "
                    Code { source: ".errors([(low, high), ..])" }
                    " draws a range over each upright bar of a grouped chart, and the tooltip and the keys read it out. On a line it is a see-through band in the line's colour."
                }
                ProfitRanges {}
                Ranking {}
            }
            DocSection { title: "Half donut and rose",
                Text {
                    Code { source: "end_angle" }
                    " ends the slices before a full turn: from -90 to 90 degrees a "
                    Code { source: "donut" }
                    " becomes a gauge, its "
                    Code { source: "center" }
                    " under the arc. "
                    Code { source: "rose" }
                    " gives every slice the same angle and its value as its radius."
                }
                Gauge {}
                Rose {}
            }
            DocSection { title: "With a table",
                Text {
                    "A chart only draws its data. Where every value matters, show the same rows in a "
                    Anchor { to: Route::TablePage {}, "Table" }
                    " beside it: a screen reader can read it cell by cell, and a reader can sort it."
                }
                SalesWithTable {}
            }
        }
    }
}

/// Losses in their own colour, a range over every bar.
#[component]
fn ProfitRanges() -> Element {
    rsx! {
        BarChart {
            title: "Profit with its range",
            x: QUARTERS,
            series: vec![
                Series::new("2025", PROFIT_2025)
                    .negative_color("error")
                    .errors([(30.0, 52.0), (44.0, 63.0), (-31.0, -19.0), (50.0, 71.0)]),
            ],
            bar_width: 0.6,
            value_labels: true,
        }
    }
}

/// A colour per bar, the leader first.
#[component]
fn Ranking() -> Element {
    rsx! {
        BarChart {
            title: "Sales per team",
            x: ["North", "South", "East"],
            series: vec![
                Series::new("Sales", [162.0, 128.0, 101.0])
                    .colors(["success", "primary", "neutral"]),
            ],
            orientation: "horizontal",
            value_labels: true,
        }
    }
}

/// A half donut with its total under the arc.
#[component]
fn Gauge() -> Element {
    rsx! {
        PieChart {
            title: "Storage used",
            slices: vec![Slice::new("Used", 72.0), Slice::new("Free", 28.0).color("neutral.3")],
            donut: true,
            start_angle: -90.0,
            end_angle: 90.0,
            label_position: "inside",
            center: rsx! { "72 GB" },
        }
    }
}

/// Every browser the same angle, its share as its radius.
#[component]
fn Rose() -> Element {
    rsx! {
        PieChart {
            title: "Browsers as a rose",
            slices: slices(),
            rose: true,
            labels: "name",
        }
    }
}

/// The demo's `chart`, with the brushed band under it while it zooms.
fn brushed(zoom: bool, selected: Signal<Option<ChartRange>>, chart: Element) -> Element {
    let text = match &*selected.read() {
        Some(range) => format!("Selected {} to {}", range.from, range.to),
        None => "Drag across the plot to select a range.".to_string(),
    };
    rsx! {
        div { width: "100%",
            {chart}
            if zoom {
                Text { size: "sm", "{text}" }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct Sales {
    quarter: &'static str,
    last: f64,
    this: f64,
}

/// The bar chart's quarters, and the same rows as a table.
#[component]
fn SalesWithTable() -> Element {
    let rows: Vec<Sales> = (0..QUARTERS.len())
        .map(|i| Sales {
            quarter: QUARTERS[i],
            last: SALES_2025[i],
            this: SALES_2026[i],
        })
        .collect();
    rsx! {
        BarChart {
            title: "Sales",
            x: XValues::from_rows(&rows, |row| row.quarter),
            series: vec![
                Series::from_rows("2025", &rows, |row| row.last),
                Series::from_rows("2026", &rows, |row| row.this),
            ],
        }
        Table {
            caption: "Sales",
            data: rows,
            columns: vec![
                column("Quarter").value(|row: &Sales| row.quarter.to_string()).row_header(),
                column("2025").value(|row: &Sales| row.last).sortable(),
                column("2026").value(|row: &Sales| row.this).sortable(),
            ],
        }
    }
}
