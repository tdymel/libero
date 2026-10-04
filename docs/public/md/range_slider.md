# RangeSlider

Crate: `libero`
Import: `use libero::components::{RangeSlider, SliderChangeEvent, SliderMark, SliderValue};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/slider>
Index: [index.md](index.md) lists every other page
Description: Two thumbs on one track, for a span instead of a point, over the same values as `Slider`.

Two thumbs on one track, for a span instead of a point. It takes the same
values as [Slider](slider.md), as a pair in track order. The thumbs never cross,
and each stops at the other or `min_range` short of it.

## Usage

`oninput` carries a `SliderChangeEvent<(V, V)>`. `Start` and `End` bracket one
drag, and `Change` carries every pair in between. A key press sends `Change`,
then `End`, so saving on `End` catches keyboard edits too.

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
                aria_label: "Price",
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

`min_range` is the smallest gap the thumbs keep. The dragged thumb stops that far
short of the other instead of pushing it. It uses the unit of `step`, and at the
default 0 the thumbs may meet. The gap rounds up to whole steps, so
`step: 10, min_range: 3` keeps them a full step apart.

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

An ordered enum that derives `SliderValue` makes the range discrete, as on a
`Slider`. `min` and `max` are written as variants, and `step` and `min_range`
count options.

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

`marks` and `format` work as on a `Slider`, and one `format` covers both
bubbles. The bar fills between the thumbs.

`name` posts the pair as two hidden inputs under one name, in track order.

```rust
use dioxus::prelude::*;
use libero::components::{RangeSlider, SliderChangeEvent};

#[component]
fn Demo() -> Element {
    let mut price = use_signal(|| (20.0, 80.0));

    rsx! {
        form {
            RangeSlider {
                label: "Price",
                name: "price",
                value: price(),
                oninput: move |event: SliderChangeEvent<(f64, f64)>| price.set(event.value()),
            }
        }
    }
}
```

```js
form.getAll("price") // ["20", "80"]
```

## Props

### `RangeSlider`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Track, thumb and font size. |
| `color` | `ThemeAwareValue` | `primary` | Accent color. A theme color name or any CSS color. |
| `value` | `Option<(V, V)>` | - | The two ends, in track order. Pair it with `oninput`, or bind it with a path `name` inside a `Form`. |
| `oninput` | `EventHandler<SliderChangeEvent<(V, V)>>` | - | Fires per value while dragging. `Start` and `End` bracket a drag, `Change` carries each new pair. A key press sends `Change`, then `End`. |
| `min` | `V` | `first option, or 0.0` | Lower bound of the track, in the value's own type. |
| `max` | `V` | `last option, or 100.0` | Upper bound of the track, in the value's own type. Off the `step` grid, the track ends at the last step below it, as on a native range input: 0 to 100 by 30 ends at 90. |
| `step` | `V::Step` | - | How far one step goes from `min`. A count of options on a discrete scale, a value on a continuous one. Also sets how many decimals a value keeps. |
| `min_range` | `V::Step` | `0` | The smallest gap the thumbs keep, in the unit of `step`. At 0 they may meet, and they never cross. |
| `format` | `Callback<V, String>` | `bare value, or SliderValue::label` | Text of the bubbles and each thumb's `aria-valuetext`. On a discrete scale it also names the marks, so this is where a translation goes. |
| `marks` | `Vec<SliderMark<V>>` | `one per option, discretely` | Ticks on the track. A labeled one gets a caption below it. Replaces the marks a discrete scale draws itself. Past about six options the derived captions touch on a phone, so pass your own `marks`, or a `step` that skips options. |
| `aria_label` | `String` | - | Names the pair when the field has no `label`: the group, and each thumb before its own word. Put in `attributes`, it would land on the wrapper instead. |
| `aria_label_from` | `String` | `slider.minimum` | Names the lower thumb. Unset, the localization's `slider.minimum`, "Minimum" in English. |
| `aria_label_to` | `String` | `slider.maximum` | Names the upper thumb. Unset, the localization's `slider.maximum`, "Maximum" in English. |
| `name` | `FieldName<(V, V)>` | - | Posts the pair as two hidden inputs of that name, in track order. A path such as `Settings::FIELDS.price()` also binds the pair to the surrounding `Form`'s value when there is no `oninput`. |
| `validate` | `Validators<(V, V)>` | - | Rules over the pair, shown once the slider loses focus or its form is submitted. |
| `label` | `Caption` | - | The caption above the track. Both thumbs' names start with it. |
| `description` | `Caption` | - | Between the label and the track. What the range means. |
| `helper` | `Caption` | - | Under the track, below the mark captions. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error, an empty one `Valid`. |
| `required` | `bool` | `false` | Adds an asterisk to the label. No `aria-required`: ARIA does not allow it on a slider, which always holds a value. |
| `disabled` | `bool` | `false` | Takes the thumbs out of the tab order and dims the slider. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` drops the slider from the tab order and the post instead. |

### `SliderMark`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `V` | - | Where the tick sits on the track. |
| `label` | `String` | - | Caption below the tick. Leave it out for a bare tick. |

Like every component, `RangeSlider` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. The attributes land on the
field wrapper.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `SliderPart::Label` | `label` | The label above the control. |
| `SliderPart::Required` | `required` | The required asterisk, in the label. |
| `SliderPart::Description` | `description` | The caption between the label and the control. |
| `SliderPart::Control` | `control` | The slider under the label: the track and the room around it. |
| `SliderPart::Track` | `track` | The rail the thumbs run along. |
| `SliderPart::Bar` | `bar` | The filled stretch of the track. |
| `SliderPart::Mark` | `mark` | One tick on the track. |
| `SliderPart::MarkLabel` | `mark-label` | A tick's caption. |
| `SliderPart::Thumb` | `thumb` | The handle; a range has two. |
| `SliderPart::Helper` | `helper` | The caption under the control. |
| `SliderPart::Status` | `status` | The validation message. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Left`, `Right`, `Up` or `Down` | Move the focused thumb `theme.slider.step` steps, one by default. Right to left, ArrowLeft raises the value instead. |
| `Shift+Arrow`, `PageUp` or `PageDown` | Move the focused thumb `theme.slider.big_step` steps, ten by default. |
| `Home` or `End` | Move the focused thumb to the end, stopping at the other thumb. |

### Libero handles

- The two thumbs sit in a `role="group"` named by the label, and each is its own
  `role="slider"`, as in the ARIA multi-thumb slider pattern. A single `Slider`
  is one slider and needs no group.
- Each thumb is named by the label plus its own word, such as "Price Minimum"
  and "Price Maximum", from the localization's `slider.minimum` and
  `slider.maximum`. Without a `label`, `aria_label` names the group and the
  thumbs the same way.
- On a discrete range the mark captions are hidden from screen readers, since
  the thumbs already name each value.
- A `role="slider"` takes no `aria-required`, so a `required` range says the
  localization's `slider.required` word in each thumb's name instead, such as
  "Price required Minimum". The asterisk stays hidden from screen readers.

### You must

- Without a `label`, set the `aria_label` prop, or the thumbs say only
  "Minimum" and "Maximum". Put in `attributes`, it would name a wrapper with no
  role.
- Set `aria_label_from` and `aria_label_to` when those words do not fit.
- Set `format` when a bare number does not say the unit. To translate a
  discrete range, pass `format`, as on a `Slider`.

### Example

A price range, `RangeSlider { label: "Price" }`: the thumbs read as "Price
Minimum" and "Price Maximum". The arrows move the focused thumb one step,
PageUp ten, and Home or End to the end, stopping at the other thumb.

## Theme defaults

`theme.slider`, the same `SliderDefaults` a `Slider` reads. See
[Slider](slider.md#theme-defaults).

## CSS variables

Every variable a `Slider` sets, plus the two the second thumb needs.

| Variable | Description |
|---|---|
| `--lsx-slider-filled-from` | Where the filled bar starts, `0` to `1`, and `0` on a single-thumb slider. |
| `--lsx-slider-filled-span` | How much of the track the bar covers, `0` to `1`. |
| `--lsx-slider-thumb-at` | One thumb's own position along the track, `0` to `1`, set on its anchor. |

## Data attributes

The same tokens a `Slider` carries: `size-*`, `dragging`, `disabled`,
`readonly` and `marks-labeled`.
