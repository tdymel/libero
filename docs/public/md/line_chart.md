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
`.color("success.7")` or `.color("#1e88e5")` sets a series' colour; otherwise it
takes the theme's palette for its place.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `x` | `XValues` | required | The x axis, one value per point: labels or numbers. |
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

Like every component, `LineChart` also takes the shared props `sx`, `class`,
`style`, `states`, `parts` (`ChartPart`), and any extra HTML attributes.

## Accessibility

### Libero handles

- The chart is a `role="figure"` named by `title`. The drawing is hidden from
  screen readers.
- A hidden summary gives the x range and each line's lowest and highest value,
  and where the highest is.
- A hidden table holds every value, up to the theme's `table_cell_cap` cells.
- Lines take the theme's palette, drawn in the current color scheme.

### You must

- Give a `title`, or name the chart with `aria_label`.
- Say in `description` what the reader should see, as "Visitors peaked on
  Saturday".
- Tell lines apart by more than colour: `dashed()` on a series, or their names
  in the legend.

### Limits

- No tooltip and no keyboard path over the points yet: the summary and the
  table are the way to the values.

## Theme defaults

`ChartDefaults` on the theme, `Theme::chart`.

| Field | Type | Description |
|---|---|---|
| `aspect_ratio` | `f32` | Width over height of the plot. |
| `palette` | `[(Color, ColorShade); 8]` | Series colours in order; the ninth series starts over. |
| `curve` | `Curve` | Default `curve`. |
| `legend` | `LegendPosition` | Where the legend sits with two series or more. |
| `grid` | `bool` | Default `grid`. |
| `points` | `bool` | Default `points`. |
| `points_limit` | `usize` | No dots past this many points. |
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
