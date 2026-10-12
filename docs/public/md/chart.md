# Chart

Crate: `libero`, feature `charts`
Import: `use libero::components::{BarChart, LineChart, PieChart, Series, Slice};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/chart>
Index: [index.md](index.md) lists every other page
Description: Line, bar and pie charts as SVG, with a tooltip, arrow keys, a toggle legend, and a summary for screen readers.

`LineChart`, `BarChart` and `PieChart`, each drawn as an SVG that fills its
width, with a tooltip, the arrow keys, a legend that hides and shows, and a
summary for screen readers. Needs the `charts` feature. Web and Android.

A chart only draws. To show its values as a table too, put them in a
[Table](table.md).

Lines and bars take one or more `Series` over a shared x axis; a pie takes
`Slice`s, each a name and a value.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{LineChart, Series};

#[component]
fn Demo() -> Element {
    rsx! {
        LineChart {
            title: "Visitors",
            x: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
            series: vec![
                Series::new("This week", [120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0]),
                Series::new("Last week", [98.0, 110.0, 115.0, 120.0, 104.0, 170.0, 180.0]).dashed(),
            ],
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
| `legend` | `LegendPosition` | bottom with two series or more | `bottom`, `top`, `start`, `end` or `none`. |
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

### `BarChart`

| Prop | Type | Default | Description |
|---|---|---|---|
| `x` | `XValues` | required | The categories, one per bar or group of bars. |
| `series` | `Vec<Series>` | required | The bars' series, each with one value per x value. `None` leaves no bar. |
| `title` | `Option<String>` | | Shown above the chart, and its accessible name. |
| `description` | `Option<String>` | | A sentence under the title: what to read from the chart. |
| `layout` | `BarLayout` | `grouped` | `grouped` side by side, `stacked`, or `percent`: stacked to 100%. |
| `orientation` | `Orientation` | `vertical` | `horizontal` lays the categories down the side, top to bottom. |
| `radius` | `Size` | `xs` | Corner radius of a bar's free end, the one away from its axis, on the theme's radius scale. |
| `value_labels` | `Option<bool>` | `false` | Each bar's value written on it. |
| `legend` | `LegendPosition` | bottom with two series or more | `bottom`, `top`, `start`, `end` or `none`. |
| `grid` | `Option<bool>` | `true` | Guide lines across the values. |
| `value_format` | `ValueFormat` | `Auto` | How values are written on the axis, the bars, in the tooltip and in the summary. |
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

### `PieChart`

| Prop | Type | Default | Description |
|---|---|---|---|
| `slices` | `Vec<Slice>` | required | The slices, each a share of their sum. A negative value takes no share. |
| `title` | `Option<String>` | | Shown above the chart, and its accessible name. |
| `description` | `Option<String>` | | A sentence under the title: what to read from the chart. |
| `donut` | `bool` | `false` | A ring instead of a full pie, `center` in its middle. |
| `labels` | `PieLabels` | `percent` | Beside each slice: `percent`, `value`, `name` or `none`. |
| `center` | `Option<Element>` | | In a donut's hole, as a total, kept off the ring; its text shrinks in a small hole, the rest is clipped. |
| `start_angle` | `f32` | `0` | Where the first slice starts, in degrees clockwise from 12 o'clock. |
| `legend` | `LegendPosition` | bottom with two slices or more | `bottom`, `top`, `start`, `end` or `none`. |
| `value_format` | `ValueFormat` | `Auto` | How values are written beside the slices, in the tooltip and in the summary. |
| `unit` | `Option<String>` | | After every value, as `"kg"`. |
| `grid` | `Option<bool>` | | Shared with the other charts; a pie has no grid to draw. |
| `aspect_ratio` | `Option<f32>` | `16 / 9` | Width over height of the plot. |
| `loading` | `bool` | `false` | A placeholder instead of the plot, and `aria-busy` on the chart. |
| `empty` | `Option<Element>` | | Shown in the plot when the slices sum to nothing. Unset, "No data". |
| `tooltip` | `Option<bool>` | `true` | A tooltip with the slice's value and share at the pointer's or the keys' slice. |
| `tooltip_content` | `Option<Callback<ChartPoint, Element>>` | | The tooltip's content instead of the slice's name, value and share. |
| `onpointclick` | `Option<EventHandler<ChartPoint>>` | | A click on a slice, a second tap on a pinned one, or Enter on the keys' slice. `index` is the slice's place in `slices`, `x` and `series_name` its name. |
| `onactivechange` | `Option<EventHandler<Option<ChartPoint>>>` | | The slice under the pointer or the keys; `None` when it leaves. |
| `hidden` | `Option<Vec<String>>` | | The names of the slices the legend hides; set, they are controlled. |
| `default_hidden` | `Vec<String>` | `[]` | Seeds the hidden slices once. Ignored when `hidden` is set. |
| `onhiddenchange` | `Option<EventHandler<Vec<String>>>` | | The hidden slices a legend button asks for. The last shown slice stays. |

Like every component, each chart also takes the shared props `sx`, `class`,
`style`, `states`, `parts` (`ChartPart`), and any extra HTML attributes.

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` | Into the plot, then on to the legend's buttons. |
| `ArrowRight` / `ArrowLeft` | Line: the next or previous point of the line, as drawn, left to right in either text direction. Bar: the next or previous category's bar of the series; `horizontal`, the next or previous series' bar in the category. Pie: the next or previous slice, clockwise. |
| `ArrowDown` / `ArrowUp` | Line: the next or previous line at the same x value, or its nearest point. Bar: the next or previous series' bar in the category; `horizontal`, the next or previous category, top to bottom. Pie: the next or previous slice. |
| `Home` / `End` | The line's or the series' first or last point or bar; a pie's first or last slice. |
| `Enter` / `Space` | Calls `onpointclick` with the point, bar or slice. |
| `Escape` | Clears the point and its tooltip. |

### Libero handles

- The chart is a `role="figure"` named by `title`. The drawing is hidden from
  screen readers.
- A hidden summary gives the x range and each series' lowest and highest value,
  and where the highest is; a pie's, the total, then each slice's value and
  share.
- The plot is one tab stop, a `role="application"` described as a chart, so a
  screen reader hands it the arrows. Each point the keys reach is announced, as
  "This week, Tue: 132" or "Chrome: 62, 62%."
- The legend's entries are toggle buttons: pressed while their series or slice
  shows, struck through when hidden. The last shown one's button is
  `aria-disabled`. A dashed line's key is a dash, as drawn.
- On a touch screen a tap pins a point's tooltip, a second tap on it clicks; a
  vertical swipe still scrolls the page.
- Each series' or slice's palette colour reaches 3:1 on the page and on a card,
  in every shipped theme and color scheme; one too close to an earlier one, also
  for colour-blind eyes, is skipped.
- In forced colours the lines, bars and slices keep their colours on the system
  background; axes and labels take the system text colour.

### You must

- Give a `title`, or name the chart with `aria_label`.
- Say in `description` what the reader should see, as "Visitors peaked on
  Saturday".
- Tell series apart by more than colour: `dashed()` on a line, the legend,
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
- A hidden slice leaves the drawing, so the shown ones' shares grow; the
  summary keeps every slice.
- `grid` draws nothing on a pie.

## Theme defaults

`ChartDefaults` on the theme, `Theme::chart`, shared by every chart.

| Field | Type | Description |
|---|---|---|
| `aspect_ratio` | `f32` | Width over height of the plot. |
| `palette` | `[(Color, ColorShade); 8]` | Series and slice colours in order, each at least 3:1 on the page (a shade darker if not); one that reads like an earlier one is skipped; past the last they start over, a line dashed. |
| `curve` | `Curve` | Default `curve`. |
| `legend` | `LegendPosition` | Where the legend sits with two series or slices or more. |
| `grid` | `bool` | Default `grid`. |
| `points` | `bool` | Default `points`. |
| `points_limit` | `usize` | No dots past this many points. |
| `tooltip` | `bool` | Default `tooltip`. |
| `stroke_width` | `f32` | Line width, in pixels. |
| `bar_layout` | `BarLayout` | Default `layout`. |
| `bar_radius` | `Size` | Default `radius`. |
| `value_labels` | `bool` | Default `value_labels`. |
| `donut_thickness` | `f32` | A donut's ring width over the pie's radius. |
| `pie_labels` | `PieLabels` | Default `labels`. |
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
