# PieChart

Crate: `libero`, feature `charts`
Import: `use libero::components::{PieChart, Slice};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/chart/pie_chart.rs>
Index: [index.md](index.md) lists every other page
Description: Parts of a whole as slices of a pie or a donut, with a summary and the data as a table for screen readers.

Parts of a whole as slices of a circle, or of a ring. Each `Slice` is a name and
a value. Needs the `charts` feature. Web and Android.

## Usage

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

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `slices` | `Vec<Slice>` | required | The slices, each a share of their sum. A negative value takes no share. |
| `title` | `Option<String>` | | Shown above the chart, and its accessible name. |
| `description` | `Option<String>` | | A sentence under the title: what to read from the chart. |
| `donut` | `bool` | `false` | A ring instead of a full pie, `center` in its middle. |
| `labels` | `PieLabels` | `percent` | Beside each slice: `percent`, `value`, `name` or `none`. |
| `center` | `Option<Element>` | | In a donut's middle, as a total. |
| `start_angle` | `f32` | `0` | Where the first slice starts, in degrees clockwise from 12 o'clock. |
| `legend` | `LegendPosition` | bottom with two slices or more | `bottom`, `top`, `start`, `end` or `none`. |
| `value_format` | `ValueFormat` | `Auto` | How values are written beside the slices, in the summary and in the table. |
| `unit` | `Option<String>` | | After every value, as `"kg"`. |
| `grid` | `Option<bool>` | | Shared with the other charts; a pie has no grid to draw. |
| `aspect_ratio` | `Option<f32>` | `16 / 9` | Width over height of the plot. |
| `loading` | `bool` | `false` | A placeholder instead of the plot, and `aria-busy` on the chart. |
| `empty` | `Option<Element>` | | Shown in the plot when the slices sum to nothing. Unset, "No data". |
| `table` | `ChartTable` | `hidden` | The data as a table: `hidden` for screen readers, `visible`, or `off`. |
| `tooltip` | `Option<bool>` | | Shared with `LineChart`; no tooltip on slices yet. |
| `tooltip_content` | `Option<Callback<ChartPoint, Element>>` | | Shared with `LineChart`; not used on slices yet. |
| `onpointclick` | `Option<EventHandler<ChartPoint>>` | | Shared with `LineChart`; slices do not call it yet. |
| `onactivechange` | `Option<EventHandler<Option<ChartPoint>>>` | | Shared with `LineChart`; slices do not call it yet. |
| `hidden` | `Option<Vec<String>>` | | Shared with `LineChart`; the legend does not hide slices yet. |
| `default_hidden` | `Vec<String>` | `[]` | Shared with `LineChart`; not used on slices yet. |
| `onhiddenchange` | `Option<EventHandler<Vec<String>>>` | | Shared with `LineChart`; slices do not call it yet. |

Like every component, `PieChart` also takes the shared props `sx`, `class`,
`style`, `states`, `parts` (`ChartPart`), and any extra HTML attributes.

## Accessibility

### Libero handles

- The chart is a `role="figure"` named by `title`. The drawing is hidden from
  screen readers.
- A hidden summary gives the total, then each slice's value and share.
- A hidden table holds each slice's value and share.
- Slices take the theme's palette, drawn in the current color scheme.

### You must

- Give a `title`, or name the chart with `aria_label`.
- Keep `labels` or the legend: colour alone does not tell slices apart.
- Keep to a few slices; fold the small ones into one "Other".

### Limits

- No tooltip and no keyboard path over the slices yet: the summary and the
  table are the way to the values.
- The legend only names the slices: the props `LineChart` uses to hide series
  and report points are taken but not wired yet.
- `grid` draws nothing on a pie.

## Theme defaults

`ChartDefaults` on the theme, `Theme::chart`, shared by every chart.

| Field | Type | Description |
|---|---|---|
| `aspect_ratio` | `f32` | Width over height of the plot. |
| `palette` | `[(Color, ColorShade); 8]` | Slice colours in order; the ninth slice starts over. |
| `legend` | `LegendPosition` | Where the legend sits with two slices or more. |
| `donut_thickness` | `f32` | A donut's ring width over the pie's radius. |
| `pie_labels` | `PieLabels` | Default `labels`. |
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
