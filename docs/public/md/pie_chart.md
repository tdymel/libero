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

The pointer or the arrow keys pick a slice: a tooltip shows its value and share.
`onpointclick` reports a clicked slice, and the legend's buttons hide and show
slices.

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
| `tooltip` | `Option<bool>` | `true` | A tooltip with the slice's value and share at the pointer's or the keys' slice. |
| `tooltip_content` | `Option<Callback<ChartPoint, Element>>` | | The tooltip's content instead of the slice's name, value and share. |
| `onpointclick` | `Option<EventHandler<ChartPoint>>` | | A click on a slice, a second tap on a pinned one, or Enter on the keys' slice. `index` is the slice's place in `slices`, `x` and `series_name` its name. |
| `onactivechange` | `Option<EventHandler<Option<ChartPoint>>>` | | The slice under the pointer or the keys; `None` when it leaves. |
| `hidden` | `Option<Vec<String>>` | | The names of the slices the legend hides; set, they are controlled. |
| `default_hidden` | `Vec<String>` | `[]` | Seeds the hidden slices once. Ignored when `hidden` is set. |
| `onhiddenchange` | `Option<EventHandler<Vec<String>>>` | | The hidden slices a legend button asks for. The last shown slice stays. |

Like every component, `PieChart` also takes the shared props `sx`, `class`,
`style`, `states`, `parts` (`ChartPart`), and any extra HTML attributes.

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` | Into the plot, then on to the legend's buttons. |
| `ArrowRight` / `ArrowDown` | The next slice, clockwise. |
| `ArrowLeft` / `ArrowUp` | The previous slice. |
| `Home` / `End` | The first or last slice. |
| `Enter` / `Space` | Calls `onpointclick` with the slice. |
| `Escape` | Clears the slice and its tooltip. |

### Libero handles

- The chart is a `role="figure"` named by `title`. The drawing is hidden from
  screen readers.
- A hidden summary gives the total, then each slice's value and share.
- A hidden table holds each slice's value and share.
- The plot is one tab stop, a `role="application"` described as a chart, so a
  screen reader hands it the arrows. Each slice the keys reach is announced, as
  "Chrome: 62, 62%."
- The legend's entries are toggle buttons: pressed while their slice shows,
  struck through when hidden. The last shown slice's button is `aria-disabled`.
- On a touch screen a tap pins a slice's tooltip, a second tap on it clicks; a
  vertical swipe still scrolls the page.
- Each slice's palette colour reaches 3:1 on the page and on a card, in every
  shipped theme and color scheme; one too close to an earlier one, also for
  colour-blind eyes, is skipped.
- In forced colours the slices keep their colours on the system background; axes
  and labels take the system text colour.

### You must

- Give a `title`, or name the chart with `aria_label`.
- Keep `labels` or the legend: colour alone does not tell slices apart.
- Keep to a few slices; fold the small ones into one "Other".
- Name every slice differently: the legend hides them by name.

### Limits

- A hex colour (`.color("#1e88e5")`) is drawn as given: check its 3:1 on your
  page.
- A hidden slice leaves the drawing, so the shown ones' shares grow; the
  summary and the table keep every slice.
- The tooltip is for the eye; screen readers hear the announcement instead, so
  `tooltip_content` reaches only sighted readers.
- `grid` draws nothing on a pie.

## Theme defaults

`ChartDefaults` on the theme, `Theme::chart`, shared by every chart.

| Field | Type | Description |
|---|---|---|
| `aspect_ratio` | `f32` | Width over height of the plot. |
| `palette` | `[(Color, ColorShade); 8]` | Slice colours in order, each at least 3:1 on the page (a shade darker if not); one that reads like an earlier one is skipped; past the last they start over. |
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
