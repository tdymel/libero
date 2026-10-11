# BarChart

Crate: `libero`, feature `charts`
Import: `use libero::components::{BarChart, Series};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/chart/bar_chart.rs>
Index: [index.md](index.md) lists every other page
Description: Values per category as bars, grouped, stacked or stacked to 100%, upright or lying, with a summary and the data as a table for screen readers.

Values per category as bars, side by side or stacked, drawn as an SVG that fills
its width. Takes the same `Series` as `LineChart`. Needs the `charts` feature.
Web and Android.

## Usage

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
scales every stack to 100% so each bar is its series' share; the table and the
summary keep the values. `orientation: "horizontal"` lays the categories down
the side, the first at the top. `value_labels: true` writes each bar's value on
it, inside the bar when stacked.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `x` | `XValues` | required | The categories, one per bar or group of bars. |
| `series` | `Vec<Series>` | required | The bars' series, each with one value per x value. `None` leaves no bar. |
| `title` | `Option<String>` | | Shown above the chart, and its accessible name. |
| `description` | `Option<String>` | | A sentence under the title: what to read from the chart. |
| `layout` | `BarLayout` | `grouped` | `grouped` side by side, `stacked`, or `percent`: stacked to 100%. |
| `orientation` | `Orientation` | `vertical` | `horizontal` lays the categories down the side, top to bottom. |
| `radius` | `Size` | `xs` | Corner radius of every bar, on the theme's radius scale. |
| `value_labels` | `Option<bool>` | `false` | Each bar's value written on it. |
| `legend` | `LegendPosition` | bottom with two series or more | `bottom`, `top`, `start`, `end` or `none`. |
| `grid` | `Option<bool>` | `true` | Guide lines across the values. |
| `value_format` | `ValueFormat` | `Auto` | How values are written on the axis, the bars, in the summary and in the table. |
| `unit` | `Option<String>` | | After every value, as `"kg"`. |
| `aspect_ratio` | `Option<f32>` | `16 / 9` | Width over height of the plot. |
| `loading` | `bool` | `false` | A placeholder instead of the plot, and `aria-busy` on the chart. |
| `empty` | `Option<Element>` | | Shown in the plot when there is no value at all. Unset, "No data". |
| `table` | `ChartTable` | `hidden` | The data as a table: `hidden` for screen readers, `visible`, or `off`. |
| `tooltip` | `Option<bool>` | | Shared with `LineChart`; no tooltip on bars yet. |
| `tooltip_content` | `Option<Callback<ChartPoint, Element>>` | | Shared with `LineChart`; not used on bars yet. |
| `onpointclick` | `Option<EventHandler<ChartPoint>>` | | Shared with `LineChart`; bars do not call it yet. |
| `onactivechange` | `Option<EventHandler<Option<ChartPoint>>>` | | Shared with `LineChart`; bars do not call it yet. |
| `hidden` | `Option<Vec<String>>` | | Shared with `LineChart`; the legend does not hide bars yet. |
| `default_hidden` | `Vec<String>` | `[]` | Shared with `LineChart`; not used on bars yet. |
| `onhiddenchange` | `Option<EventHandler<Vec<String>>>` | | Shared with `LineChart`; bars do not call it yet. |

Like every component, `BarChart` also takes the shared props `sx`, `class`,
`style`, `states`, `parts` (`ChartPart`), and any extra HTML attributes.

## Accessibility

### Libero handles

- The chart is a `role="figure"` named by `title`. The drawing is hidden from
  screen readers.
- A hidden summary gives the category range and each series' lowest and highest
  value, and where the highest is.
- A hidden table holds every value, up to the theme's `table_cell_cap` cells; a
  `percent` chart's table keeps the values, not the shares.
- Bars take the theme's palette, drawn in the current color scheme.

### You must

- Give a `title`, or name the chart with `aria_label`.
- Say in `description` what the reader should see, as "Q3 grew the most".
- Keep the legend, or `value_labels`, when series are told apart by colour
  alone.

### Limits

- No tooltip and no keyboard path over the bars yet: the summary and the table
  are the way to the values.
- The legend only names the series: the props `LineChart` uses to hide them and
  report points are taken but not wired yet.

## Theme defaults

`ChartDefaults` on the theme, `Theme::chart`, shared by every chart.

| Field | Type | Description |
|---|---|---|
| `aspect_ratio` | `f32` | Width over height of the plot. |
| `palette` | `[(Color, ColorShade); 8]` | Series colours in order; the ninth series starts over. |
| `legend` | `LegendPosition` | Where the legend sits with two series or more. |
| `grid` | `bool` | Default `grid`. |
| `bar_layout` | `BarLayout` | Default `layout`. |
| `bar_radius` | `Size` | Default `radius`. |
| `value_labels` | `bool` | Default `value_labels`. |
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
