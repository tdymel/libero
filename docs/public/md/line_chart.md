# LineChart

Crate: `libero`, feature `charts`
Import: `use libero::components::{LineChart, Series};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/chart/line_chart.rs>
Index: [index.md](index.md) lists every other page
Description: Series of values over a shared x axis as lines, with a summary and the data as a table for screen readers.

One or more `Series` over a shared x axis, drawn as an SVG that fills its
width. Needs the `charts` feature. Web and Android.

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
in the tooltip, summary and table; a unit every series shares labels the y axis.
Values group their thousands as the `Formats` say: `12,500` or `12.500`.

The pointer or the arrow keys pick a point: a tooltip shows the x value and
each line's value there. `onpointclick` reports a clicked point, and the
legend's buttons hide and show lines.

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

## Props

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
| `value_format` | `ValueFormat` | `Auto` | How values are written on the axis, in the summary and in the table. |
| `unit` | `Option<String>` | | After every value, as `"kg"`. |
| `aspect_ratio` | `Option<f32>` | `16 / 9` | Width over height of the plot. |
| `loading` | `bool` | `false` | A placeholder instead of the plot, and `aria-busy` on the chart. |
| `empty` | `Option<Element>` | | Shown in the plot when there is no value at all. Unset, "No data". |
| `table` | `ChartTable` | `hidden` | The data as a table: `hidden` for screen readers, `visible`, or `off`. |
| `tooltip` | `Option<bool>` | `true` | A tooltip and a crosshair at the pointer's or the keys' point. |
| `tooltip_content` | `Option<Callback<ChartPoint, Element>>` | | The tooltip's content instead of the x value and each line's value there. |
| `onpointclick` | `Option<EventHandler<ChartPoint>>` | | A click on a point, a second tap on a pinned one, or Enter on the keys' point. |
| `onactivechange` | `Option<EventHandler<Option<ChartPoint>>>` | | The point under the pointer or the keys; `None` when it leaves. |
| `hidden` | `Option<Vec<String>>` | | The names of the lines the legend hides; set, they are controlled. |
| `default_hidden` | `Vec<String>` | `[]` | Seeds the hidden lines once. Ignored when `hidden` is set. |
| `onhiddenchange` | `Option<EventHandler<Vec<String>>>` | | The hidden lines a legend button asks for. The last shown line stays. |

Like every component, `LineChart` also takes the shared props `sx`, `class`,
`style`, `states`, `parts` (`ChartPart`), and any extra HTML attributes.

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` | Into the plot, then on to the legend's buttons. |
| `ArrowRight` / `ArrowLeft` | The next or previous point of the line, as drawn: left to right in either text direction. |
| `ArrowDown` / `ArrowUp` | The next or previous line at the same x value, or its nearest point. |
| `Home` / `End` | The line's first or last point. |
| `Enter` / `Space` | Calls `onpointclick` with the point. |
| `Escape` | Clears the point and its tooltip. |

### Libero handles

- The chart is a `role="figure"` named by `title`. The drawing is hidden from
  screen readers.
- A hidden summary gives the x range and each line's lowest and highest value,
  and where the highest is.
- A hidden table holds every value, up to the theme's `table_cell_cap` cells.
- The plot is one tab stop, a `role="application"` described as a chart, so a
  screen reader hands it the arrows. Each point the keys reach is announced, as
  "This week, Tue: 132".
- The legend's entries are toggle buttons: pressed while their line shows,
  struck through when hidden. The last shown line's button is `aria-disabled`.
- On a touch screen a tap pins a point's tooltip, a second tap on it clicks; a
  vertical swipe still scrolls the page.
- Each line's palette colour reaches 3:1 on the page and on a card, in every
  shipped theme and color scheme; one too close to an earlier one, also for
  colour-blind eyes, is skipped.
- In forced colours the lines keep their colours on the system background; axes
  and labels take the system text colour.

### You must

- Give a `title`, or name the chart with `aria_label`.
- Say in `description` what the reader should see, as "Visitors peaked on
  Saturday".
- Tell lines apart by more than colour: `dashed()` on a series, or their names
  in the legend.
- Name every series differently: the legend hides them by name.

### Limits

- A hex colour (`.color("#1e88e5")`) is drawn as given: check its 3:1 on your
  page.
- The tooltip is for the eye; screen readers hear the announcement instead, so
  `tooltip_content` reaches only sighted readers.
- The x axis runs left to right in a right-to-left page too.

## Theme defaults

`ChartDefaults` on the theme, `Theme::chart`.

| Field | Type | Description |
|---|---|---|
| `aspect_ratio` | `f32` | Width over height of the plot. |
| `palette` | `[(Color, ColorShade); 8]` | Series colours in order, each at least 3:1 on the page (a shade darker if not); one that reads like an earlier one is skipped; past the last they start over, a line dashed. |
| `curve` | `Curve` | Default `curve`. |
| `legend` | `LegendPosition` | Where the legend sits with two series or more. |
| `grid` | `bool` | Default `grid`. |
| `points` | `bool` | Default `points`. |
| `points_limit` | `usize` | No dots past this many points. |
| `tooltip` | `bool` | Default `tooltip`. |
| `stroke_width` | `f32` | Line width, in pixels. |
| `table_cell_cap` | `usize` | The most cells the data table holds. |
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
`legend-bottom` while a legend shows, and `table-visible`.
