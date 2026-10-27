# Chart

Crate: `libero`, feature `charts`
Import: `use libero::components::{BarChart, Candle, CandlestickChart, ChartHandle, ChartRange, FunnelChart, HeatmapChart, LineChart, PieChart, RadarAxis, RadarChart, ScatterChart, ScatterSeries, Series, Slice, Trend, use_chart};`
Feature: `charts` (off by default, turns on `skeleton`, `mark`, `focus-trap`, `use-accessibility`, `use-popover`, `use-debounce`, `use-id`, `use-icon`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/chart>
Index: [index.md](index.md) lists every other page
Description: Line, bar, pie, funnel, scatter, heatmap, candlestick and radar charts as SVG, with a tooltip, arrow keys, a toggle legend, and a summary for screen readers.

`LineChart`, `BarChart`, `PieChart`, `FunnelChart`, `ScatterChart`,
`HeatmapChart`, `CandlestickChart` and `RadarChart`, each drawn as an SVG that
fills its width, with a tooltip, the arrow keys, a legend that hides and shows,
and a summary for screen readers. Lines, bars and candlesticks zoom and pan with
`zoom`; every chart exports as SVG or PNG through a `use_chart()` handle. Needs
the `charts` feature. Web and Android.

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

## Props

### `LineChart`

| Prop | Type | Default | Description |
|---|---|---|---|
| `x` | `XValues` | required | The x axis, one value per point: labels, one evenly spaced slot each, or numbers, each point at its value, the axis ticking at round values written with `value_format`. `XValues::from_rows(&rows, |row| ..)` reads a list of records. |
| `series` | `Vec<Series>` | required | The lines, each with one value per x value. `None`, `NaN` and infinity are gaps. `.color(..)` (a theme color or hex) sets a colour, else the palette by place; `.unit(..)` gives its own unit in the tooltip and summary, while a unit every series shares labels the y axis; `.symbol(PointSymbol::Square)` draws points as that shape, a cue beside colour; `.errors([(low, high), ..])` draws a see-through band, which a stacked chart reads in the tooltip alone; `.mark_line(MarkLine::average())` draws a dashed line labelled on the chart, also `min()`, `max()`, `value(150.0)` and `.label(..)`; `.mark_point(MarkPoint::Max)` pins the highest value, `Min` the lowest; `.secondary_axis()` measures a line on its own scale at the right. |
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
| `value_format` | `ValueFormat` | `Auto` | How values are written on the axis, in the tooltip and in the summary. Thousands group as the `Formats` say, as `12,500` or `12.500`. |
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
| `handle` | `Option<ChartHandle>` | | From `use_chart()`: reads the drawing as SVG or PNG, saves it (through `save_file`: a download on the web, the share sheet on Android), resets the zoom and says whether it is zoomed. `is_zoomed()` re-runs the component that reads it. |

### `BarChart`

| Prop | Type | Default | Description |
|---|---|---|---|
| `x` | `XValues` | required | The categories, one per bar or group of bars. |
| `series` | `Vec<Series>` | required | The bars' series, each with one value per x value. `None` leaves no bar. `.colors([..])` colours bars one by one, `.negative_color(..)` those below 0, and `.errors([(low, high), ..])` draws error bars, written after the value in the tooltip. A stack rounds only its topmost positive and lowest negative segment. Upright, a `.kind(SeriesKind::Line)` one is drawn as a line over the bars. |
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
| `value_format` | `ValueFormat` | `Auto` | How values are written on the axis, the bars, in the tooltip and in the summary. On `percent` the axis and the bars write shares in percent. Thousands group as the `Formats` say, as `12,500` or `12.500`. |
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
| `handle` | `Option<ChartHandle>` | | From `use_chart()`: reads the drawing as SVG or PNG, saves it (through `save_file`: a download on the web, the share sheet on Android), resets the zoom and says whether it is zoomed. `is_zoomed()` re-runs the component that reads it. |

### `FunnelChart`

| Prop | Type | Default | Description |
|---|---|---|---|
| `stages` | `Vec<Slice>` | required | The stages, top to bottom, each as wide as its value over the largest. Drawn widest first: a stage larger than the one before it moves up, with a warning in development. A negative value has no width. |
| `title` | `Option<String>` | | Shown above the chart, and its accessible name. |
| `description` | `Option<String>` | | A sentence under the title: what to read from the chart. |
| `labels` | `PieLabels` | `percent` | On each stage: `percent`, its share of the first stage, `value`, `name` or `none`, in the ink that reads there. A label wider than its stage is left out; the tooltip and the summary keep it. |
| `legend` | `LegendPosition` | bottom with two stages or more | `bottom`, `top`, `start`, `end` or `none`. On a chart under 300 px wide, `start` goes above the plot and `end` below it. |
| `value_format` | `ValueFormat` | `Auto` | How values are written on the stages, in the tooltip and in the summary. Thousands group as the `Formats` say, as `12,500` or `12.500`. |
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
| `handle` | `Option<ChartHandle>` | | From `use_chart()`: reads the drawing as SVG or PNG, saves it (through `save_file`: a download on the web, the share sheet on Android). |

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
| `value_format` | `ValueFormat` | `Auto` | How values are written in the cells, on the scale, in the tooltip and in the summary. Thousands group as the `Formats` say, as `12,500` or `12.500`. |
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
| `handle` | `Option<ChartHandle>` | | From `use_chart()`: reads the drawing as SVG or PNG, saves it (through `save_file`: a download on the web, the share sheet on Android). |

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
| `value_format` | `ValueFormat` | `Auto` | How prices are written on the axis, in the tooltip and in the summary. Thousands group as the `Formats` say, as `12,500` or `12.500`. |
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
| `handle` | `Option<ChartHandle>` | | From `use_chart()`: reads the drawing as SVG or PNG, saves it (through `save_file`: a download on the web, the share sheet on Android), resets the zoom and says whether it is zoomed. `is_zoomed()` re-runs the component that reads it. |

### `PieChart`

| Prop | Type | Default | Description |
|---|---|---|---|
| `slices` | `Vec<Slice>` | required | The slices, each a share of their sum. A negative value takes no share. `Slice::from(("Chrome", 62.0))` builds one from a pair and `.color("success")` sets its colour, else the palette by place. |
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
| `value_format` | `ValueFormat` | `Auto` | How values are written beside the slices, in the tooltip and in the summary. Thousands group as the `Formats` say, as `12,500` or `12.500`. |
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
| `handle` | `Option<ChartHandle>` | | From `use_chart()`: reads the drawing as SVG or PNG, saves it (through `save_file`: a download on the web, the share sheet on Android). |

### `ScatterChart`

| Prop | Type | Default | Description |
|---|---|---|---|
| `series` | `Vec<ScatterSeries>` | required | The groups of points, each an x and a y, `ScatterSeries::new("Online", [(12.0, 34.0)])`. A third value, a size, makes the series bubbles, their area by it. A point with an x or y that is not a number is left out. `ScatterSeries::from_rows("Sales", &rows, |row| (row.price, row.sold))` builds one from your rows. |
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
| `value_format` | `ValueFormat` | `Auto` | How values are written on the axes, in the tooltip and in the summary. The axes tick at round values. Thousands group as the `Formats` say, as `12,500` or `12.500`. |
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
| `handle` | `Option<ChartHandle>` | | From `use_chart()`: reads the drawing as SVG or PNG, saves it (through `save_file`: a download on the web, the share sheet on Android). |

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
| `value_format` | `ValueFormat` | `Auto` | How values are written in the tooltip and in the summary. Thousands group as the `Formats` say, as `12,500` or `12.500`. |
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
| `handle` | `Option<ChartHandle>` | | From `use_chart()`: reads the drawing as SVG or PNG, saves it (through `save_file`: a download on the web, the share sheet on Android). |

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
  12 rows or 150 cells, only the empty cells the keys skip, then a pointer to
  the keys); a candlestick chart's,
  its periods, the first open and the last close with the change, the highest
  high and the lowest low with their periods, and how many candles rose, fell
  and stayed; a radar's, its axes, then each series' highest and lowest axis,
  by share of the axis maximum as drawn, and the axes it has no value on.
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
