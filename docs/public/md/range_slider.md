# RangeSlider

Crate: `libero`
Import: `use libero::components::{RangeSlider, SliderChangeEvent, SliderMark, SliderValue};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/slider>
Index: [index.md](index.md) - every other component's markdown page
Description: Two thumbs on one track for a span rather than a point - the `Slider` engine, over a pair of values.

Two thumbs on one track, for a span rather than a point. Controlled: it renders
`value` - a pair, in track order - and asks for a new one through `oninput`.

The same engine and the same value types as [Slider](slider.md): continuous over
`f64`, discrete over an ordered enum that derives `SliderValue`. The thumbs
never cross - each stops at the other, or `min_range` short of it.

## Usage

`oninput` carries a `SliderChangeEvent<(V, V)>`: `Start` and `End` bracket one
drag, `Change` carries every pair in between. A key press emits `Change` then
`End`, so committing on `End` catches a keyboard edit too.

```rust
use dioxus::prelude::*;
use libero::components::{Flex, RangeSlider, SliderChangeEvent, Text};
use libero::sx::sx;

#[component]
fn Demo() -> Element {
    let mut price = use_signal(|| (20.0, 80.0));
    let mut last = use_signal(|| SliderChangeEvent::Change((20.0, 80.0)));

    rsx! {
        Flex { direction: "column", gap: "sm", sx: sx().width("100%"),
            RangeSlider {
                label: "Price",
                value: price(),
                oninput: move |event: SliderChangeEvent<(f64, f64)>| {
                    price.set(event.value());
                    last.set(event)
                },
            }
            Text { size: "sm", "value: {price():?} - last event: {last():?}" }
        }
    }
}
```

## Keeping the thumbs apart

`min_range` is the smallest gap the two keep: the dragged thumb stops that far
short of the other rather than pushing it along. It is written in the same unit
as `step` - a distance continuously, a count of options discretely - and
defaults to zero, which lets the thumbs meet. The track's own `min` and `max`
win over the gap, so a range too narrow to hold one never puts a thumb off the
track.

```rust,ignore
RangeSlider {
    label: "Price",
    value: price(),
    min: 0.0,
    max: 200.0,
    step: 5.0,
    min_range: 25.0,
    oninput: move |event: SliderChangeEvent<(f64, f64)>| price.set(event.value()),
}
```

## Discrete values

Deriving `SliderValue` on an ordered enum makes the range discrete, exactly as
it does a `Slider`: the track, the step grid and one mark per option come from
the option list, `min`/`max` are written in the value's own type, and `step` and
`min_range` are counts of options.

```rust
use dioxus::prelude::*;
use libero::components::{RangeSlider, SliderChangeEvent, SliderValue};

#[derive(Clone, Copy, Debug, PartialEq, SliderValue)]
enum Quality {
    Low,
    Medium,
    High,
    #[slider(label = "Max")]
    Ultra,
}

#[component]
fn Demo() -> Element {
    let mut quality = use_signal(|| (Quality::Low, Quality::High));

    rsx! {
        RangeSlider {
            label: "Quality",
            value: quality(),
            min_range: 1usize,
            oninput: move |event: SliderChangeEvent<(Quality, Quality)>| {
                quality.set(event.value())
            },
        }
    }
}
```

## Marks and labels

`marks` and `format` behave as they do on a `Slider`: ticks with optional
captions, and one formatter for both bubbles. A mark reads as filled while it
lies *between* the thumbs, which is also where the bar is drawn - a range fills
from the lower thumb rather than from the track's start.

## Posting with a form

`name` renders two hidden inputs under that one name, in track order, so the
pair posts natively and reads back as a pair:

```rust,ignore
RangeSlider { name: "price", value: price(), oninput: move |e| price.set(e.value()) }
```

```js
form.getAll("price") // ["20", "80"]
```

## Accessibility

The focused thumb is the one the keys move. Arrows move one `step`,
Shift+arrow, PageUp and PageDown move `big_step` of them, and Home and End jump
that thumb to `min` or `max` - stopping at its neighbour like any other move.

A `label` names both thumbs, so each follows it with its own name: "Price
Minimum" and "Price Maximum", with "Minimum" and "Maximum" by default. Pass `aria_label_from` and `aria_label_to` where those
words do not fit, and `format` where a bare number does not say the unit.

## Props

`RangeSlider`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `(V, V)` | required | The two ends, in track order. Strictly controlled - pair it with `oninput`. |
| `min` | `V` | first option, or `0.0` | Lower bound of the track, written in the value's own type. |
| `max` | `V` | last option, or `100.0` | Upper bound of the track, written in the value's own type. |
| `step` | `V::Step` | - | Distance one step covers, measured from `min`: a count of options discretely, a value continuously. Also sets how many decimals an emitted value keeps. |
| `min_range` | `V::Step` | `0` - the thumbs may meet | The smallest gap the two thumbs keep. Neither can cross the other. |
| `size` | `Size` | `md` | Controls track, thumb, and font size. |
| `color` | `ThemeAwareValue` | `primary` | Accent color; a theme color name or a literal CSS color. |
| `disabled` | `bool` | `false` | Disables interaction and dims the slider. |
| `format` | `Callback<V, String>` | bare value, or `SliderValue::label` | Formats the bubble shown on hover, drag and keyboard focus, and sets each thumb's `aria-valuetext`. |
| `marks` | `Vec<SliderMark<V>>` | one per option, discretely | Ticks on the track; a labeled one gets a caption below it. Replaces the marks a discrete scale derives. |
| `aria_label_from` | `String` | `Minimum` | Names the lower thumb, which the field's label cannot tell apart from the upper one. |
| `aria_label_to` | `String` | `Maximum` | Names the upper thumb. |
| `label` | `Caption` | - | The field's caption, above the track. Named by `aria-labelledby`, since `for` cannot name a thumb. |
| `description` | `Caption` | - | Between the label and the track: what the range means. |
| `helper` | `Caption` | - | Under the track, below the mark captions. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Adds `aria-required` to both thumbs and marks the label. |
| `name` | `String` | - | Emits two hidden inputs of that name, in track order, so the pair posts with a form. |
| `oninput` | `EventHandler<SliderChangeEvent<(V, V)>>` | - | Fires per value - a drag is the DOM's `input` event. `Start`/`End` bracket a drag, `Change` carries every new pair. A key press emits `Change` then `End`. |

Like every component, `RangeSlider` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

`SliderMark`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `V` | required | Where the tick sits on the track. |
| `label` | `String` | - | Caption shown below the tick; omit for an unlabeled mark. |

## Theme defaults

`SliderDefaults` on the theme - the same one a `Slider` reads, including `step`
and `big_step`. See [Slider](slider.md#theme-defaults).

## CSS variables

Every variable a `Slider` sets, plus the two the second thumb needs.

| Variable | Description |
|---|---|
| `--lsx-slider-filled-from` | Where the filled bar starts, `0` to `1`; `0` on a single-thumb slider. |
| `--lsx-slider-filled-span` | How much of the track the bar covers, `0` to `1`. |
| `--lsx-slider-thumb-at` | One thumb's own position along the track, `0` to `1`, set on its anchor. |

## Data attributes

The same tokens a `Slider` carries - `size-*`, `dragging`, `disabled` and
`marks-labeled`.
