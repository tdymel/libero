# Chart

Crate: `libero`, feature `charts`
Import: `use libero::components::{BarChart, Candle, CandlestickChart, ChartHandle, ChartRange, FunnelChart, HeatmapChart, LineChart, PieChart, RadarAxis, RadarChart, ScatterChart, ScatterSeries, Series, Slice, Trend, use_chart};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/chart>
Index: [index.md](index.md) lists every other page
Description: Line, bar, pie, funnel, scatter, heatmap, candlestick and radar charts as SVG, with a tooltip, arrow keys, a toggle legend, and a summary for screen readers.

`LineChart`, `BarChart`, `PieChart`, `FunnelChart`, `ScatterChart`,
`HeatmapChart`, `CandlestickChart` and `RadarChart`, each
drawn as an SVG that fills its width, with a tooltip, the arrow keys, a legend
that hides and shows, and a summary for screen readers. Needs the `charts`
feature. Web and Android.

A chart only draws. To show its values as a table too, put them in a
[Table](table.md).

Lines and bars take one or more `Series` over a shared x axis; a pie takes
`Slice`s, each a name and a value, and a funnel takes them as its stages, top
to bottom; a heatmap takes a grid of values, a row per name in `y`. A line of another size or unit goes on a
right y axis with `.secondary_axis()`; in an upright bar chart, a series is
drawn as a line over the bars with `.kind(SeriesKind::Line)`. A scatter takes
`ScatterSeries` of x and y pairs, bubbles with a size as a third value, and fits
a `trend` line through each. A candlestick chart takes a `Candle` per period,
its open, high, low and close. A radar takes `RadarAxis` spokes and one
`Series` value per spoke.

Lines, bars and candlesticks zoom and pan their x axis with `zoom` and brush a
range for `onrange`. Every chart exports as SVG or PNG through a `use_chart()` handle,
which also resets a zoom and says whether the chart is zoomed; the chart draws
no buttons, render your own.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{LineChart, MarkLine, MarkPoint, Series};
use libero::theme::PointSymbol;

#[component]
fn Demo() -> Element {
    rsx! {
        LineChart {
            title: "Visitors",
            x: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
            series: vec![
                Series::new("This week", [120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0])
                    .mark_line(MarkLine::average())
                    .mark_point(MarkPoint::Max),
                Series::new("Last week", [98.0, 110.0, 115.0, 120.0, 104.0, 170.0, 180.0])
                    .dashed()
                    .symbol(PointSymbol::Square),
            ],
            area: true,
        }
    }
}
```

`Series::new` takes any numbers, or `Option`s of them: `None`, `NaN` and
infinity are gaps. `Series::from_rows("Revenue", &rows, |row| row.revenue)` and
`XValues::from_rows(&rows, |row| row.month.clone())` read a list of records.

`x` is one of two kinds. Text labels (`["Mon", "Tue"]`, `XValues::Categories`) sit
in evenly spaced slots, whatever they say. Numbers (`vec![0.0, 1.0, 6.0]`,
`XValues::Numbers`) sit at their value on a number axis, so uneven samples keep
their gaps; the axis ticks at round values and writes them with `value_format`.
`.color("success.7")` or `.color("#1e88e5")` sets a series' colour; otherwise it
takes the theme's palette for its place. `.unit("%")` gives a series its own unit
in the tooltip and summary; a unit every series shares labels the y axis.
Values group their thousands as the `Formats` say: `12,500` or `12.500`.
`.symbol(PointSymbol::Square)` draws a line's points as squares, `Triangle` or
`Diamond` as those: a cue beside the colour. Once one line sets its shape, every
legend key shows its line's. `area: true` fills under each line in its colour,
see-through at the theme's `area_opacity`, so the grid and the other lines show.

`BarChart` takes the same `Series`:

```rust
use dioxus::prelude::*;
use libero::components::{BarChart, Series};

#[component]
fn Demo() -> Element {
    rsx! {
        BarChart {
            title: "Sales",
            x: ["Q1", "Q2", "Q3", "Q4"],
            series: vec![
                Series::new("2025", [120.0, 132.0, 101.0, 134.0]),
                Series::new("2026", [140.0, 128.0, 150.0, 162.0]),
            ],
        }
    }
}
```

`layout: "stacked"` puts a category's bars on top of each other, `"percent"`
scales every stack to 100% so each bar is its series' share; the tooltip and the
summary keep the values. `orientation: "horizontal"` lays the categories down
the side, the first at the top. `value_labels: true` writes each bar's value on
it, inside the bar when stacked.

A bar is rounded at its free end only, the one away from the axis it grows from:
the top of a rising bar, the bottom of a falling one, the right end of a lying
one. The end at the axis and an end against another bar stay square, so a stack
rounds its topmost positive and its lowest negative segment.

```rust
use dioxus::prelude::*;
use libero::components::{BarChart, Series};

#[component]
fn Demo() -> Element {
    rsx! {
        BarChart {
            title: "Sales",
            x: ["Q1", "Q2", "Q3", "Q4"],
            series: vec![
                Series::new("2025", [120.0, 132.0, 101.0, 134.0]),
                Series::new("2026", [140.0, 128.0, 150.0, 162.0]),
            ],
            layout: "stacked",
            radius: "lg",
        }
    }
}
```

`bar_width` is the share of a category's band its bars fill, from `0.1` to `1`;
`bar_gap` the pixels between two bars of a group. `value_label_position` puts a
value `"inside"` its bar or `"outside"` past its end; unset, a lying bar or one with
error ranges writes inside, an upright one past its end. A stack always writes inside.

A series colours its bars one by one with `.colors(["success", "primary"])`, or
the ones below 0 with `.negative_color("error")`. `.errors([(low, high), ..])`
gives each value a range: a bar draws it as an error bar, and
the tooltip and the keys write it after the value, as "120 (110–130)".

```rust
use dioxus::prelude::*;
use libero::components::{BarChart, Series};

#[component]
fn Demo() -> Element {
    rsx! {
        BarChart {
            title: "Profit with its range",
            x: ["Q1", "Q2", "Q3", "Q4"],
            series: vec![
                Series::new("2025", [40.0, 55.0, -25.0, 62.0])
                    .negative_color("error")
                    .errors([(30.0, 52.0), (44.0, 63.0), (-31.0, -19.0), (50.0, 71.0)]),
            ],
            bar_width: 0.6,
            value_labels: true,
        }
    }
}
```

On a line, `.errors(..)` draws a see-through band in the line's colour from each
point's low to its high, as a forecast's or a measurement's range; a `stacked`
chart reads it in the tooltip alone.

```rust
use dioxus::prelude::*;
use libero::components::{LineChart, Series};

#[component]
fn Demo() -> Element {
    rsx! {
        LineChart {
            title: "Visitors",
            x: ["Mon", "Tue", "Wed", "Thu", "Fri"],
            series: vec![
                Series::new("Forecast", [120.0, 132.0, 101.0, 134.0, 90.0])
                    .errors([(110.0, 130.0), (120.0, 145.0), (90.0, 112.0), (120.0, 150.0), (78.0, 102.0)]),
            ],
        }
    }
}
```

`PieChart` takes slices:

```rust
use dioxus::prelude::*;
use libero::components::{PieChart, Slice};

#[component]
fn Demo() -> Element {
    rsx! {
        PieChart {
            title: "Browsers",
            donut: true,
            slices: vec![
                Slice::new("Chrome", 62.0),
                Slice::new("Safari", 20.0),
                Slice::new("Firefox", 10.0),
                Slice::new("Other", 8.0),
            ],
            center: rsx! { "1,204 users" },
        }
    }
}
```

`Slice::from(("Chrome", 62.0))` builds a slice from a pair; `.color("success")`
sets its colour, otherwise it takes the theme's palette for its place. A slice's
share is its value over the sum; a negative or missing value takes none.

`end_angle` ends the slices before a full turn: from `-90` to `90` degrees a
`donut` is a gauge, its `center` under the arc. `rose: true` gives every slice
the same angle and its value as its radius. `label_position: "inside"` writes a
label on its slice, `min_label_angle` drops the label of a slice narrower than
that many degrees. `slice_gap` sets the pixels between two slices, `slice_radius`
rounds their corners.

```rust
use dioxus::prelude::*;
use libero::components::{PieChart, Slice};

#[component]
fn Demo() -> Element {
    rsx! {
        PieChart {
            title: "Storage used",
            slices: vec![Slice::new("Used", 72.0), Slice::new("Free", 28.0)],
            donut: true,
            start_angle: -90.0,
            end_angle: 90.0,
            label_position: "inside",
            center: rsx! { "72 GB" },
        }
    }
}
```

`FunnelChart` takes its stages as slices, top to bottom, each as wide as its
value over the largest. Its `labels` write a stage's `percent` of the first
stage, its `value` or `name` on it; one wider than its stage is left out.

```rust
use dioxus::prelude::*;
use libero::components::{FunnelChart, Slice};

#[component]
fn Demo() -> Element {
    rsx! {
        FunnelChart {
            title: "Checkout",
            stages: vec![
                Slice::new("Visits", 1200.0),
                Slice::new("Product views", 860.0),
                Slice::new("Carts", 480.0),
                Slice::new("Checkouts", 300.0),
                Slice::new("Orders", 210.0),
            ],
        }
    }
}
```

`ScatterChart` takes points by two numbers, x and y. A third, a size, makes the
series bubbles, their area by it. `trend` fits a curve through each series'
points in its colour: `Trend::Linear`, `Exponential` (over the points above y 0),
`Logarithmic` (right of x 0) or `Polynomial(n)`, its order held to 2 to 6. Both
axes end on round numbers; `x_min`, `x_max`, `y_min` and `y_max` fix them.

```rust
use dioxus::prelude::*;
use libero::components::{ScatterChart, ScatterSeries, Trend};

#[component]
fn Demo() -> Element {
    rsx! {
        ScatterChart {
            title: "Ad spend and sales",
            x_title: "Ad spend (k€)",
            y_title: "Sales (k€)",
            trend: Trend::Linear,
            series: vec![
                ScatterSeries::new("Online", [(12.0, 34.0, 80.0), (25.0, 52.0, 90.0), (38.0, 71.0, 200.0)]),
                ScatterSeries::new("Stores", [(8.0, 22.0, 50.0), (22.0, 33.0, 110.0), (35.0, 47.0, 130.0)]),
            ],
        }
    }
}
```

`ScatterSeries::from_rows("Sales", &rows, |r| (r.price, r.sold))` builds a series
from your rows; `.color`, `.symbol` and `.unit` work as on `Series`.

`HeatmapChart` colours a grid of values on a scale drawn under the plot, and
never by colour alone: each cell writes its value where it fits, an empty one
is outlined. `steps` sorts the values into classes, `colors` sets the scale.

```rust
use dioxus::prelude::*;
use libero::components::HeatmapChart;

#[component]
fn Demo() -> Element {
    rsx! {
        HeatmapChart {
            title: "Visits by hour",
            x: ["8", "10", "12", "14", "16", "18"].map(String::from).to_vec(),
            y: ["Mon", "Tue", "Wed"].map(String::from).to_vec(),
            values: vec![
                vec![12.0, 30.0, 41.0, 35.0, 22.0, 9.0],
                vec![15.0, 34.0, 46.0, 38.0, 25.0, 11.0],
                vec![11.0, 28.0, 52.0, 44.0, 27.0, 10.0],
            ],
            steps: 5,
        }
    }
}
```

`CandlestickChart` draws a period's open, high, low and close as a candle, and
never tells the direction by colour alone: a rising candle is hollow, a falling
one filled, and a key under the plot says so. `style: "ohlc"` draws OHLC bars.

```rust
use dioxus::prelude::*;
use libero::components::{Candle, CandlestickChart};

#[component]
fn Demo() -> Element {
    rsx! {
        CandlestickChart {
            title: "Share price",
            x: ["Jan", "Feb", "Mar", "Apr"].map(String::from).to_vec(),
            candles: vec![
                Candle::new(32.0, 38.0, 30.0, 36.0),
                Candle::new(36.0, 41.0, 34.0, 35.0),
                Candle::new(35.0, 37.0, 29.0, 30.0),
                Candle::new(30.0, 34.0, 28.0, 33.0),
            ],
            unit: "€",
        }
    }
}
```

`RadarChart` puts several measures on spokes round a centre, clockwise from the
top: each `RadarAxis` runs from 0 to its `max` at the rim, unset the axis'
largest value rounded up. Each `Series` gives one value per axis; a missing one
is drawn at the centre. `filled` and `rings` set the shapes' fill and the web.

```rust
use dioxus::prelude::*;
use libero::components::{RadarAxis, RadarChart, Series};

#[component]
fn Demo() -> Element {
    rsx! {
        RadarChart {
            title: "Phones",
            axes: vec![
                RadarAxis::from(("Battery", 10.0)),
                ("Camera", 10.0).into(),
                ("Screen", 10.0).into(),
                ("Speed", 10.0).into(),
                ("Value", 10.0).into(),
            ],
            series: vec![
                Series::new("Phone A", [8.0, 6.0, 7.0, 9.0, 5.0]),
                Series::new("Phone B", [6.0, 9.0, 8.0, 6.0, 7.0]),
            ],
        }
    }
}
```

`y_min` and `y_max` fix the value axis' ends, `y_ticks` the steps between them.
`x_title` and `y_title` say what each axis counts; the summary reads them out.
Both `LineChart` and `BarChart` take them; a `horizontal` bar chart writes its
`y_title` under the values.

```rust
use dioxus::prelude::*;
use libero::components::{LineChart, Series};

#[component]
fn Demo() -> Element {
    rsx! {
        LineChart {
            title: "Visitors",
            x: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
            series: vec![Series::new("This week", [120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0])],
            y_min: 50.0,
            y_max: 250.0,
            y_ticks: 4usize,
            x_title: "Day",
            y_title: "Visits",
        }
    }
}
```

`.mark_line(MarkLine::average())` draws a dashed line across the plot at a
series' average, in its colour, and writes "Average: 140" by it; `min()` and
`max()` mark its lowest and highest value, `value(150.0)` a fixed one, and
`.label("Target")` names it. The summary reads each one out. A line chart that is
not `stacked` and upright, `grouped` bars draw them.

```rust
use dioxus::prelude::*;
use libero::components::{BarChart, MarkLine, Series};

#[component]
fn Demo() -> Element {
    rsx! {
        BarChart {
            title: "Sales",
            x: ["Q1", "Q2", "Q3", "Q4"],
            series: vec![
                Series::new("2026", [140.0, 128.0, 150.0, 162.0])
                    .mark_line(MarkLine::average())
                    .mark_line(MarkLine::value(150.0).label("Target")),
            ],
        }
    }
}
```

`.mark_point(MarkPoint::Max)` pins a line's highest value by its point, in a pill
of the series' colour with black or white text, whichever reads on it; `Min` its
lowest. A pin goes below a point too close to the top. The summary says where
each one is.

```rust
use dioxus::prelude::*;
use libero::components::{LineChart, MarkPoint, Series};

#[component]
fn Demo() -> Element {
    rsx! {
        LineChart {
            title: "Visitors",
            x: ["Mon", "Tue", "Wed", "Thu", "Fri"],
            series: vec![
                Series::new("This week", [120.0, 132.0, 101.0, 134.0, 90.0])
                    .mark_point(MarkPoint::Max)
                    .mark_point(MarkPoint::Min),
            ],
        }
    }
}
```

`.secondary_axis()` measures a line on a second y axis at the right, by its own
scale and in the unit its series share, titled by `y2_title`. Give it a `symbol`
or `dashed()` too, so its line is told apart without its colour. The summary
names the series on the right axis.

```rust
use dioxus::prelude::*;
use libero::components::{LineChart, Series};
use libero::theme::PointSymbol;

#[component]
fn Demo() -> Element {
    rsx! {
        LineChart {
            title: "Weather",
            x: ["Mon", "Tue", "Wed", "Thu", "Fri"],
            series: vec![
                Series::new("Temperature", [12.0, 15.0, 14.0, 18.0, 16.0]).unit("°C"),
                Series::new("Rain", [2.0, 0.0, 6.5, 1.0, 3.0])
                    .unit("mm")
                    .symbol(PointSymbol::Triangle)
                    .secondary_axis(),
            ],
            y_title: "Temperature",
            y2_title: "Rain",
        }
    }
}
```

`.kind(SeriesKind::Line)` draws a series as a line over the bars of an upright
`BarChart`, through the middle of each category, keyed by a stroke in the
legend. It never joins a stack; with `.secondary_axis()` it is read on the right
axis, titled by `y2_title`. The tooltip, the legend and the keys keep the
series' order. A `LineChart` given the same series draws them all as lines.

```rust
use dioxus::prelude::*;
use libero::components::{BarChart, Series, SeriesKind};

#[component]
fn Demo() -> Element {
    rsx! {
        BarChart {
            title: "Sales and margin",
            x: ["Q1", "Q2", "Q3", "Q4"],
            series: vec![
                Series::new("2025", [120.0, 132.0, 101.0, 134.0]),
                Series::new("2026", [140.0, 128.0, 150.0, 162.0]),
                Series::new("Margin", [18.0, 21.0, 15.0, 24.0])
                    .unit("%")
                    .kind(SeriesKind::Line)
                    .secondary_axis(),
            ],
            y_title: "Sales",
            y2_title: "Margin",
        }
    }
}
```

`stacked: true` draws each line on top of the ones before it, so the top one is
the total; with `area`, each fills down to the line below. The tooltip, the keys
and the summary keep each line's own value, and the summary says the top line is
the total. A line on the right y axis is not stacked, and the summary says so.

```rust
use dioxus::prelude::*;
use libero::components::{LineChart, Series};

#[component]
fn Demo() -> Element {
    rsx! {
        LineChart {
            title: "Visitors by device",
            x: ["Mon", "Tue", "Wed", "Thu", "Fri"],
            series: vec![
                Series::new("Desktop", [80.0, 92.0, 71.0, 84.0, 60.0]),
                Series::new("Mobile", [40.0, 40.0, 30.0, 50.0, 30.0]),
            ],
            area: true,
            stacked: true,
        }
    }
}
```

The pointer or the arrow keys pick a point, a bar or a slice: a tooltip shows
its values there. `onpointclick` reports a clicked one, and the legend's buttons
hide and show series or slices.

```rust
use dioxus::prelude::*;
use libero::components::{ChartPoint, LineChart, Series};

#[component]
fn Demo() -> Element {
    let mut picked = use_signal(|| None::<ChartPoint>);
    rsx! {
        LineChart {
            title: "Visitors",
            x: ["Mon", "Tue", "Wed"],
            series: vec![
                Series::new("This week", [120, 132, 101]),
                Series::new("Last week", [98, 110, 115]).dashed(),
            ],
            default_hidden: vec!["Last week".to_string()],
            onpointclick: move |point: ChartPoint| picked.set(Some(point)),
        }
        if let Some(point) = picked() {
            p { "{point.series_name} on {point.x}: {point.value}" }
        }
    }
}
```

`zoom: true` lets the reader zoom the x axis with Ctrl or Cmd and the wheel, a
pinch, a drag or `+`, `-` and `0`. With `onrange`
set, a drag brushes a band of x values instead, and Shift or Alt with the drag
pans. The chart draws no buttons: `use_chart()` gives a handle your own buttons
call. It reads the drawing through `svg()` and `png().await`, saves it with
`save_svg(name)` and `save_png(name)` (through `save_file`: a download on the
web, the share sheet on Android), resets the zoom with `reset_zoom()` and
tells in `is_zoomed()`, which re-runs the component that reads it, whether the
chart shows less than its whole axis.

```rust
use dioxus::prelude::*;
use libero::components::{Button, ChartRange, LineChart, Series, use_chart};

#[component]
fn Demo() -> Element {
    let chart = use_chart();
    let mut selected = use_signal(|| None::<ChartRange>);
    rsx! {
        LineChart {
            title: "Visitors",
            x: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
            series: vec![Series::new("This week", [120, 132, 101, 134, 90, 230, 210])],
            zoom: true,
            onrange: move |range| selected.set(range),
            handle: chart,
        }
        if let Some(range) = selected() {
            p { "Selected {range.from} to {range.to}" }
        }
        Button {
            onclick: move |_| async move {
                chart.save_png("visitors.png").await;
            },
            "Save as PNG"
        }
        Button {
            disabled: !chart.is_zoomed(),
            onclick: move |_| chart.reset_zoom(),
            "Reset zoom"
        }
    }
}
```

A chart only draws its data. Where every value matters, show the same rows in a
[Table](table.md) beside it: a screen reader can read it cell by cell, and a
reader can sort it.

```rust
use dioxus::prelude::*;
use libero::components::{BarChart, Series, Table, XValues, column};

#[derive(Clone, PartialEq)]
struct Sales {
    quarter: &'static str,
    last: f64,
    this: f64,
}

#[component]
fn Demo(rows: Vec<Sales>) -> Element {
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
```

## Props

### `LineChart`

| Prop | Type | Default | Description |
|---|---|---|---|
| `x` | `XValues` | required | The x axis, one value per point: labels, one evenly spaced slot each, or numbers, each point at its value. |
| `series` | `Vec<Series>` | required | The lines, each with one value per x value. `None` is a gap. |
| `title` | `Option<String>` | | Shown above the chart, and its accessible name. |
| `description` | `Option<String>` | | A sentence under the title: what to read from the chart. |
| `curve` | `Curve` | `linear` | `linear`, `smooth` or `step`. |
| `points` | `Option<bool>` | `true` | A dot on every point; never past the theme's `points_limit` points. |
| `area` | `Option<bool>` | `false` | The area under each line filled in its colour, at the theme's `area_opacity`. |
| `stacked` | `bool` | `false` | Each line drawn on top of the ones before it, so the top one is the total; with `area`, each fills down to the line below. The tooltip and the keys keep each line's own value. A line on the right y axis is not stacked. |
| `y_min` | `Option<f64>` | | The y axis' lowest value. Unset, 0 or the lowest value below it. |
| `y_max` | `Option<f64>` | | The y axis' highest value. Unset, a round number at or over the highest value. |
| `y_ticks` | `Option<usize>` | `6` | The steps the y axis is cut into: 4 writes 5 labels. |
| `x_title` | `Option<String>` | | Under the x axis: what it counts. |
| `y_title` | `Option<String>` | | Beside the y axis, read upward: what it measures. |
| `y2_title` | `Option<String>` | | Beside the right y axis, read downward: what its `Series::secondary_axis()` series measure. |
| `legend` | `LegendPosition` | bottom with two series or more | `bottom`, `top`, `start`, `end` or `none`. On a chart under 300 px wide, `start` goes above the plot and `end` below it. |
| `grid` | `Option<bool>` | `true` | Horizontal guide lines. |
| `value_format` | `ValueFormat` | `Auto` | How values are written on the axis, in the tooltip and in the summary. |
| `unit` | `Option<String>` | | After every value, as `"kg"`. |
| `aspect_ratio` | `Option<f32>` | `16 / 9` | Width over height of the plot. |
| `loading` | `bool` | `false` | A placeholder instead of the plot, and `aria-busy` on the chart. |
| `empty` | `Option<Element>` | | Shown in the plot when there is no value at all. Unset, "No data". |
| `tooltip` | `Option<bool>` | `true` | A tooltip and a crosshair at the pointer's or the keys' point. |
| `tooltip_content` | `Option<Callback<ChartPoint, Element>>` | | The tooltip's content instead of the x value and each line's value there. |
| `onpointclick` | `Option<EventHandler<ChartPoint>>` | | A click on a point, a second tap on a pinned one, or Enter on the keys' point. |
| `onactivechange` | `Option<EventHandler<Option<ChartPoint>>>` | | The point under the pointer or the keys; `None` when it leaves. |
| `hidden` | `Option<Vec<String>>` | | The names of the lines the legend hides; set, they are controlled. |
| `default_hidden` | `Vec<String>` | `[]` | Seeds the hidden lines once. Ignored when `hidden` is set. |
| `onhiddenchange` | `Option<EventHandler<Vec<String>>>` | | The hidden lines a legend button asks for. The last shown line stays. |
| `zoom` | `Option<bool>` | `false` | Zoom and pan the x axis: Ctrl or Cmd with the wheel, a pinch, a drag (Shift or Alt with it once `onrange` is set), the keys. Reset it from code with the `handle`. |
| `onrange` | `Option<EventHandler<Option<ChartRange>>>` | | Set, a drag or Shift with an arrow brushes a band of x values; Shift or Alt with a drag pans instead. `None` when Escape clears it. |
| `handle` | `Option<ChartHandle>` | | From `use_chart()`: reads the drawing as SVG or PNG, saves it, resets the zoom and says whether it is zoomed. |

### `BarChart`

| Prop | Type | Default | Description |
|---|---|---|---|
| `x` | `XValues` | required | The categories, one per bar or group of bars. |
| `series` | `Vec<Series>` | required | The bars' series, each with one value per x value. `None` leaves no bar. Upright, a `.kind(SeriesKind::Line)` one is drawn as a line over the bars. |
| `title` | `Option<String>` | | Shown above the chart, and its accessible name. |
| `description` | `Option<String>` | | A sentence under the title: what to read from the chart. |
| `layout` | `BarLayout` | `grouped` | `grouped` side by side, `stacked`, or `percent`: stacked to 100%. |
| `orientation` | `Orientation` | `vertical` | `horizontal` lays the categories down the side, top to bottom. |
| `radius` | `Size` | `xs` | Corner radius of a bar's free end, the one away from its axis, on the theme's radius scale. |
| `value_labels` | `Option<bool>` | `false` | Each bar's value written on it. |
| `y_min` | `Option<f64>` | | The value axis' lowest value. Unset, 0 or the lowest value below it. Ignored by `percent`, which runs 0 to 100%. |
| `y_max` | `Option<f64>` | | The value axis' highest value. Unset, a round number at or over the highest value. Ignored by `percent`. |
| `y_ticks` | `Option<usize>` | `6` | The steps the value axis is cut into: 4 writes 5 labels. |
| `x_title` | `Option<String>` | | Along the categories: what they are. Under them, or beside them when `horizontal`. |
| `y_title` | `Option<String>` | | Along the values: what they measure. Beside them, or under them when `horizontal`. |
| `y2_title` | `Option<String>` | | Beside the right y axis, read downward: what its `Series::secondary_axis()` series measure. An upright chart only. |
| `value_label_position` | `BarLabelPosition` | chart-wide: inside on a `horizontal` chart or once any shown series has error ranges, past the end otherwise | `inside` the bar, in the ink that reads on it, or `outside` past its end. A stack always writes inside. |
| `bar_width` | `Option<f32>` | 5 px off each side of the band | The share of a category's band its bars fill, clamped to `0.1..=1`. |
| `bar_gap` | `Option<f32>` | `3` | Between two bars of a group, in pixels. |
| `legend` | `LegendPosition` | bottom with two series or more | `bottom`, `top`, `start`, `end` or `none`. On a chart under 300 px wide, `start` goes above the plot and `end` below it. |
| `grid` | `Option<bool>` | `true` | Guide lines across the values. |
| `value_format` | `ValueFormat` | `Auto` | How values are written on the axis, the bars, in the tooltip and in the summary. On `percent` the axis and the bars write shares in percent. |
| `unit` | `Option<String>` | | After every value, as `"kg"`. |
| `aspect_ratio` | `Option<f32>` | `16 / 9` | Width over height of the plot. |
| `loading` | `bool` | `false` | A placeholder instead of the plot, and `aria-busy` on the chart. |
| `empty` | `Option<Element>` | | Shown in the plot when there is no value at all. Unset, "No data". |
| `tooltip` | `Option<bool>` | `true` | A tooltip at the pointer's or the keys' bar. |
| `tooltip_content` | `Option<Callback<ChartPoint, Element>>` | | The tooltip's content instead of the category and each series' value there. |
| `onpointclick` | `Option<EventHandler<ChartPoint>>` | | A click on a bar, a second tap on a pinned one, or Enter on the keys' bar. |
| `onactivechange` | `Option<EventHandler<Option<ChartPoint>>>` | | The bar under the pointer or the keys; `None` when it leaves. |
| `hidden` | `Option<Vec<String>>` | | The names of the series the legend hides; set, they are controlled. |
| `default_hidden` | `Vec<String>` | `[]` | Seeds the hidden series once. Ignored when `hidden` is set. |
| `onhiddenchange` | `Option<EventHandler<Vec<String>>>` | | The hidden series a legend button asks for. The last shown series stays. |
| `zoom` | `Option<bool>` | `false` | Zoom and pan along the categories: Ctrl or Cmd with the wheel, a pinch, a drag (Shift or Alt with it once `onrange` is set), the keys. Reset it from code with the `handle`. |
| `onrange` | `Option<EventHandler<Option<ChartRange>>>` | | Set, a drag or Shift with an arrow brushes a band of categories; Shift or Alt with a drag pans instead. `None` when Escape clears it. |
| `handle` | `Option<ChartHandle>` | | From `use_chart()`: reads the drawing as SVG or PNG, saves it, resets the zoom and says whether it is zoomed. |

### `FunnelChart`

| Prop | Type | Default | Description |
|---|---|---|---|
| `stages` | `Vec<Slice>` | required | The stages, top to bottom, each as wide as its value over the largest. Drawn widest first: a stage larger than the one before it moves up, with a warning in development. A negative value has no width. |
| `title` | `Option<String>` | | Shown above the chart, and its accessible name. |
| `description` | `Option<String>` | | A sentence under the title: what to read from the chart. |
| `labels` | `PieLabels` | `percent` | On each stage: `percent`, its share of the first stage, `value`, `name` or `none`, in the ink that reads there. A label wider than its stage is left out; the tooltip and the summary keep it. |
| `legend` | `LegendPosition` | bottom with two stages or more | `bottom`, `top`, `start`, `end` or `none`. On a chart under 300 px wide, `start` goes above the plot and `end` below it. |
| `value_format` | `ValueFormat` | `Auto` | How values are written on the stages, in the tooltip and in the summary. |
| `unit` | `Option<String>` | | After every value, as `"kg"`. |
| `grid` | `Option<bool>` | | Shared with the other charts; a funnel has no grid to draw. |
| `aspect_ratio` | `Option<f32>` | `4 / 3` | Width over height of the plot. Unset, the theme's `pie_aspect_ratio`. |
| `loading` | `bool` | `false` | A placeholder instead of the plot, and `aria-busy` on the chart. |
| `empty` | `Option<Element>` | | Shown in the plot when no stage has a value. Unset, "No data". |
| `tooltip` | `Option<bool>` | `true` | A tooltip with the stage's value at the pointer's or the keys' stage. |
| `tooltip_content` | `Option<Callback<ChartPoint, Element>>` | | The tooltip's content instead of the stage's name and value. |
| `onpointclick` | `Option<EventHandler<ChartPoint>>` | | A click on a stage, a second tap on a pinned one, or Enter on the keys' stage. `index` is the stage's place in `stages`, `x` and `series_name` its name. |
| `onactivechange` | `Option<EventHandler<Option<ChartPoint>>>` | | The stage under the pointer or the keys; `None` when it leaves. |
| `hidden` | `Option<Vec<String>>` | | The names of the stages the legend hides; set, they are controlled. |
| `default_hidden` | `Vec<String>` | `[]` | Seeds the hidden stages once. Ignored when `hidden` is set. |
| `onhiddenchange` | `Option<EventHandler<Vec<String>>>` | | The hidden stages a legend button asks for. The last shown stage stays. |
| `handle` | `Option<ChartHandle>` | | From `use_chart()`: reads the drawing as SVG or PNG, saves it. |

### `HeatmapChart`

| Prop | Type | Default | Description |
|---|---|---|---|
| `x` | `Vec<String>` | required | The columns' names, left to right. |
| `y` | `Vec<String>` | required | The rows' names, top to bottom. |
| `values` | `Vec<Vec<f64>>` | required | A row per `y`, a value per `x`. NaN, or a value missing at a row's end, is an empty cell, outlined; a row of the wrong length warns in development. |
| `colors` | `Vec<ChartColor>` | light to strong primary | The scale from the lowest value's colour to the highest's, two or more, as `"primary.2"` or a hex. |
| `min` | `Option<f64>` | | The scale's low end. Unset, the lowest value. |
| `max` | `Option<f64>` | | The scale's high end. Unset, the highest value. |
| `steps` | `Option<usize>` | | Sorts the values into this many classes of the same width, one colour each, instead of a gradient. |
| `cell_labels` | `Option<bool>` | `true` | Each value written in its cell, in black or white by the cell. A value wider or taller than its cell is left out; the tooltip and the summary still have it. |
| `title` | `Option<String>` | | Shown above the chart, and its accessible name. |
| `description` | `Option<String>` | | A sentence under the title: what to read from the chart. |
| `legend` | `LegendPosition` | bottom | The colour scale, under the plot; `none` leaves it out. |
| `value_format` | `ValueFormat` | `Auto` | How values are written in the cells, on the scale, in the tooltip and in the summary. |
| `unit` | `Option<String>` | | After every value, as `"°C"`. |
| `grid` | `Option<bool>` | | Shared with the other charts; a heatmap has no grid to draw. |
| `hidden` | `Option<Vec<String>>` | | Shared with the other charts; a heatmap hides no rows. |
| `default_hidden` | `Vec<String>` | `[]` | Shared with the other charts; a heatmap hides no rows. |
| `onhiddenchange` | `Option<EventHandler<Vec<String>>>` | | Shared with the other charts; never called by a heatmap. |
| `aspect_ratio` | `Option<f32>` | `4 / 3` | Width over height of the plot and its scale. Unset, the theme's `pie_aspect_ratio`, so rows keep room for their names. |
| `loading` | `bool` | `false` | A placeholder instead of the plot, and `aria-busy` on the chart. |
| `empty` | `Option<Element>` | | Shown in the plot when no cell has a value. Unset, "No data". |
| `tooltip` | `Option<bool>` | `true` | A tooltip with the cell's column, row and value at the pointer's or the keys' cell. |
| `tooltip_content` | `Option<Callback<ChartPoint, Element>>` | | The tooltip's content instead of the cell's column, row and value. |
| `onpointclick` | `Option<EventHandler<ChartPoint>>` | | A click on a cell, a second tap on a pinned one, or Enter on the keys' cell. `series` and `series_name` are its row, `index` and `x` its column. |
| `onactivechange` | `Option<EventHandler<Option<ChartPoint>>>` | | The cell under the pointer or the keys; `None` when it leaves. |
| `handle` | `Option<ChartHandle>` | | From `use_chart()`: reads the drawing as SVG or PNG, saves it. |

### `CandlestickChart`

| Prop | Type | Default | Description |
|---|---|---|---|
| `x` | `Vec<String>` | required | The periods' names, left to right, evenly spaced. |
| `candles` | `Vec<Candle>` | required | A candle per `x`, `Candle::new(open, high, low, close)` or from a tuple. A price that is not a number, or a candle missing at the end, leaves a gap; a high under the body or a low over it warns in development. |
| `name` | `Option<String>` | "Price" | The prices' name in the tooltip and the summary. |
| `style` | `CandleStyle` | candle | `candle`: a body from the open to the close on a wick, hollow when it rose, filled when it fell. `ohlc`: a bar from the low to the high with the open ticked left and the close right. |
| `up_color` | `Option<ChartColor>` | success | A rising candle's colour, as `"success.6"` or a hex. |
| `down_color` | `Option<ChartColor>` | error | A falling candle's colour. |
| `title` | `Option<String>` | | Shown above the chart, and its accessible name. |
| `description` | `Option<String>` | | A sentence under the title: what to read from the chart. |
| `y_min` | `Option<f64>` | | The price axis' lowest value. Unset, a round number at or under the lowest low. |
| `y_max` | `Option<f64>` | | The price axis' highest value. Unset, a round number at or over the highest high. |
| `y_ticks` | `Option<usize>` | `6` | The steps the price axis is cut into. |
| `x_title` | `Option<String>` | | Under the periods: what they are. |
| `y_title` | `Option<String>` | | Beside the price axis, read upward: what it measures. |
| `zoom` | `Option<bool>` | `false` | Zoom and pan the periods: Ctrl or Cmd with the wheel, a pinch, a drag (Shift or Alt with it once `onrange` is set), the keys. Reset it from code with the `handle`. |
| `onrange` | `Option<EventHandler<Option<ChartRange>>>` | | Set, a drag or Shift with an arrow brushes a band of periods; Shift or Alt with a drag pans instead. `None` when Escape clears it. |
| `legend` | `LegendPosition` | bottom | The Up and Down key, under the plot; `none` leaves it out. |
| `grid` | `Option<bool>` | `true` | Horizontal guide lines. |
| `value_format` | `ValueFormat` | `Auto` | How prices are written on the axis, in the tooltip and in the summary. |
| `unit` | `Option<String>` | | After every price, as `"€"`. |
| `hidden` | `Option<Vec<String>>` | | Shared with the other charts; a candlestick chart hides nothing. |
| `default_hidden` | `Vec<String>` | `[]` | Shared with the other charts; a candlestick chart hides nothing. |
| `onhiddenchange` | `Option<EventHandler<Vec<String>>>` | | Shared with the other charts; never called by a candlestick chart. |
| `aspect_ratio` | `Option<f32>` | `16 / 9` | Width over height of the plot and its key. |
| `loading` | `bool` | `false` | A placeholder instead of the plot, and `aria-busy` on the chart. |
| `empty` | `Option<Element>` | | Shown in the plot when no candle has its four prices. Unset, "No data". |
| `tooltip` | `Option<bool>` | `true` | A tooltip with the period's open, high, low and close at the pointer's or the keys' candle. |
| `tooltip_content` | `Option<Callback<ChartPoint, Element>>` | | The tooltip's content instead of the period and its prices. |
| `onpointclick` | `Option<EventHandler<ChartPoint>>` | | A click on a candle, a second tap on a pinned one, or Enter on the keys' candle. `index` is its place in `candles`, `value` its close. |
| `onactivechange` | `Option<EventHandler<Option<ChartPoint>>>` | | The candle under the pointer or the keys; `None` when it leaves. |
| `handle` | `Option<ChartHandle>` | | From `use_chart()`: reads the drawing as SVG or PNG, saves it, resets the zoom and says whether it is zoomed. |

### `PieChart`

| Prop | Type | Default | Description |
|---|---|---|---|
| `slices` | `Vec<Slice>` | required | The slices, each a share of their sum. A negative value takes no share. |
| `title` | `Option<String>` | | Shown above the chart, and its accessible name. |
| `description` | `Option<String>` | | A sentence under the title: what to read from the chart. |
| `donut` | `bool` | `false` | A ring instead of a full pie, `center` in its middle. |
| `labels` | `PieLabels` | `percent` | Beside each slice: `percent`, `value`, `name` or `none`. One too long for the room ends in an ellipsis; one that would overlap a larger slice's is left out. The tooltip and the summary write each whole. |
| `center` | `Option<Element>` | | In a donut's hole, as a total, kept off the ring; its text shrinks in a small hole, the rest is clipped. |
| `start_angle` | `f32` | `0` | Where the first slice starts, in degrees clockwise from 12 o'clock. |
| `end_angle` | `Option<f32>` | a full turn | Where the last slice ends, in the same degrees: `start_angle: -90.0, end_angle: 90.0` is a half pie, a gauge with `donut`. At or before the start, it counts a turn on: `90.0` to `-90.0` is the lower half. |
| `rose` | `bool` | `false` | A nightingale rose: every slice the same angle, its radius by its value. |
| `label_position` | `PieLabelPosition` | `outside` | `outside` beside each slice, or `inside` on it in the ink that reads there. Inside, a label too wide or too tall for its slice is left out; the summary keeps it. |
| `min_label_angle` | `Option<f32>` | `0` | A slice narrower than this many degrees gets no label; the summary still names it. |
| `slice_gap` | `Option<f32>` | `1` | Between two slices, in pixels, in the page's colour. |
| `slice_radius` | `Size` | `none` | Corner radius of a slice, on the theme's radius scale; a thin slice or a thin ring takes less. Unset, square. |
| `legend` | `LegendPosition` | bottom with two slices or more | `bottom`, `top`, `start`, `end` or `none`. On a chart under 300 px wide, `start` goes above the plot and `end` below it. |
| `value_format` | `ValueFormat` | `Auto` | How values are written beside the slices, in the tooltip and in the summary. |
| `unit` | `Option<String>` | | After every value, as `"kg"`. |
| `grid` | `Option<bool>` | | Shared with the other charts; a pie has no grid to draw. |
| `aspect_ratio` | `Option<f32>` | `4 / 3` | Width over height of the plot. Unset, the theme's `pie_aspect_ratio`, squarer than the other charts' `aspect_ratio`. |
| `loading` | `bool` | `false` | A placeholder instead of the plot, and `aria-busy` on the chart. |
| `empty` | `Option<Element>` | | Shown in the plot when the slices sum to nothing. Unset, "No data". |
| `tooltip` | `Option<bool>` | `true` | A tooltip with the slice's value and share at the pointer's or the keys' slice. |
| `tooltip_content` | `Option<Callback<ChartPoint, Element>>` | | The tooltip's content instead of the slice's name, value and share. |
| `onpointclick` | `Option<EventHandler<ChartPoint>>` | | A click on a slice, a second tap on a pinned one, or Enter on the keys' slice. `index` is the slice's place in `slices`, `x` and `series_name` its name. |
| `onactivechange` | `Option<EventHandler<Option<ChartPoint>>>` | | The slice under the pointer or the keys; `None` when it leaves. |
| `hidden` | `Option<Vec<String>>` | | The names of the slices the legend hides; set, they are controlled. |
| `default_hidden` | `Vec<String>` | `[]` | Seeds the hidden slices once. Ignored when `hidden` is set. |
| `onhiddenchange` | `Option<EventHandler<Vec<String>>>` | | The hidden slices a legend button asks for. The last shown slice stays. |
| `handle` | `Option<ChartHandle>` | | From `use_chart()`: reads the drawing as SVG or PNG, saves it. |

### `ScatterChart`

| Prop | Type | Default | Description |
|---|---|---|---|
| `series` | `Vec<ScatterSeries>` | required | The groups of points, each an x and a y, `ScatterSeries::new("Online", [(12.0, 34.0)])`. A third value, a size, makes the series bubbles, their area by it. A point with an x or y that is not a number is left out. |
| `title` | `Option<String>` | | Shown above the chart, and its accessible name. |
| `description` | `Option<String>` | | A sentence under the title: what to read from the chart. |
| `trend` | `Option<Trend>` | | A curve fitted through each series' points, in its colour: `Linear`, `Exponential` over the points above y 0, `Logarithmic` over those right of x 0, or `Polynomial(n)`, its order held to 2 to 6. |
| `x_min` | `Option<f64>` | | The x axis' lowest value. Unset, a round number at or under the lowest x. |
| `x_max` | `Option<f64>` | | The x axis' highest value. Unset, a round number at or over the highest x. |
| `y_min` | `Option<f64>` | | The y axis' lowest value. Unset, a round number at or under the lowest y. |
| `y_max` | `Option<f64>` | | The y axis' highest value. Unset, a round number at or over the highest y. |
| `y_ticks` | `Option<usize>` | `4` | About how many steps each axis is cut into. |
| `x_title` | `Option<String>` | | Under the x axis: what it measures. |
| `y_title` | `Option<String>` | | Beside the y axis, read upward: what it measures. |
| `legend` | `LegendPosition` | bottom with two series or more | `bottom`, `top`, `start`, `end` or `none`. Each key is its series' point shape. |
| `grid` | `Option<bool>` | `true` | Horizontal guide lines. |
| `value_format` | `ValueFormat` | `Auto` | How values are written on the axes, in the tooltip and in the summary. |
| `unit` | `Option<String>` | | After every y value, as `"kg"`; a series' own `unit` wins. |
| `aspect_ratio` | `Option<f32>` | `16 / 9` | Width over height of the plot. |
| `loading` | `bool` | `false` | A placeholder instead of the plot, and `aria-busy` on the chart. |
| `empty` | `Option<Element>` | | Shown in the plot when there is no point at all. Unset, "No data". |
| `tooltip` | `Option<bool>` | `true` | A tooltip and a crosshair at the pointer's nearest or the keys' point. |
| `tooltip_content` | `Option<Callback<ChartPoint, Element>>` | | The tooltip's content instead of the series and the point's x, y and size. |
| `onpointclick` | `Option<EventHandler<ChartPoint>>` | | A click on a point, a second tap on a pinned one, or Enter on the keys' point. `index` is the point's place in its series. |
| `onactivechange` | `Option<EventHandler<Option<ChartPoint>>>` | | The point under the pointer or the keys; `None` when it leaves. |
| `hidden` | `Option<Vec<String>>` | | The names of the series the legend hides; set, they are controlled. |
| `default_hidden` | `Vec<String>` | `[]` | Seeds the hidden series once. Ignored when `hidden` is set. |
| `onhiddenchange` | `Option<EventHandler<Vec<String>>>` | | The hidden series a legend button asks for. The last shown series stays. |
| `handle` | `Option<ChartHandle>` | | From `use_chart()`: reads the drawing as SVG or PNG, saves it. |

### `RadarChart`

| Prop | Type | Default | Description |
|---|---|---|---|
| `axes` | `Vec<RadarAxis>` | required | The spokes, clockwise from the top, at least three: `"Speed".into()` or `("Speed", 10.0).into()`, the value at its rim. Unset, the rim is the axis' largest value rounded up to 1, 2, 2.5 or 5 times a power of ten. Fewer than three show the empty state. |
| `series` | `Vec<Series>` | required | One value per axis each, in the axes' order. A missing value is drawn at the centre; a series with more or fewer values than axes warns in development and is cut or padded. |
| `title` | `Option<String>` | | Shown above the chart, and its accessible name. |
| `description` | `Option<String>` | | A sentence under the title: what to read from the chart. |
| `filled` | `Option<bool>` | `true` | Each series' shape filled, translucent, inside its outline. |
| `rings` | `Option<usize>` | `5` | The web's rings, its steps from the centre to the rim. |
| `legend` | `LegendPosition` | bottom with two series or more | `bottom`, `top`, `start`, `end` or `none`. Each key is its series' point shape. |
| `grid` | `Option<bool>` | `true` | The web: its rings and spokes. |
| `value_format` | `ValueFormat` | `Auto` | How values are written in the tooltip and in the summary. |
| `unit` | `Option<String>` | | After every value, as `"pts"`; a series' own `unit` wins. |
| `aspect_ratio` | `Option<f32>` | `4 / 3` | Width over height of the plot. Unset, the theme's `pie_aspect_ratio`. |
| `loading` | `bool` | `false` | A placeholder instead of the plot, and `aria-busy` on the chart. |
| `empty` | `Option<Element>` | | Shown in the plot when there is no value or fewer than three axes. Unset, "No data". |
| `tooltip` | `Option<bool>` | `true` | A tooltip with each series' value on the axis of the pointer's nearest or the keys' point. |
| `tooltip_content` | `Option<Callback<ChartPoint, Element>>` | | The tooltip's content instead of the axis and each series' value there. |
| `onpointclick` | `Option<EventHandler<ChartPoint>>` | | A click on a point, a second tap on a pinned one, or Enter on the keys' point. `index` is the axis' place in `axes`, `x` its name. |
| `onactivechange` | `Option<EventHandler<Option<ChartPoint>>>` | | The point under the pointer or the keys; `None` when it leaves. |
| `hidden` | `Option<Vec<String>>` | | The names of the series the legend hides; set, they are controlled. |
| `default_hidden` | `Vec<String>` | `[]` | Seeds the hidden series once. Ignored when `hidden` is set. |
| `onhiddenchange` | `Option<EventHandler<Vec<String>>>` | | The hidden series a legend button asks for. The last shown series stays. |
| `handle` | `Option<ChartHandle>` | | From `use_chart()`: reads the drawing as SVG or PNG, saves it. |

Like every component, each chart also takes the shared props `sx`, `class`,
`style`, `states`, `parts` (`ChartPart`), and any extra HTML attributes.

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` | Into the plot and the legend's buttons in the order drawn: a legend at the top or the start comes first. |
| `ArrowRight` / `ArrowLeft` | Line: the next or previous point of the line, as drawn, left to right in either text direction. Bar: the next or previous category's bar of the series; `horizontal`, the next or previous series' bar in the category. Pie: the next or previous slice, clockwise. Funnel: the next or previous stage, top down. Scatter: the series' next or previous point, left to right. Heatmap: the next or previous cell in the row, left to right. Candlestick: the next or previous candle, left to right. Radar: the series' point on the next or previous axis, clockwise. |
| `ArrowDown` / `ArrowUp` | Line: the next or previous line at the same x value, or its nearest point. Bar: the next or previous series' bar in the category; `horizontal`, the next or previous category, top to bottom. Pie and funnel: the next or previous slice or stage. Scatter: the next or previous series' point nearest in x. Heatmap: the row below or above, at the nearest column. Radar: the next or previous series' point on the same axis. |
| `Home` / `End` | The line's or the series' first or last point or bar; a pie's first or last slice, a funnel's top or bottom stage; a scatter series' leftmost or rightmost point; a heatmap row's first or last cell; the first or last candle; a radar series' point on the first or last axis. |
| `Enter` / `Space` | Calls `onpointclick` with the point, bar or slice. |
| `Escape` | Clears the brushed band, else the point and its tooltip. A tooltip a pointer hovers closes with Escape wherever focus is, and stays closed until the pointer leaves the plot. |
| `+` / `-` | With `zoom`: zooms the x axis, a lying bar chart's categories, in or out around the keys' point; the arrows carry the window along at its edge. |
| `0` | With `zoom`: shows every x value or category again. |
| `Shift+ArrowRight` / `Shift+ArrowLeft` | With `onrange`: brushes from the keys' point to the next or previous x value. |

### Libero handles

- The chart is a `role="figure"` named by `title`, else by a spread
  `aria-label`, else "chart". The drawing is hidden from screen readers.
- A loading chart says "Loading chart" in a live region when it starts, and its
  summary says it too.
- A hidden summary gives the x range and each series' lowest and highest value,
  and where the highest is, then each mark line's name and value and each marked
  point's place and value, each line's error band and each bar series'
  error bars with their outer ends, the series drawn as a line over
  bars, the series on the right y axis, the axes' titles and, stacked, that the
  top line is the total; a pie's, the total, then each slice's value and share;
  a funnel's, its first and last stage, then each stage's value and share of
  the first; a scatter's, each series' count of points and its x and y ranges,
  a bubble series' sizes and each trend line's kind and direction, as "Online:
  linear trend, rising."; a heatmap's, its rows by columns and its lowest and
  highest value, the highest and the lowest cell, then each row's cells (above
  12 rows or 150 cells, a pointer to the keys instead); a candlestick chart's,
  its periods, the first open and the last close with the change, the highest
  high and the lowest low with their periods, and how many candles rose, fell
  and stayed; a radar's, its axes, then each series' highest and lowest axis
  and the axes it has no value on.
- The plot is one tab stop, a `role="application"` described as a chart, so a
  screen reader hands it the arrows. Each point the keys reach is announced, as
  "This week, Tue: 132", "Chrome: 62, 62%." "Online: x 12, y 34, size 80" or, for a heatmap's cell, "Tue, 12: 46"; a candle reads its four prices and its direction, as "Price, May: open 33, high 44, low 32, close 43, up".
- A heatmap never tells by colour alone: each cell writes its value where it
  fits, in black or white by the cell; an empty cell is outlined, dashed, and
  the scale under the plot keys it as "No value".
- A candle never tells its direction by colour alone: a rising one is hollow, a
  falling one filled, and the key under the plot says which is which; an OHLC
  bar ticks its open left and its close right.
- The legend's entries are toggle buttons: pressed while their series or slice
  shows, struck through when hidden. The last shown one's button is
  `aria-disabled`. Should `hidden` or `default_hidden` name every one, the plot
  and the summary say "All series are hidden." and the legend stays. A dashed
  line's key is a dash, as drawn; once a line sets a `symbol`, each key is its
  line's point shape. A line among bars is keyed by a short stroke, the bars by
  squares.
- On a touch screen a tap pins a point's tooltip, a second tap on it clicks; a
  vertical swipe still scrolls the page.
- With `zoom`, the shown range is announced once a zoom or pan settles, as
  "Showing Tue to Fri". Ctrl or Cmd with the wheel zooms only a zoomable plot, so
  the page zoom works everywhere else.
- A drag with `onrange` set brushes; Shift or Alt with it pans. The band is
  outlined as well as tinted, and announced as "Selected Tue to Thu".
- A saved SVG is self-contained: its colours resolved, its font named, the
  chart's `title` as its `<title>`.
- The chart draws no buttons: the ones you render from the `use_chart()` handle
  are yours to name, to announce what they saved and to keep focusable while
  there is nothing to reset.
- Each series' or slice's palette colour reaches 3:1 on the page and on a card,
  in every shipped theme and color scheme; one too close to an earlier one, also
  for colour-blind eyes, is skipped.
- In forced colours the lines, bars and slices keep their colours on the system
  background; axes and labels take the system text colour.
- A bar's or a point's error range is written after its value in the tooltip
  and the announcement, as "120 (110–130)".
- A label on a bar or a slice takes black or white, whichever reads on it.

### You must

- Give a `title`, or name the chart with `aria_label`.
- Say in `description` what the reader should see, as "Visitors peaked on
  Saturday".
- Tell series apart by more than colour: `dashed()` or `symbol()` on a line, the legend,
  `value_labels` on bars, `labels` on a pie.
- Name every series and slice differently: the legend hides them by name.
- Keep a pie to a few slices; fold the small ones into one "Other".
- Where every value matters, show the data in a `Table` beside the chart too.

### Limits

- A hex colour (`.color("#1e88e5")`) is drawn as given: check its 3:1 on your
  page.
- The tooltip is for the eye; screen readers hear the announcement instead, so
  `tooltip_content` reaches only sighted readers.
- The drawing runs left to right in a right-to-left page too, and the arrows
  follow it unflipped.
- A hidden slice leaves the drawing, so the shown ones' shares grow in the
  labels, the tooltip and the announcement; the summary keeps every slice and
  its share of all, and says so while one is hidden.
- The summary describes the data, not the drawing: a series the legend hides
  stays in it, with its mark lines and error band.
- The tooltip follows the point and takes no pointer, so it is not hoverable
  (WCAG 1.4.13); Escape closes it and the announcement gives the same values.
- `grid` draws nothing on a pie.
- `.colors` and `.negative_color` add no meaning of their own: a loss already
  hangs below the axis. The legend keeps the series' colour.
- A percent stack draws no error bars and reads the range in its tooltip; on any
  other layout the value axis reaches the far end of a range unless `y_min` or
  `y_max` is set. A line draws its ranges as a band, only around its points and
  not when `stacked`.
- A rose's radius and a slice under `min_label_angle` are read by eye or not at
  all; the summary gives every value.
- `zoom` and `onrange` are for lines, bars and candlesticks; a pie, a funnel, a
  scatter, a heatmap and a radar export only.
- A radar's web writes no values on its rings; each axis may have its own rim,
  so the tooltip, the announcement and the summary give the values. A value past
  its axis' `max` is drawn at the rim.
- A bubble's size and a trend line are read by eye; the tooltip, the
  announcement and the summary give the sizes and each trend's direction.
  Bubbles are drawn largest first, so a small one stays on top.
- A heatmap's scale is drawn under the plot whatever `legend` says, but `none`;
  in forced colours its cells keep the scale's colours and their black or white
  values; the axes, the names and an empty cell's outline take the system's.
- A funnel is drawn widest stage first; the keys, the summary and the legend
  keep the order of `stages`. A hidden stage leaves the drawing and the others widen to
  the largest shown; percent labels stay shares of the first stage.
- An export keeps the normal palette in forced colours. `png()` is `None` on
  Blitz and on the server, and `save_png` returns `SaveOutcome::Failed`.
- A mark line is drawn on a line chart that is not `stacked`, on upright,
  `grouped` bars and on a line among upright bars, and only inside the y axis'
  ends; the summary leaves out the ones not drawn, except past the ends. A
  marked point is drawn on a line only, in a line chart or among upright bars.
- A `SeriesKind::Line` series is drawn as a line on an upright `BarChart` only;
  a `horizontal` one draws it as a bar and warns in development. A line among
  bars never stacks, and on a `percent` chart it is read on the right y axis.
- A right y axis is drawn on a line chart and on an upright bar chart, though
  not for a `percent` chart's bars, which share one scale; `y_min` and `y_max`
  set the left one, a `stacked` line chart stacks only the left one's lines, and
  stacked bars stack apart on each axis. Give its series a `symbol` or
  `dashed()` and a `y2_title`, so the reader can tell which axis a series is
  read on.

## Theme defaults

`ChartDefaults` on the theme, `Theme::chart`, shared by every chart.

| Field | Type | Description |
|---|---|---|
| `aspect_ratio` | `f32` | Width over height of a line's or a bar chart's plot. |
| `pie_aspect_ratio` | `f32` | Width over height of a pie's and a radar's plot, `4 / 3`: squarer, so the circle fills more of it. |
| `palette` | `[(Color, ColorShade); 8]` | Series and slice colours in order, each at least 3:1 on the page (a shade darker if not); one that reads like an earlier one is skipped; past the last they start over, a line dashed. |
| `curve` | `Curve` | Default `curve`. |
| `legend` | `LegendPosition` | Where the legend sits with two series or slices or more. |
| `grid` | `bool` | Default `grid`. |
| `points` | `bool` | Default `points`. |
| `points_limit` | `usize` | No dots past this many points. |
| `tooltip` | `bool` | Default `tooltip`. |
| `stroke_width` | `f32` | Line width, in pixels. |
| `area` | `bool` | Default `area`. |
| `area_opacity` | `f32` | How opaque an area under a line is, from 0 to 1. |
| `y_ticks` | `usize` | Default `y_ticks`. |
| `bar_layout` | `BarLayout` | Default `layout`. |
| `bar_radius` | `Size` | Default `radius`. |
| `value_labels` | `bool` | Default `value_labels`. |
| `bar_label_position` | `Option<BarLabelPosition>` | Default `value_label_position`; `None` inside a stack, a lying chart or one where any shown series has error ranges, else past the end. |
| `bar_width` | `Option<f32>` | Default `bar_width`; `None` leaves 5 px each side. |
| `bar_gap` | `f32` | Default `bar_gap`, in pixels. |
| `donut_thickness` | `f32` | A donut's ring width over the pie's radius. |
| `pie_labels` | `PieLabels` | Default `labels`. |
| `pie_label_position` | `PieLabelPosition` | Default `label_position`. |
| `slice_gap` | `f32` | Default `slice_gap`, in pixels. |
| `slice_radius` | `Option<Size>` | Default `slice_radius`; `None` leaves the corners square. |
| `min_label_angle` | `f32` | Default `min_label_angle`, in degrees. |
| `radar_filled` | `bool` | Default `filled` of a `RadarChart`. |
| `radar_rings` | `usize` | Default `rings` of a `RadarChart`. |
| `zoom` | `bool` | Default `zoom`. |
| `legend_gap` | `&'static str` | Between the plot and the legend. |
| `swatch_size` | `&'static str` | A legend swatch's side. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-chart-aspect-ratio` | The plot's width over height. |
| `--lsx-chart-legend-gap` | Between the plot and the legend. |
| `--lsx-chart-swatch-size` | A legend swatch's side. |

## Data attributes

`data-state` carries `legend-top`, `legend-start`, `legend-end` or
`legend-bottom` while a legend shows.
