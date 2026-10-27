// page-snippets: charts
use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, or_unset, prop, props};
use dioxus::prelude::*;
use libero::use_theme;
use libero::{
    components::{
        BarChart, Button, Candle, CandlestickChart, ChartHandle, ChartRange, Code, Flex,
        FunnelChart, HeatmapChart, LineChart, MarkLine, MarkPoint, Orientation, PieChart,
        RadarAxis, RadarChart, ScatterChart, ScatterSeries, Series, SeriesKind, Slice, Table, Text,
        Trend, column, use_chart,
    },
    theme::{
        BarLabelPosition, BarLayout, Curve, LegendPosition, PieLabelPosition, PieLabels,
        PointSymbol, Size,
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
const PROFIT_2025: [f64; 4] = [40.0, 55.0, -25.0, 62.0];
const PROFIT_2025_RANGE: [(f64, f64); 4] =
    [(30.0, 52.0), (44.0, 63.0), (-31.0, -19.0), (50.0, 71.0)];
const PROFIT_2026: [f64; 4] = [30.0, -15.0, 20.0, 48.0];
const MARGIN: [f64; 4] = [18.0, 9.0, 12.0, 24.0];
const QUARTER_COLORS: [&str; 4] = ["success", "info", "warning", "neutral"];
const BROWSERS: [(&str, f64); 4] = [
    ("Chrome", 62.0),
    ("Safari", 20.0),
    ("Firefox", 10.0),
    ("Other", 8.0),
];
const STAGES: [(&str, f64); 5] = [
    ("Visits", 1200.0),
    ("Product views", 860.0),
    ("Carts", 480.0),
    ("Checkouts", 300.0),
    ("Orders", 210.0),
];
/// A shop's weeks: ad spend and sales in k€, and orders as the bubble's size.
const ONLINE: [(f64, f64, f64); 7] = [
    (12.0, 34.0, 80.0),
    (18.0, 41.0, 120.0),
    (25.0, 52.0, 90.0),
    (31.0, 58.0, 150.0),
    (38.0, 71.0, 200.0),
    (44.0, 76.0, 140.0),
    (52.0, 90.0, 240.0),
];
const STORES: [(f64, f64, f64); 7] = [
    (8.0, 22.0, 50.0),
    (15.0, 30.0, 70.0),
    (22.0, 33.0, 110.0),
    (28.0, 45.0, 60.0),
    (35.0, 47.0, 130.0),
    (41.0, 55.0, 100.0),
    (48.0, 60.0, 160.0),
];
const HOURS: [&str; 6] = ["8", "10", "12", "14", "16", "18"];
const WEEKDAYS: [&str; 5] = ["Mon", "Tue", "Wed", "Thu", "Fri"];
const VISITS: [[f64; 6]; 5] = [
    [12.0, 30.0, 41.0, 35.0, 22.0, 9.0],
    [15.0, 34.0, 46.0, 38.0, 25.0, 11.0],
    [11.0, 28.0, 52.0, 44.0, 27.0, 10.0],
    [14.0, 31.0, 39.0, 33.0, 29.0, 16.0],
    [10.0, 24.0, 33.0, 28.0, 31.0, 21.0],
];
const MONTHS: [&str; 8] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug"];
/// A share's months: open, high, low and close in €.
const PRICES: [(f64, f64, f64, f64); 8] = [
    (32.0, 38.0, 30.0, 36.0),
    (36.0, 41.0, 34.0, 35.0),
    (35.0, 37.0, 29.0, 30.0),
    (30.0, 34.0, 28.0, 33.0),
    (33.0, 44.0, 32.0, 43.0),
    (43.0, 46.0, 40.0, 41.0),
    (41.0, 43.0, 39.0, 41.0),
    (41.0, 50.0, 39.0, 48.0),
];
/// Two phones' review scores out of 10.
const FEATURES: [&str; 5] = ["Battery", "Camera", "Screen", "Speed", "Value"];
const PHONE_A: [f64; 5] = [8.0, 6.0, 7.0, 9.0, 5.0];
const PHONE_B: [f64; 5] = [6.0, 9.0, 8.0, 6.0, 7.0];

fn on(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn slices() -> Vec<Slice> {
    BROWSERS.into_iter().map(Slice::from).collect()
}

fn stages() -> Vec<Slice> {
    STAGES.into_iter().map(Slice::from).collect()
}

fn candles() -> Vec<Candle> {
    PRICES.into_iter().map(Candle::from).collect()
}

fn names(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| name.to_string()).collect()
}

/// Hides a control unless the type is `kind`; `axes` hides it on a pie, a funnel, a heatmap
/// and a radar, `shares` shows it on a pie and a funnel, `series` on lines and bars only.
fn is(kind: &'static str) -> fn(&DemoValues) -> bool {
    match kind {
        "bar" => |values| values.str("type") != "bar",
        "pie" => |values| values.str("type") != "pie",
        "scatter" => |values| values.str("type") != "scatter",
        "heatmap" => |values| values.str("type") != "heatmap",
        "candlestick" => |values| values.str("type") != "candlestick",
        "radar" => |values| values.str("type") != "radar",
        "axes" => |values| {
            matches!(
                values.str("type").as_str(),
                "pie" | "funnel" | "heatmap" | "radar"
            )
        },
        "shares" => |values| !matches!(values.str("type").as_str(), "pie" | "funnel"),
        "series" => |values| !matches!(values.str("type").as_str(), "line" | "bar"),
        "zoom" => |values| !matches!(values.str("type").as_str(), "line" | "bar" | "candlestick"),
        _ => |values| values.str("type") != "line",
    }
}

/// The scatter chart's series, as bubbles when the switch says, in code and as built.
fn scatter_series(values: &DemoValues) -> (String, Vec<ScatterSeries>) {
    let bubbles = values.str("bubbles") == "true";
    let series = [("Online", ONLINE), ("Stores", STORES)].map(|(name, points)| {
        let pairs: Vec<(f64, f64)> = points.iter().map(|p| (p.0, p.1)).collect();
        match bubbles {
            true => (
                format!("ScatterSeries::new({name:?}, {points:?})"),
                ScatterSeries::new(name, points),
            ),
            false => (
                format!("ScatterSeries::new({name:?}, {pairs:?})"),
                ScatterSeries::new(name, pairs),
            ),
        }
    });
    let code = series.iter().map(|s| s.0.as_str()).collect::<Vec<_>>();
    (
        format!("series: vec![{}]", code.join(", ")),
        series.into_iter().map(|s| s.1).collect(),
    )
}

/// The scatter chart's trend line, in code and as built.
fn trend(values: &DemoValues) -> Option<(&'static str, Trend)> {
    match values.str("trend").as_str() {
        "linear" => Some(("Trend::Linear", Trend::Linear)),
        "exponential" => Some(("Trend::Exponential", Trend::Exponential)),
        "logarithmic" => Some(("Trend::Logarithmic", Trend::Logarithmic)),
        "polynomial" => Some(("Trend::Polynomial(2)", Trend::Polynomial(2))),
        _ => None,
    }
}

/// The value axis' fixed ends, `y_min` and `y_max`, for the chart type.
fn y_range(values: &DemoValues) -> Option<(f64, f64)> {
    match (values.str("y_range").as_str(), values.str("type").as_str()) {
        ("auto", _) => None,
        (_, "bar") => Some((-50.0, 150.0)),
        (_, "scatter") => Some((0.0, 100.0)),
        (_, "candlestick") => Some((20.0, 60.0)),
        _ if values.str("area") == "stacked" => Some((0.0, 400.0)),
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
        (true, "scatter") => Some(("Ad spend (k€)", "Sales (k€)")),
        (true, "candlestick") => Some(("Month", "Price (€)")),
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

/// The bar chart's first series, coloured and ranged as the controls say, in code and as built.
fn profit_2025(values: &DemoValues) -> (String, Series) {
    let (mut code, mut series) = (String::new(), Series::new("2025", PROFIT_2025));
    match values.str("colors").as_str() {
        "per_bar" => {
            code += &format!(".colors({QUARTER_COLORS:?})");
            series = series.colors(QUARTER_COLORS);
        }
        "negative" => {
            code += ".negative_color(\"error\")";
            series = series.negative_color("error");
        }
        _ => {}
    }
    if values.str("error_bars") == "true" {
        code += &format!(".errors({PROFIT_2025_RANGE:?})");
        series = series.errors(PROFIT_2025_RANGE);
    }
    (code, series)
}

/// A table row: what the chart's x axis or slice names it, then each series' value.
type Row = (&'static str, Vec<f64>);

/// The chart's data for a table: the caption, the first column's header, the value columns'
/// headers and the rows.
fn table(values: &DemoValues) -> (&'static str, &'static str, Vec<&'static str>, Vec<Row>) {
    let rows = |x: &[&'static str], series: &[&[f64]]| {
        x.iter()
            .enumerate()
            .map(|(i, x)| (*x, series.iter().map(|s| s[i]).collect()))
            .collect()
    };
    match values.str("type").as_str() {
        "bar" if margin(values).is_some() => (
            "Profit",
            "Quarter",
            vec!["2025", "2026", "Margin"],
            rows(&QUARTERS, &[&PROFIT_2025, &PROFIT_2026, &MARGIN]),
        ),
        "bar" => (
            "Profit",
            "Quarter",
            vec!["2025", "2026"],
            rows(&QUARTERS, &[&PROFIT_2025, &PROFIT_2026]),
        ),
        "pie" => (
            "Browsers",
            "Browser",
            vec!["Share"],
            BROWSERS
                .iter()
                .map(|(name, share)| (*name, vec![*share]))
                .collect(),
        ),
        "funnel" => (
            "Checkout",
            "Stage",
            vec!["Visitors"],
            STAGES
                .iter()
                .map(|(name, count)| (*name, vec![*count]))
                .collect(),
        ),
        "scatter" => (
            "Ad spend and sales",
            "Channel",
            vec!["Ad spend", "Sales", "Orders"],
            [("Online", ONLINE), ("Stores", STORES)]
                .iter()
                .flat_map(|(name, points)| points.iter().map(|p| (*name, vec![p.0, p.1, p.2])))
                .collect(),
        ),
        "heatmap" => (
            "Visits by hour",
            "Day",
            HOURS.to_vec(),
            WEEKDAYS
                .iter()
                .zip(VISITS)
                .map(|(day, row)| (*day, row.to_vec()))
                .collect(),
        ),
        "candlestick" => (
            "Share price",
            "Month",
            vec!["Open", "High", "Low", "Close"],
            MONTHS
                .iter()
                .zip(PRICES)
                .map(|(month, (o, h, l, c))| (*month, vec![o, h, l, c]))
                .collect(),
        ),
        "radar" => (
            "Phones",
            "Feature",
            vec!["Phone A", "Phone B"],
            rows(&FEATURES, &[&PHONE_A, &PHONE_B]),
        ),
        _ if conversion(values).is_some() => (
            "Visitors",
            "Day",
            vec!["This week", "Last week", "Conversion"],
            rows(&DAYS, &[&THIS_WEEK, &LAST_WEEK, &CONVERSION]),
        ),
        _ => (
            "Visitors",
            "Day",
            vec!["This week", "Last week"],
            rows(&DAYS, &[&THIS_WEEK, &LAST_WEEK]),
        ),
    }
}

/// The `Table` the `table` switch puts under the chart, as code.
fn table_code(values: &DemoValues) -> String {
    let (caption, first, names, rows) = table(values);
    let row = format!("&(&str{})", ", f64".repeat(names.len()));
    let data = rows
        .iter()
        .map(|(x, cells)| {
            let cells: String = cells.iter().map(|cell| format!(", {cell:?}")).collect();
            format!("        ({x:?}{cells}),\n")
        })
        .collect::<String>();
    let columns = names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            format!(
                "        column({name:?}).value(|row: {row}| row.{}).sortable(),\n",
                i + 1
            )
        })
        .collect::<String>();
    format!(
        "Table {{\n    caption: {caption:?},\n    data: vec![\n{data}    ],\n    columns: vec![\n        column({first:?}).value(|row: {row}| row.0.to_string()).row_header(),\n{columns}    ],\n}}"
    )
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

/// The bar chart's line over its bars, on the axis the control says, in code and as built.
fn margin(values: &DemoValues) -> Option<(String, Series)> {
    let axis = values.str("line_series");
    if values.str("type") != "bar" || axis == "none" {
        return None;
    }
    let series = Series::new("Margin", MARGIN)
        .unit("%")
        .kind(SeriesKind::Line);
    Some(match axis.as_str() {
        "right" => (
            format!(
                ", Series::new(\"Margin\", {MARGIN:?}).unit(\"%\").kind(SeriesKind::Line).secondary_axis()"
            ),
            series.secondary_axis(),
        ),
        _ => (
            format!(", Series::new(\"Margin\", {MARGIN:?}).unit(\"%\").kind(SeriesKind::Line)"),
            series,
        ),
    })
}

/// The right y axis' title, when a series stands on it; a percent chart reads its line there.
fn y2_title(values: &DemoValues) -> Option<&'static str> {
    let percent = values.str("layout") == "percent";
    match (conversion(values), values.str("line_series").as_str()) {
        (Some(_), _) => Some("Conversion"),
        (None, "right") if values.str("type") == "bar" => Some("Margin"),
        (None, "left") if values.str("type") == "bar" && percent => Some("Margin"),
        _ => None,
    }
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

/// Whether the chart is a line or bar chart with `zoom` on.
fn zoomable(values: &DemoValues) -> bool {
    values.str("zoom") == "true"
        && matches!(values.str("type").as_str(), "line" | "bar" | "candlestick")
}

/// The buttons under the chart, as code: save as SVG and PNG, and Reset zoom with `zoom`.
fn buttons_code(zoom: bool) -> String {
    let mut code = String::from(
        "Button {\n    onclick: move |_| async move {\n        chart.save_svg(\"chart.svg\").await;\n    },\n    \"Save as SVG\"\n}\nButton {\n    onclick: move |_| async move {\n        chart.save_png(\"chart.png\").await;\n    },\n    \"Save as PNG\"\n}",
    );
    if zoom {
        code += "\nButton {\n    disabled: !chart.is_zoomed(),\n    onclick: move |_| chart.reset_zoom(),\n    \"Reset zoom\"\n}";
    }
    code
}

/// The demo's buttons under `drawn`, each calling the chart's `handle`.
fn with_buttons(values: &DemoValues, chart: ChartHandle, drawn: Element) -> Element {
    if values.str("buttons") != "true" {
        return drawn;
    }
    let zoom = zoomable(values);
    rsx! {
        div { width: "100%",
            {drawn}
            Flex { direction: "row", gap: "sm", wrap: true,
                Button {
                    onclick: move |_| async move {
                        chart.save_svg("chart.svg").await;
                    },
                    "Save as SVG"
                }
                Button {
                    onclick: move |_| async move {
                        chart.save_png("chart.png").await;
                    },
                    "Save as PNG"
                }
                if zoom {
                    Button {
                        disabled: !chart.is_zoomed(),
                        focusable_when_disabled: true,
                        onclick: move |_| chart.reset_zoom(),
                        "Reset zoom"
                    }
                }
            }
        }
    }
}

/// The type's component in place of the generated `Chart`, and the table under it when shown.
fn component(values: &DemoValues, source: &str) -> String {
    let name = match values.str("type").as_str() {
        "bar" => "BarChart",
        "pie" => "PieChart",
        "funnel" => "FunnelChart",
        "scatter" => "ScatterChart",
        "heatmap" => "HeatmapChart",
        "candlestick" => "CandlestickChart",
        "radar" => "RadarChart",
        _ => "LineChart",
    };
    let mut chart = source.replacen("Chart", name, 1);
    if values.str("buttons") == "true" {
        chart = format!("{chart}\n{}", buttons_code(zoomable(values)));
    }
    // `onrange` sets this signal.
    if zoomable(values) {
        chart = format!("let mut selected = use_signal(|| None::<ChartRange>);\n\n{chart}");
    }
    if values.str("buttons") == "true" {
        chart = format!("let chart = use_chart();\n\n{chart}");
    }
    match values.str("table") == "true" {
        true => format!("{chart}\n{}", table_code(values)),
        false => chart,
    }
}

/// The chart's data as a sortable `Table`, the first column naming each row.
fn data_table(values: &DemoValues) -> Element {
    let (caption, first, names, rows) = table(values);
    let mut columns = vec![
        column(first)
            .value(|row: &Row| row.0.to_string())
            .row_header(),
    ];
    columns.extend(
        names
            .into_iter()
            .enumerate()
            .map(|(i, name)| column(name).value(move |row: &Row| row.1[i]).sortable()),
    );
    rsx! {
        Table { caption, data: rows, columns }
    }
}

#[component]
pub fn ChartPage() -> Element {
    let theme = use_theme();
    let legend = ["bottom", "top", "start", "end", "none"];
    let mut selected = use_signal(|| None::<ChartRange>);
    let chart = use_chart();
    rsx! {
        DocPage {
            title: "Chart",
            source: "libero/src/components/data_display/chart",
            markdown: "/md/chart.md",
            properties: vec![
                props("LineChart", vec![
                    prop("x", "XValues").default("required").doc("The x axis, one value per point: labels, one evenly spaced slot each, or numbers, each point at its value, the axis ticking at round values written with `value_format`. `XValues::from_rows(&rows, |row| ..)` reads a list of records."),
                    prop("series", "Vec<Series>").default("required").doc("The lines, each with one value per x value. `None`, `NaN` and infinity are gaps. `.color(..)` (a theme color or hex) sets a colour, else the palette by place; `.unit(..)` gives its own unit in the tooltip and summary, while a unit every series shares labels the y axis; `.symbol(PointSymbol::Square)` draws points as that shape, a cue beside colour; `.errors([(low, high), ..])` draws a see-through band, which a stacked chart reads in the tooltip alone; `.mark_line(MarkLine::average())` draws a dashed line labelled on the chart, also `min()`, `max()`, `value(150.0)` and `.label(..)`; `.mark_point(MarkPoint::Max)` pins the highest value, `Min` the lowest; `.secondary_axis()` measures a line on its own scale at the right."),
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
                    prop("legend", "LegendPosition").default("bottom with two series or more").doc("`bottom`, `top`, `start`, `end` or `none`. On a chart under 300 px wide, `start` goes above the plot and `end` below it."),
                    prop("grid", "Option<bool>").default(on(theme.chart.grid)).doc("Horizontal guide lines."),
                    prop("value_format", "ValueFormat").default("Auto").doc("How values are written on the axis, in the tooltip and in the summary. Thousands group as the `Formats` say, as `12,500` or `12.500`."),
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
                    prop("zoom", "Option<bool>").default(on(theme.chart.zoom)).doc("Zoom and pan the x axis: Ctrl or Cmd with the wheel, a pinch, a drag (Shift or Alt with it once `onrange` is set), the keys. Reset it from code with the `handle`."),
                    prop("onrange", "Option<EventHandler<Option<ChartRange>>>").doc("Set, a drag or Shift with an arrow brushes a band of x values; Shift or Alt with a drag pans instead. `None` when Escape clears it."),
                    prop("handle", "Option<ChartHandle>").doc("From `use_chart()`: reads the drawing as SVG or PNG, saves it (through `save_file`: a download on the web, the share sheet on Android), resets the zoom and says whether it is zoomed. `is_zoomed()` re-runs the component that reads it."),
                ]),
                props("BarChart", vec![
                    prop("x", "XValues").default("required").doc("The categories, one per bar or group of bars."),
                    prop("series", "Vec<Series>").default("required").doc("The bars' series, each with one value per x value. `None` leaves no bar. `.colors([..])` colours bars one by one, `.negative_color(..)` those below 0, and `.errors([(low, high), ..])` draws error bars, written after the value in the tooltip. A stack rounds only its topmost positive and lowest negative segment. Upright, a `.kind(SeriesKind::Line)` one is drawn as a line over the bars."),
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
                    prop("y2_title", "Option<String>").doc("Beside the right y axis, read downward: what its `Series::secondary_axis()` series measure. An upright chart only."),
                    prop("value_label_position", "BarLabelPosition").default("chart-wide: inside on a `horizontal` chart or once any shown series has error ranges, past the end otherwise").doc("`inside` the bar, in the ink that reads on it, or `outside` past its end. A stack always writes inside."),
                    prop("bar_width", "Option<f32>").default("5 px each side of a band").doc("The share of a category's band its bars fill, clamped to `0.1..=1`."),
                    prop("bar_gap", "Option<f32>").default(theme.chart.bar_gap.to_string()).doc("Between two bars of a group, in pixels."),
                    prop("legend", "LegendPosition").default("bottom with two series or more").doc("`bottom`, `top`, `start`, `end` or `none`. On a chart under 300 px wide, `start` goes above the plot and `end` below it."),
                    prop("grid", "Option<bool>").default(on(theme.chart.grid)).doc("Guide lines across the values."),
                    prop("value_format", "ValueFormat").default("Auto").doc("How values are written on the axis, the bars, in the tooltip and in the summary. On `percent` the axis and the bars write shares in percent. Thousands group as the `Formats` say, as `12,500` or `12.500`."),
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
                    prop("zoom", "Option<bool>").default(on(theme.chart.zoom)).doc("Zoom and pan along the categories: Ctrl or Cmd with the wheel, a pinch, a drag (Shift or Alt with it once `onrange` is set), the keys. Reset it from code with the `handle`."),
                    prop("onrange", "Option<EventHandler<Option<ChartRange>>>").doc("Set, a drag or Shift with an arrow brushes a band of categories; Shift or Alt with a drag pans instead. `None` when Escape clears it."),
                    prop("handle", "Option<ChartHandle>").doc("From `use_chart()`: reads the drawing as SVG or PNG, saves it (through `save_file`: a download on the web, the share sheet on Android), resets the zoom and says whether it is zoomed. `is_zoomed()` re-runs the component that reads it."),
                ]),
                props("FunnelChart", vec![
                    prop("stages", "Vec<Slice>").default("required").doc("The stages, top to bottom, each as wide as its value over the largest. Drawn widest first: a stage larger than the one before it moves up, with a warning in development. A negative value has no width."),
                    prop("title", "Option<String>").doc("Shown above the chart, and its accessible name."),
                    prop("description", "Option<String>").doc("A sentence under the title: what to read from the chart."),
                    prop("labels", "PieLabels").default(theme.chart.pie_labels.as_str()).doc("On each stage: `percent`, its share of the first stage, `value`, `name` or `none`, in the ink that reads there. A label wider than its stage is left out; the tooltip and the summary keep it."),
                    prop("legend", "LegendPosition").default("bottom with two stages or more").doc("`bottom`, `top`, `start`, `end` or `none`. On a chart under 300 px wide, `start` goes above the plot and `end` below it."),
                    prop("value_format", "ValueFormat").default("Auto").doc("How values are written on the stages, in the tooltip and in the summary. Thousands group as the `Formats` say, as `12,500` or `12.500`."),
                    prop("unit", "Option<String>").doc("After every value, as `\"kg\"`."),
                    prop("grid", "Option<bool>").doc("Shared with the other charts; a funnel has no grid to draw."),
                    prop("aspect_ratio", "Option<f32>").default("4 / 3").doc("Width over height of the plot. Unset, the theme's `pie_aspect_ratio`."),
                    prop("loading", "bool").default("false").doc("A placeholder instead of the plot, and `aria-busy` on the chart."),
                    prop("empty", "Option<Element>").doc("Shown in the plot when no stage has a value. Unset, \"No data\"."),
                    prop("tooltip", "Option<bool>").default(on(theme.chart.tooltip)).doc("A tooltip with the stage's value at the pointer's or the keys' stage."),
                    prop("tooltip_content", "Option<Callback<ChartPoint, Element>>").doc("The tooltip's content instead of the stage's name and value."),
                    prop("onpointclick", "Option<EventHandler<ChartPoint>>").doc("A click on a stage, a second tap on a pinned one, or Enter on the keys' stage. `index` is the stage's place in `stages`, `x` and `series_name` its name."),
                    prop("onactivechange", "Option<EventHandler<Option<ChartPoint>>>").doc("The stage under the pointer or the keys; `None` when it leaves."),
                    prop("hidden", "Option<Vec<String>>").doc("The names of the stages the legend hides; set, they are controlled."),
                    prop("default_hidden", "Vec<String>").default("[]").doc("Seeds the hidden stages once. Ignored when `hidden` is set."),
                    prop("onhiddenchange", "Option<EventHandler<Vec<String>>>").doc("The hidden stages a legend button asks for. The last shown stage stays."),
                    prop("handle", "Option<ChartHandle>").doc("From `use_chart()`: reads the drawing as SVG or PNG, saves it (through `save_file`: a download on the web, the share sheet on Android)."),
                ]),
                props("HeatmapChart", vec![
                    prop("x", "Vec<String>").default("required").doc("The columns' names, left to right."),
                    prop("y", "Vec<String>").default("required").doc("The rows' names, top to bottom."),
                    prop("values", "Vec<Vec<f64>>").default("required").doc("A row per `y`, a value per `x`. NaN, or a value missing at a row's end, is an empty cell, outlined; a row of the wrong length warns in development."),
                    prop("colors", "Vec<ChartColor>").default("light to strong primary").doc("The scale from the lowest value's colour to the highest's, two or more, as `\"primary.2\"` or a hex."),
                    prop("min", "Option<f64>").doc("The scale's low end. Unset, the lowest value."),
                    prop("max", "Option<f64>").doc("The scale's high end. Unset, the highest value."),
                    prop("steps", "Option<usize>").doc("Sorts the values into this many classes of the same width, one colour each, instead of a gradient."),
                    prop("cell_labels", "Option<bool>").default("true").doc("Each value written in its cell, in black or white by the cell. A value wider or taller than its cell is left out; the tooltip and the summary still have it."),
                    prop("title", "Option<String>").doc("Shown above the chart, and its accessible name."),
                    prop("description", "Option<String>").doc("A sentence under the title: what to read from the chart."),
                    prop("legend", "LegendPosition").default("bottom").doc("The colour scale, under the plot; `none` leaves it out."),
                    prop("value_format", "ValueFormat").default("Auto").doc("How values are written in the cells, on the scale, in the tooltip and in the summary. Thousands group as the `Formats` say, as `12,500` or `12.500`."),
                    prop("unit", "Option<String>").doc("After every value, as `\"°C\"`."),
                    prop("grid", "Option<bool>").doc("Shared with the other charts; a heatmap has no grid to draw."),
                    prop("hidden", "Option<Vec<String>>").doc("Shared with the other charts; a heatmap hides no rows."),
                    prop("default_hidden", "Vec<String>").default("[]").doc("Shared with the other charts; a heatmap hides no rows."),
                    prop("onhiddenchange", "Option<EventHandler<Vec<String>>>").doc("Shared with the other charts; never called by a heatmap."),
                    prop("aspect_ratio", "Option<f32>").default("4 / 3").doc("Width over height of the plot and its scale. Unset, the theme's `pie_aspect_ratio`, so rows keep room for their names."),
                    prop("loading", "bool").default("false").doc("A placeholder instead of the plot, and `aria-busy` on the chart."),
                    prop("empty", "Option<Element>").doc("Shown in the plot when no cell has a value. Unset, \"No data\"."),
                    prop("tooltip", "Option<bool>").default(on(theme.chart.tooltip)).doc("A tooltip with the cell's column, row and value at the pointer's or the keys' cell."),
                    prop("tooltip_content", "Option<Callback<ChartPoint, Element>>").doc("The tooltip's content instead of the cell's column, row and value."),
                    prop("onpointclick", "Option<EventHandler<ChartPoint>>").doc("A click on a cell, a second tap on a pinned one, or Enter on the keys' cell. `series` and `series_name` are its row, `index` and `x` its column."),
                    prop("onactivechange", "Option<EventHandler<Option<ChartPoint>>>").doc("The cell under the pointer or the keys; `None` when it leaves."),
                    prop("handle", "Option<ChartHandle>").doc("From `use_chart()`: reads the drawing as SVG or PNG, saves it (through `save_file`: a download on the web, the share sheet on Android)."),
                ]),
                props("CandlestickChart", vec![
                    prop("x", "Vec<String>").default("required").doc("The periods' names, left to right, evenly spaced."),
                    prop("candles", "Vec<Candle>").default("required").doc("A candle per `x`, `Candle::new(open, high, low, close)` or from a tuple. A price that is not a number, or a candle missing at the end, leaves a gap; a high under the body or a low over it warns in development."),
                    prop("name", "Option<String>").default("\"Price\"").doc("The prices' name in the tooltip and the summary."),
                    prop("style", "CandleStyle").default("candle").doc("`candle`: a body from the open to the close on a wick, hollow when it rose, filled when it fell. `ohlc`: a bar from the low to the high with the open ticked left and the close right."),
                    prop("up_color", "Option<ChartColor>").default("success").doc("A rising candle's colour, as `\"success.6\"` or a hex."),
                    prop("down_color", "Option<ChartColor>").default("error").doc("A falling candle's colour."),
                    prop("title", "Option<String>").doc("Shown above the chart, and its accessible name."),
                    prop("description", "Option<String>").doc("A sentence under the title: what to read from the chart."),
                    prop("y_min", "Option<f64>").doc("The price axis' lowest value. Unset, a round number at or under the lowest low."),
                    prop("y_max", "Option<f64>").doc("The price axis' highest value. Unset, a round number at or over the highest high."),
                    prop("y_ticks", "Option<usize>").default(theme.chart.y_ticks.to_string()).doc("The steps the price axis is cut into."),
                    prop("x_title", "Option<String>").doc("Under the periods: what they are."),
                    prop("y_title", "Option<String>").doc("Beside the price axis, read upward: what it measures."),
                    prop("zoom", "Option<bool>").default(on(theme.chart.zoom)).doc("Zoom and pan the periods: Ctrl or Cmd with the wheel, a pinch, a drag (Shift or Alt with it once `onrange` is set), the keys. Reset it from code with the `handle`."),
                    prop("onrange", "Option<EventHandler<Option<ChartRange>>>").doc("Set, a drag or Shift with an arrow brushes a band of periods; Shift or Alt with a drag pans instead. `None` when Escape clears it."),
                    prop("legend", "LegendPosition").default("bottom").doc("The Up and Down key, under the plot; `none` leaves it out."),
                    prop("grid", "Option<bool>").default(on(theme.chart.grid)).doc("Horizontal guide lines."),
                    prop("value_format", "ValueFormat").default("Auto").doc("How prices are written on the axis, in the tooltip and in the summary. Thousands group as the `Formats` say, as `12,500` or `12.500`."),
                    prop("unit", "Option<String>").doc("After every price, as `\"€\"`."),
                    prop("hidden", "Option<Vec<String>>").doc("Shared with the other charts; a candlestick chart hides nothing."),
                    prop("default_hidden", "Vec<String>").default("[]").doc("Shared with the other charts; a candlestick chart hides nothing."),
                    prop("onhiddenchange", "Option<EventHandler<Vec<String>>>").doc("Shared with the other charts; never called by a candlestick chart."),
                    prop("aspect_ratio", "Option<f32>").default("16 / 9").doc("Width over height of the plot and its key."),
                    prop("loading", "bool").default("false").doc("A placeholder instead of the plot, and `aria-busy` on the chart."),
                    prop("empty", "Option<Element>").doc("Shown in the plot when no candle has its four prices. Unset, \"No data\"."),
                    prop("tooltip", "Option<bool>").default(on(theme.chart.tooltip)).doc("A tooltip with the period's open, high, low and close at the pointer's or the keys' candle."),
                    prop("tooltip_content", "Option<Callback<ChartPoint, Element>>").doc("The tooltip's content instead of the period and its prices."),
                    prop("onpointclick", "Option<EventHandler<ChartPoint>>").doc("A click on a candle, a second tap on a pinned one, or Enter on the keys' candle. `index` is its place in `candles`, `value` its close."),
                    prop("onactivechange", "Option<EventHandler<Option<ChartPoint>>>").doc("The candle under the pointer or the keys; `None` when it leaves."),
                    prop("handle", "Option<ChartHandle>").doc("From `use_chart()`: reads the drawing as SVG or PNG, saves it (through `save_file`: a download on the web, the share sheet on Android), resets the zoom and says whether it is zoomed. `is_zoomed()` re-runs the component that reads it."),
                ]),
                props("PieChart", vec![
                    prop("slices", "Vec<Slice>").default("required").doc("The slices, each a share of their sum. A negative value takes no share. `Slice::from((\"Chrome\", 62.0))` builds one from a pair and `.color(\"success\")` sets its colour, else the palette by place."),
                    prop("title", "Option<String>").doc("Shown above the chart, and its accessible name."),
                    prop("description", "Option<String>").doc("A sentence under the title: what to read from the chart."),
                    prop("donut", "bool").default("false").doc("A ring instead of a full pie, `center` in its middle."),
                    prop("labels", "PieLabels").default(theme.chart.pie_labels.as_str()).doc("Beside each slice: `percent`, `value`, `name` or `none`. One too long for the room ends in an ellipsis; one that would overlap a larger slice's is left out. The tooltip and the summary write each whole."),
                    prop("center", "Option<Element>").doc("In a donut's hole, as a total, kept off the ring; its text shrinks in a small hole, the rest is clipped."),
                    prop("start_angle", "f32").default("0").doc("Where the first slice starts, in degrees clockwise from 12 o'clock."),
                    prop("end_angle", "Option<f32>").default("a full turn").doc("Where the last slice ends, in the same degrees: `start_angle: -90.0, end_angle: 90.0` is a half pie, a gauge with `donut`. At or before the start, it counts a turn on: `90.0` to `-90.0` is the lower half."),
                    prop("rose", "bool").default("false").doc("A nightingale rose: every slice the same angle, its radius by its value."),
                    prop("label_position", "PieLabelPosition").default(theme.chart.pie_label_position.as_str()).doc("`outside` beside each slice, or `inside` on it in the ink that reads there. Inside, a label too wide or too tall for its slice is left out; the summary keeps it."),
                    prop("min_label_angle", "Option<f32>").default(theme.chart.min_label_angle.to_string()).doc("A slice narrower than this many degrees gets no label; the summary still names it."),
                    prop("slice_gap", "Option<f32>").default(theme.chart.slice_gap.to_string()).doc("Between two slices, in pixels, in the page's colour."),
                    prop("slice_radius", "Size").default(theme.chart.slice_radius.map_or("none", |size| size.as_str())).doc("Corner radius of a slice, on the theme's radius scale; a thin slice or a thin ring takes less. Unset, square."),
                    prop("legend", "LegendPosition").default("bottom with two slices or more").doc("`bottom`, `top`, `start`, `end` or `none`. On a chart under 300 px wide, `start` goes above the plot and `end` below it."),
                    prop("value_format", "ValueFormat").default("Auto").doc("How values are written beside the slices, in the tooltip and in the summary. Thousands group as the `Formats` say, as `12,500` or `12.500`."),
                    prop("unit", "Option<String>").doc("After every value, as `\"kg\"`."),
                    prop("grid", "Option<bool>").doc("Shared with the other charts; a pie has no grid to draw."),
                    prop("aspect_ratio", "Option<f32>").default("4 / 3").doc("Width over height of the plot. Unset, the theme's `pie_aspect_ratio`, squarer than the other charts' `aspect_ratio`."),
                    prop("loading", "bool").default("false").doc("A placeholder instead of the plot, and `aria-busy` on the chart."),
                    prop("empty", "Option<Element>").doc("Shown in the plot when the slices sum to nothing. Unset, \"No data\"."),
                    prop("tooltip", "Option<bool>").default(on(theme.chart.tooltip)).doc("A tooltip with the slice's value and share at the pointer's or the keys' slice."),
                    prop("tooltip_content", "Option<Callback<ChartPoint, Element>>").doc("The tooltip's content instead of the slice's name, value and share."),
                    prop("onpointclick", "Option<EventHandler<ChartPoint>>").doc("A click on a slice, a second tap on a pinned one, or Enter on the keys' slice. `index` is the slice's place in `slices`, `x` and `series_name` its name."),
                    prop("onactivechange", "Option<EventHandler<Option<ChartPoint>>>").doc("The slice under the pointer or the keys; `None` when it leaves."),
                    prop("hidden", "Option<Vec<String>>").doc("The names of the slices the legend hides; set, they are controlled."),
                    prop("default_hidden", "Vec<String>").default("[]").doc("Seeds the hidden slices once. Ignored when `hidden` is set."),
                    prop("onhiddenchange", "Option<EventHandler<Vec<String>>>").doc("The hidden slices a legend button asks for. The last shown slice stays."),
                    prop("handle", "Option<ChartHandle>").doc("From `use_chart()`: reads the drawing as SVG or PNG, saves it (through `save_file`: a download on the web, the share sheet on Android)."),
                ]),
                props("ScatterChart", vec![
                    prop("series", "Vec<ScatterSeries>").default("required").doc("The groups of points, each an x and a y, `ScatterSeries::new(\"Online\", [(12.0, 34.0)])`. A third value, a size, makes the series bubbles, their area by it. A point with an x or y that is not a number is left out. `ScatterSeries::from_rows(\"Sales\", &rows, |row| (row.price, row.sold))` builds one from your rows."),
                    prop("title", "Option<String>").doc("Shown above the chart, and its accessible name."),
                    prop("description", "Option<String>").doc("A sentence under the title: what to read from the chart."),
                    prop("trend", "Option<Trend>").doc("A curve fitted through each series' points, in its colour: `Linear`, `Exponential` over the points above y 0, `Logarithmic` over those right of x 0, or `Polynomial(n)`, its order held to 2 to 6."),
                    prop("x_min", "Option<f64>").doc("The x axis' lowest value. Unset, a round number at or under the lowest x."),
                    prop("x_max", "Option<f64>").doc("The x axis' highest value. Unset, a round number at or over the highest x."),
                    prop("y_min", "Option<f64>").doc("The y axis' lowest value. Unset, a round number at or under the lowest y."),
                    prop("y_max", "Option<f64>").doc("The y axis' highest value. Unset, a round number at or over the highest y."),
                    prop("y_ticks", "Option<usize>").default(theme.chart.y_ticks.to_string()).doc("About how many steps each axis is cut into."),
                    prop("x_title", "Option<String>").doc("Under the x axis: what it measures."),
                    prop("y_title", "Option<String>").doc("Beside the y axis, read upward: what it measures."),
                    prop("legend", "LegendPosition").default("bottom with two series or more").doc("`bottom`, `top`, `start`, `end` or `none`. Each key is its series' point shape."),
                    prop("grid", "Option<bool>").default(on(theme.chart.grid)).doc("Horizontal guide lines."),
                    prop("value_format", "ValueFormat").default("Auto").doc("How values are written on the axes, in the tooltip and in the summary. The axes tick at round values. Thousands group as the `Formats` say, as `12,500` or `12.500`."),
                    prop("unit", "Option<String>").doc("After every y value, as `\"kg\"`; a series' own `unit` wins."),
                    prop("aspect_ratio", "Option<f32>").default("16 / 9").doc("Width over height of the plot."),
                    prop("loading", "bool").default("false").doc("A placeholder instead of the plot, and `aria-busy` on the chart."),
                    prop("empty", "Option<Element>").doc("Shown in the plot when there is no point at all. Unset, \"No data\"."),
                    prop("tooltip", "Option<bool>").default(on(theme.chart.tooltip)).doc("A tooltip and a crosshair at the pointer's nearest or the keys' point."),
                    prop("tooltip_content", "Option<Callback<ChartPoint, Element>>").doc("The tooltip's content instead of the series and the point's x, y and size."),
                    prop("onpointclick", "Option<EventHandler<ChartPoint>>").doc("A click on a point, a second tap on a pinned one, or Enter on the keys' point. `index` is the point's place in its series."),
                    prop("onactivechange", "Option<EventHandler<Option<ChartPoint>>>").doc("The point under the pointer or the keys; `None` when it leaves."),
                    prop("hidden", "Option<Vec<String>>").doc("The names of the series the legend hides; set, they are controlled."),
                    prop("default_hidden", "Vec<String>").default("[]").doc("Seeds the hidden series once. Ignored when `hidden` is set."),
                    prop("onhiddenchange", "Option<EventHandler<Vec<String>>>").doc("The hidden series a legend button asks for. The last shown series stays."),
                    prop("handle", "Option<ChartHandle>").doc("From `use_chart()`: reads the drawing as SVG or PNG, saves it (through `save_file`: a download on the web, the share sheet on Android)."),
                ]),
                props("RadarChart", vec![
                    prop("axes", "Vec<RadarAxis>").default("required").doc("The spokes, clockwise from the top, at least three: `\"Speed\".into()` or `(\"Speed\", 10.0).into()`, the value at its rim. Unset, the rim is the axis' largest value rounded up to 1, 2, 2.5 or 5 times a power of ten. Fewer than three show the empty state."),
                    prop("series", "Vec<Series>").default("required").doc("One value per axis each, in the axes' order. A missing value is drawn at the centre; a series with more or fewer values than axes warns in development and is cut or padded."),
                    prop("title", "Option<String>").doc("Shown above the chart, and its accessible name."),
                    prop("description", "Option<String>").doc("A sentence under the title: what to read from the chart."),
                    prop("filled", "Option<bool>").default(on(theme.chart.radar_filled)).doc("Each series' shape filled, translucent, inside its outline."),
                    prop("rings", "Option<usize>").default(theme.chart.radar_rings.to_string()).doc("The web's rings, its steps from the centre to the rim."),
                    prop("legend", "LegendPosition").default("bottom with two series or more").doc("`bottom`, `top`, `start`, `end` or `none`. Each key is its series' point shape."),
                    prop("grid", "Option<bool>").default(on(theme.chart.grid)).doc("The web: its rings and spokes."),
                    prop("value_format", "ValueFormat").default("Auto").doc("How values are written in the tooltip and in the summary. Thousands group as the `Formats` say, as `12,500` or `12.500`."),
                    prop("unit", "Option<String>").doc("After every value, as `\"pts\"`; a series' own `unit` wins."),
                    prop("aspect_ratio", "Option<f32>").default("4 / 3").doc("Width over height of the plot. Unset, the theme's `pie_aspect_ratio`."),
                    prop("loading", "bool").default("false").doc("A placeholder instead of the plot, and `aria-busy` on the chart."),
                    prop("empty", "Option<Element>").doc("Shown in the plot when there is no value or fewer than three axes. Unset, \"No data\"."),
                    prop("tooltip", "Option<bool>").default(on(theme.chart.tooltip)).doc("A tooltip with each series' value on the axis of the pointer's nearest or the keys' point."),
                    prop("tooltip_content", "Option<Callback<ChartPoint, Element>>").doc("The tooltip's content instead of the axis and each series' value there."),
                    prop("onpointclick", "Option<EventHandler<ChartPoint>>").doc("A click on a point, a second tap on a pinned one, or Enter on the keys' point. `index` is the axis' place in `axes`, `x` its name."),
                    prop("onactivechange", "Option<EventHandler<Option<ChartPoint>>>").doc("The point under the pointer or the keys; `None` when it leaves."),
                    prop("hidden", "Option<Vec<String>>").doc("The names of the series the legend hides; set, they are controlled."),
                    prop("default_hidden", "Vec<String>").default("[]").doc("Seeds the hidden series once. Ignored when `hidden` is set."),
                    prop("onhiddenchange", "Option<EventHandler<Vec<String>>>").doc("The hidden series a legend button asks for. The last shown series stays."),
                    prop("handle", "Option<ChartHandle>").doc("From `use_chart()`: reads the drawing as SVG or PNG, saves it (through `save_file`: a download on the web, the share sheet on Android)."),
                ]),
            ],
            accessibility: a11y()
                .key(["Tab"], "Into the plot and the legend's buttons in the order drawn: a legend at the top or the start comes first.")
                .key(["ArrowRight", "ArrowLeft"], "Line: the next or previous point of the line, as drawn, left to right in either text direction. Bar: the next or previous category's bar of the series; `horizontal`, the next or previous series' bar in the category. Pie: the next or previous slice, clockwise. Funnel: the next or previous stage, top down. Scatter: the series' next or previous point, left to right. Heatmap: the next or previous cell in the row, left to right. Candlestick: the next or previous candle, left to right. Radar: the series' point on the next or previous axis, clockwise.")
                .key(["ArrowDown", "ArrowUp"], "Line: the next or previous line at the same x value, or its nearest point. Bar: the next or previous series' bar in the category; `horizontal`, the next or previous category, top to bottom. Pie and funnel: the next or previous slice or stage. Scatter: the next or previous series' point nearest in x. Heatmap: the row below or above, at the nearest column. Radar: the next or previous series' point on the same axis.")
                .key(["Home", "End"], "The line's or the series' first or last point or bar; a pie's first or last slice, a funnel's top or bottom stage; a scatter series' leftmost or rightmost point; a heatmap row's first or last cell; the first or last candle; a radar series' point on the first or last axis.")
                .key(["Enter", "Space"], "Calls `onpointclick` with the point, bar or slice.")
                .key(["Escape"], "Clears the brushed band, else the point and its tooltip. A tooltip a pointer hovers closes with Escape wherever focus is, and stays closed until the pointer leaves the plot.")
                .key(["+", "-"], "With `zoom`: zooms the x axis, a lying bar chart's categories, in or out around the keys' point; the arrows carry the window along at its edge.")
                .key(["0"], "With `zoom`: shows every x value or category again.")
                .key(["Shift+ArrowRight", "Shift+ArrowLeft"], "With `onrange`: brushes from the keys' point to the next or previous x value.")
                .handles([
                    "The chart is a `role=\"figure\"` named by `title`, else by a spread `aria-label`, else \"chart\". The drawing is hidden from screen readers.",
                    "A loading chart says \"Loading chart\" in a live region when it starts, and its summary says it too.",
                    "A hidden summary gives the x range and each series' lowest and highest value, and where the highest is, then each mark line's name and value and each marked point's place and value, each line's error band and each bar series' error bars with their outer ends, the series drawn as a line over bars, the series on the right y axis, the axes' titles and, stacked, that the top line is the total; a pie's, the total, then each slice's value and share; a funnel's, its first and last stage, then each stage's value and share of the first; a scatter's, each series' count of points and its x and y ranges, a bubble series' sizes and each trend line's kind and direction, as \"Online: linear trend, rising.\"; a heatmap's, its rows by columns and its lowest and highest value, the highest and the lowest cell, then each row's cells (above 12 rows or 150 cells, only the empty cells the keys skip, then a pointer to the keys); a candlestick chart's, its periods, the first open and the last close with the change, the highest high and the lowest low with their periods, and how many candles rose, fell and stayed; a radar's, its axes, then each series' highest and lowest axis, by share of the axis maximum as drawn, and the axes it has no value on.",
                    "The plot is one tab stop, a `role=\"application\"` described as a chart, so a screen reader hands it the arrows. Each point the keys reach is announced, as \"This week, Tue: 132\", \"Chrome: 62, 62%.\" \"Online: x 12, y 34, size 80\" or, for a heatmap's cell, \"Tue, 12: 46\"; a candle reads its four prices and its direction, as \"Price, May: open 33, high 44, low 32, close 43, up\".",
                    "A heatmap never tells by colour alone: each cell writes its value where it fits, in black or white by the cell; an empty cell is outlined, dashed, and the scale under the plot keys it as \"No value\".",
                    "A candle never tells its direction by colour alone: a rising one is hollow, a falling one filled, and the key under the plot says which is which; an OHLC bar ticks its open left and its close right.",
                    "The legend's entries are toggle buttons: pressed while their series or slice shows, struck through when hidden. The last shown one's button is `aria-disabled`. Should `hidden` or `default_hidden` name every one, the plot and the summary say \"All series are hidden.\" and the legend stays. A dashed line's key is a dash, as drawn; once a line sets a `symbol`, each key is its line's point shape. A line among bars is keyed by a short stroke, the bars by squares.",
                    "On a touch screen a tap pins a point's tooltip, a second tap on it clicks; a vertical swipe still scrolls the page.",
                    "With `zoom`, the shown range is announced once a zoom or pan settles, as \"Showing Tue to Fri\". Ctrl or Cmd with the wheel zooms only a zoomable plot, so the page zoom works everywhere else.",
                    "A drag with `onrange` set brushes; Shift or Alt with it pans. The band is outlined as well as tinted, and announced as \"Selected Tue to Thu\".",
                    "A saved SVG is self-contained: its colours resolved, its font named, the chart's `title` as its `<title>`.",
                    "The chart draws no buttons: the ones you render from the `use_chart()` handle are yours to name, to announce what they saved and to keep focusable while there is nothing to reset.",
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
                    "A hidden slice leaves the drawing, so the shown ones' shares grow in the labels, the tooltip and the announcement; the summary keeps every slice and its share of all, and says so while one is hidden.",
                    "The summary describes the data, not the drawing: a series the legend hides stays in it, with its mark lines and error band.",
                    "The tooltip follows the point and takes no pointer, so it is not hoverable (WCAG 1.4.13); Escape closes it and the announcement gives the same values.",
                    "`grid` draws nothing on a pie.",
                    "`.colors` and `.negative_color` add no meaning of their own: a loss already hangs below the axis. The legend keeps the series' colour.",
                    "A percent stack draws no error bars and reads the range in its tooltip; on any other layout the value axis reaches the far end of a range unless `y_min` or `y_max` is set. A line draws its ranges as a band, only around its points and not when `stacked`.",
                    "A rose's radius and a slice under `min_label_angle` are read by eye or not at all; the summary gives every value.",
                    "`zoom` and `onrange` are for lines and bars; a pie, a funnel, a scatter, a heatmap, a candlestick chart and a radar export only.",
                    "A radar's web writes no values on its rings; each axis may have its own rim, so the tooltip, the announcement and the summary give the values. A value past its axis' `max` is drawn at the rim.",
                    "A bubble's size and a trend line are read by eye; the tooltip, the announcement and the summary give the sizes and each trend's direction. Bubbles are drawn largest first, so a small one stays on top.",
                    "A heatmap's scale is drawn under the plot whatever `legend` says, but `none`; in forced colours its cells keep the scale's colours and their black or white values; the axes, the names and an empty cell's outline take the system's.",
                    "A funnel is drawn widest stage first; the keys, the summary and the legend keep the order of `stages`. A hidden stage leaves the drawing and the others widen to the largest shown; percent labels stay shares of the first stage.",
                    "An export keeps the normal palette in forced colours. `png()` is `None` on Blitz and on the server, and `save_png` returns `SaveOutcome::Failed`.",
                    "A mark line is drawn on a line chart that is not `stacked`, on upright, `grouped` bars and on a line among upright bars, and only inside the y axis' ends; the summary leaves out the ones not drawn, except past the ends. A marked point is drawn on a line only, in a line chart or among upright bars.",
                    "A `SeriesKind::Line` series is drawn as a line on an upright `BarChart` only; a `horizontal` one draws it as a bar and warns in development. A line among bars never stacks, and on a `percent` chart it is read on the right y axis.",
                    "A right y axis is drawn on a line chart and on an upright bar chart, though not for a `percent` chart's bars, which share one scale; `y_min` and `y_max` set the left one, a `stacked` line chart stacks only the left one's lines, and stacked bars stack apart on each axis. Give its series a `symbol` or `dashed()` and a `y2_title`, so the reader can tell which axis a series is read on.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "LineChart" }
                    ", "
                    Code { source: "BarChart" }
                    ", "
                    Code { source: "PieChart" }
                    ", "
                    Code { source: "FunnelChart" }
                    ", "
                    Code { source: "ScatterChart" }
                    ", "
                    Code { source: "HeatmapChart" }
                    ", "
                    Code { source: "CandlestickChart" }
                    " and "
                    Code { source: "RadarChart" }
                    ", each drawn as an SVG that fills its width, with a tooltip, the arrow keys, a legend that hides and shows, and a summary for screen readers. Lines, bars and candlesticks zoom and pan with "
                    Code { source: "zoom" }
                    "; every chart exports as SVG or PNG through a "
                    Code { source: "use_chart()" }
                    " handle. Needs the "
                    Code { source: "charts" }
                    " feature. Web and Android."
                }
            },
            Demo {
                component: "Chart",
                children_text: "",
                wrap: Wrap(component),
                controls: vec![
                    // A select: eight types overflow a segmented toggle.
                    Control::select(
                        "type",
                        ["line", "bar", "pie", "funnel", "scatter", "heatmap", "candlestick", "radar"],
                    )
                        .default("line")
                        .labels(["Line", "Bar", "Pie", "Funnel", "Scatter", "Heatmap", "Candlestick", "Radar"])
                        .code(|_, values| {
                            let mark = marks_code(values);
                            match values.str("type").as_str() {
                            "bar" => vec![
                                "title: \"Profit\"".to_string(),
                                format!("x: {QUARTERS:?}"),
                                format!(
                                    "series: vec![Series::new(\"2025\", {PROFIT_2025:?}){}{mark}, Series::new(\"2026\", {PROFIT_2026:?}){}]",
                                    profit_2025(values).0,
                                    margin(values).map(|m| m.0).unwrap_or_default()
                                ),
                            ],
                            "pie" => vec![
                                "title: \"Browsers\"".to_string(),
                                "slices: vec![Slice::new(\"Chrome\", 62.0), Slice::new(\"Safari\", 20.0), Slice::new(\"Firefox\", 10.0), Slice::new(\"Other\", 8.0)]".to_string(),
                            ],
                            "funnel" => vec![
                                "title: \"Checkout\"".to_string(),
                                "stages: vec![Slice::new(\"Visits\", 1200.0), Slice::new(\"Product views\", 860.0), Slice::new(\"Carts\", 480.0), Slice::new(\"Checkouts\", 300.0), Slice::new(\"Orders\", 210.0)]".to_string(),
                            ],
                            "scatter" => vec![
                                "title: \"Ad spend and sales\"".to_string(),
                                scatter_series(values).0,
                            ],
                            "heatmap" => vec![
                                "title: \"Visits by hour\"".to_string(),
                                format!("x: {HOURS:?}.map(String::from).to_vec()"),
                                format!("y: {WEEKDAYS:?}.map(String::from).to_vec()"),
                                format!("values: {VISITS:?}.map(|row| row.to_vec()).to_vec()"),
                            ],
                            "candlestick" => vec![
                                "title: \"Share price\"".to_string(),
                                format!("x: {MONTHS:?}.map(String::from).to_vec()"),
                                format!(
                                    "candles: vec![{}]",
                                    PRICES.map(|(o, h, l, c)| format!("Candle::new({o:?}, {h:?}, {l:?}, {c:?})")).join(", ")
                                ),
                            ],
                            "radar" => vec![
                                "title: \"Phones\"".to_string(),
                                format!(
                                    "axes: vec![{}]",
                                    FEATURES.map(|name| format!("({name:?}, 10.0).into()")).join(", ")
                                ),
                                format!("series: vec![Series::new(\"Phone A\", {PHONE_A:?}), Series::new(\"Phone B\", {PHONE_B:?})]"),
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
                    // The 2025 bars: one colour, one per quarter, or its loss in red.
                    Control::toggle("colors", ["series", "per_bar", "negative"])
                        .labels(["Series", "Per bar", "Losses"])
                        .hidden_when(is("bar"))
                        .code(|_, _| vec![]),
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
                    Control::select("steps", ["gradient", "3", "5"])
                        .default("gradient")
                        .hidden_when(is("heatmap"))
                        .code(|_, values| match values.str("steps").as_str() {
                            "gradient" => vec![],
                            steps => vec![format!("steps: {steps}")],
                        }),
                    Control::switch("cell_labels")
                        .default("true")
                        .hidden_when(is("heatmap"))
                        .code(|_, values| match values.str("cell_labels").as_str() {
                            "true" => vec![],
                            _ => vec!["cell_labels: false".to_string()],
                        }),
                    Control::toggle("style", ["candle", "ohlc"])
                        .default("candle")
                        .labels(["Candles", "OHLC bars"])
                        .hidden_when(is("candlestick")),
                    Control::select("labels", ["percent", "value", "name", "none"])
                        .default(theme.chart.pie_labels.as_str())
                        .hidden_when(is("shares")),
                    Control::toggle("label_position", ["outside", "inside"])
                        .default(theme.chart.pie_label_position.as_str())
                        .hidden_when(is("pie")),
                    Control::slider("min_label_angle", ["0", "15", "30", "45"])
                        .labels(["0°", "15°", "30°", "45°"])
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
                        .labels(["0 px", "1 px", "2 px", "4 px", "8 px"])
                        .default("1")
                        .hidden_when(is("pie"))
                        .code(|_, values| match values.str("slice_gap").as_str() {
                            "1" => vec![],
                            gap => vec![format!("slice_gap: {gap}.0")],
                        }),
                    Control::select("slice_radius", ["auto", "xs", "sm", "md", "lg"])
                        .default("auto")
                        .hidden_when(is("pie"))
                        .code(|_, values| match values.str("slice_radius").as_str() {
                            "auto" => vec![],
                            size => vec![format!("slice_radius: \"{size}\"")],
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
                    Control::select("trend", ["linear", "exponential", "logarithmic", "polynomial", "none"])
                        .default("linear")
                        .hidden_when(is("scatter"))
                        .code(|_, values| match trend(values) {
                            Some((code, _)) => vec![format!("trend: {code}")],
                            None => vec![],
                        }),
                    // A third value per point, the orders, sizes each bubble.
                    Control::switch("bubbles")
                        .default("true")
                        .hidden_when(is("scatter"))
                        .code(|_, _| vec![]),
                    Control::switch("filled")
                        .default(on(theme.chart.radar_filled))
                        .hidden_when(is("radar")),
                    Control::slider("rings", ["2", "3", "4", "5", "6", "8", "10"])
                        .default(theme.chart.radar_rings.to_string())
                        .hidden_when(is("radar"))
                        .code(|control, values| match values.str("rings") {
                            rings if rings == control.default => vec![],
                            rings => vec![format!("rings: {rings}")],
                        }),
                    Control::toggle("mark_point", ["max", "min", "both", "none"])
                        .default("max")
                        .hidden_when(is("line"))
                        .code(|_, _| vec![]),
                    Control::toggle("mark_line", ["average", "target", "none"])
                        .default("average")
                        .hidden_when(is("series"))
                        .code(|_, _| vec![]),
                    Control::switch("error_band")
                        .default("true")
                        .hidden_when(is("line"))
                        .code(|_, _| vec![]),
                    // Drawn on every layout but percent, which reads them in the tooltip.
                    Control::switch("error_bars")
                        .hidden_when(is("bar"))
                        .code(|_, _| vec![]),
                    Control::switch("secondary_axis")
                        .default("true")
                        .hidden_when(is("line"))
                        .code(|_, _| vec![]),
                    // A third series drawn as a line over the bars, on either y axis.
                    Control::toggle("line_series", ["none", "left", "right"])
                        .default("none")
                        .labels(["None", "Left axis", "Right axis"])
                        .hidden_when(is("bar"))
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
                                if let Some(title) = y2_title(values) {
                                    lines.push(format!("y2_title: {title:?}"));
                                }
                                lines
                            }
                            None => vec![],
                        }),
                    Control::select("legend", legend),
                    Control::switch("tooltip").default(on(theme.chart.tooltip)),
                    Control::switch("zoom").default("true").hidden_when(is("zoom")).code(|_, values| {
                        match (values.str("zoom").as_str(), values.str("type").as_str()) {
                            ("true", "line" | "bar" | "candlestick") => vec![
                                "zoom: true".to_string(),
                                "onrange: move |range| selected.set(range)".to_string(),
                            ],
                            _ => vec![],
                        }
                    }),
                    // Save and Reset zoom buttons the demo draws from the handle, the way a user would.
                    Control::switch("buttons").default("true").code(|_, values| {
                        match values.str("buttons").as_str() {
                            "true" => vec!["handle: chart".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("loading"),
                    // The same data in a `Table` under the chart, for a screen reader and sorting.
                    Control::switch("table").code(|_, _| vec![]),
                ],
                render: move |values: DemoValues| {
                    let legend = LegendPosition::from(values.str("legend").as_str());
                    let tooltip = values.str("tooltip") == "true";
                    let loading = values.str("loading") == "true";
                    let range = y_range(&values);
                    let (y_min, y_max) = (range.map(|r| r.0), range.map(|r| r.1));
                    let y_ticks = values.str("y_ticks").parse::<usize>().ok();
                    let titles = axis_titles(&values);
                    let y2_title = titles.and(y2_title(&values)).map(str::to_string);
                    let (x_title, y_title) = (
                        titles.map(|t| t.0.to_string()),
                        titles.map(|t| t.1.to_string()),
                    );
                    let zoom = values.str("zoom") == "true";
                    let handle = (values.str("buttons") == "true").then_some(chart);
                    let onrange = zoom.then_some(EventHandler::new(move |range: Option<ChartRange>| selected.set(range)));
                    let drawn = match values.str("type").as_str() {
                        "bar" => brushed(zoom, selected, rsx! {
                            BarChart {
                                title: "Profit",
                                x: QUARTERS,
                                series: vec![marked(&values, profit_2025(&values).1), Series::new("2026", PROFIT_2026)]
                                    .into_iter().chain(margin(&values).map(|m| m.1)).collect::<Vec<_>>(),
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
                                y_min, y_max, y_ticks, x_title, y_title, y2_title: y2_title.clone(),
                                legend, tooltip, loading, zoom, onrange, handle,
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
                                    slice_radius: or_unset::<Size>(values.str("slice_radius")),
                                    center: rsx! { "1,204 users" },
                                    legend, tooltip, loading, handle,
                                }
                            }
                        }
                        "funnel" => rsx! {
                            FunnelChart {
                                title: "Checkout",
                                stages: stages(),
                                labels: PieLabels::from(values.str("labels").as_str()),
                                legend, tooltip, loading, handle,
                            }
                        },
                        "scatter" => rsx! {
                            ScatterChart {
                                title: "Ad spend and sales",
                                series: scatter_series(&values).1,
                                trend: trend(&values).map(|t| t.1),
                                y_min, y_max, y_ticks, x_title, y_title,
                                legend, tooltip, loading, handle,
                            }
                        },
                        "heatmap" => rsx! {
                            HeatmapChart {
                                title: "Visits by hour",
                                x: names(&HOURS),
                                y: names(&WEEKDAYS),
                                values: VISITS.map(|row| row.to_vec()).to_vec(),
                                steps: values.str("steps").parse::<usize>().ok(),
                                cell_labels: values.str("cell_labels") == "true",
                                legend, tooltip, loading, handle,
                            }
                        },
                        "candlestick" => brushed(zoom, selected, rsx! {
                            CandlestickChart {
                                title: "Share price",
                                x: names(&MONTHS),
                                candles: candles(),
                                style: values.str("style"),
                                y_min, y_max, y_ticks, x_title, y_title,
                                legend, tooltip, loading, zoom, onrange, handle,
                            }
                        }),
                        "radar" => rsx! {
                            RadarChart {
                                title: "Phones",
                                axes: FEATURES.map(|name| RadarAxis::from((name, 10.0))).to_vec(),
                                series: vec![Series::new("Phone A", PHONE_A), Series::new("Phone B", PHONE_B)],
                                filled: values.str("filled") == "true",
                                rings: values.str("rings").parse::<usize>().ok(),
                                legend, tooltip, loading, handle,
                            }
                        },
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
                                legend, tooltip, loading, zoom, onrange, handle,
                            }
                        }),
                    };
                    let drawn = with_buttons(&values, chart, drawn);
                    match values.str("table") == "true" {
                        true => rsx! {
                            div { width: "100%",
                                {drawn}
                                {data_table(&values)}
                            }
                        },
                        false => drawn,
                    }
                },
            }
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
