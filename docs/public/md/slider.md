# Slider

Crate: `libero`
Import: `use libero::components::{Slider, SliderChangeEvent, SliderMark, SliderValue};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/slider>
Index: [index.md](index.md) lists every other page
Description: A value dragged along a track, continuous over `f64` or discrete over an ordered enum that derives `SliderValue`.

A value you drag along a track. It slides over any `SliderValue`. An `f64`
gives a continuous range. An ordered enum that derives it makes the slider
discrete, and the range, steps, marks and captions come from its options.

## Usage

`oninput` carries a `SliderChangeEvent`. `Start` and `End` bracket one drag,
and `Change` carries every value in between. A key press sends `Change`, then
`End`, so saving on `End` catches keyboard edits too.

```rust
use dioxus::prelude::*;
use libero::components::{Flex, Slider, SliderChangeEvent, Text};
use libero::sx::sx;

#[component]
fn Demo() -> Element {
    let mut volume = use_signal(|| 40.0);
    let mut last = use_signal(|| SliderChangeEvent::Change(40.0));

    rsx! {
        Flex { direction: "column", gap: "sm", sx: sx().width("100%"),
            Slider {
                aria_label: "Volume",
                value: volume(),
                oninput: move |event: SliderChangeEvent| {
                    volume.set(event.value());
                    last.set(event)
                },
            }
            Text { size: "sm", "value: {volume()} - last event: {last():?}" }
        }
    }
}
```

Derive `SliderValue` on an ordered enum for a discrete slider. The range, the
steps and one mark per option come from the variants, and
`#[slider(label = "..")]` renames one. `min` and `max` are written as variants,
and `step` counts options.

```rust
use dioxus::prelude::*;
use libero::components::{Flex, Slider, SliderChangeEvent, SliderValue, Text};
use libero::sx::sx;

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
    let mut quality = use_signal(|| Quality::Medium);
    let mut last = use_signal(|| SliderChangeEvent::Change(Quality::Medium));

    rsx! {
        Flex { direction: "column", gap: "sm", sx: sx().width("100%"),
            Slider {
                aria_label: "Quality",
                value: quality(),
                min: Quality::Low,
                max: Quality::Ultra,
                oninput: move |event: SliderChangeEvent<Quality>| {
                    quality.set(event.value());
                    last.set(event)
                },
            }
            Text { size: "sm", "value: {quality():?} - last event: {last():?}" }
        }
    }
}
```

`marks` puts ticks on the track, and a labeled one gets a caption below it. On
a discrete slider they replace the one mark per option. `format` sets the text
of the bubble and the thumb's `aria-valuetext`.

```rust
use dioxus::prelude::*;
use libero::components::{Slider, SliderChangeEvent, SliderMark};

#[component]
fn Demo() -> Element {
    let mut volume = use_signal(|| 40.0);

    rsx! {
        Slider {
            aria_label: "Volume",
            value: volume(),
            marks: vec![
                SliderMark::labeled(0.0, "0"),
                SliderMark::new(50.0),
                SliderMark::labeled(100.0, "100"),
            ],
            format: Callback::new(|value: f64| format!("{value}%")),
            oninput: move |event: SliderChangeEvent| volume.set(event.value()),
        }
    }
}
```

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Left`, `Right`, `Up` or `Down` | Move one step. With `step: 0.0` a step is 1% of the range. |
| `Shift+Arrow`, `PageUp` or `PageDown` | Move `big_step` steps. |
| `Home` or `End` | Jump to the ends. |

### Libero handles

- `format` replaces `SliderValue::label` in the bubble, the captions and
  `aria-valuetext`, and runs during render, so it can read the locale from
  context.
- On a discrete slider the mark captions are hidden from screen readers, since
  the thumb already names each value. On a continuous one they stay, since they
  can say more than the number.

### You must

- Without a `label`, set the `aria_label` prop. Put in `attributes`, it would
  name the wrapper instead of the thumb.
- Pass `format` when a bare number does not say the unit, and to translate a
  discrete slider.

## Props

### `Slider`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Track, thumb and font size. |
| `color` | `ThemeAwareValue` | `primary` | Accent color. A theme color name or any CSS color. |
| `value` | `Option<V>` | - | The value. Pair it with `oninput`, or bind it with a path `name` inside a `Form`. |
| `oninput` | `EventHandler<SliderChangeEvent<V>>` | - | Fires per value while dragging. `Start` and `End` bracket a drag, `Change` carries each new value. A key press sends `Change`, then `End`. |
| `min` | `V` | `first option, or 0.0` | Lower bound, in the value's own type. |
| `max` | `V` | `last option, or 100.0` | Upper bound, in the value's own type. |
| `step` | `V::Step` | - | How far one step goes from `min`. A count of options on a discrete scale, a value on a continuous one. Also sets how many decimals a value keeps. |
| `format` | `Callback<V, String>` | `bare value, or SliderValue::label` | Text of the bubble and the thumb's `aria-valuetext`. On a discrete scale it also names the marks, so this is where a translation goes. |
| `marks` | `Vec<SliderMark<V>>` | `one per option, discretely` | Ticks on the track. A labeled one gets a caption below it. Replaces the marks a discrete scale draws itself. |
| `aria_label` | `String` | - | Names the thumb when the field has no `label`. Put in `attributes`, it would land on the wrapper instead. |
| `name` | `FieldName<V>` | - | Posts the value in a hidden input of that name. A path such as `Settings::FIELDS.volume()` also binds the value to the surrounding `Form`'s value when there is no `oninput`. |
| `validate` | `Validators<V>` | - | Rules over the value, shown once the slider loses focus or its form is submitted. |
| `label` | `Caption` | - | The caption above the track, and the thumb's name. |
| `description` | `Caption` | - | Between the label and the track. What the value means. |
| `helper` | `Caption` | - | Under the track, below the mark captions. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Sets `aria-required` on the thumb and marks the label. |
| `disabled` | `bool` | `false` | Takes the thumb out of the tab order and dims the slider. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` drops the slider from the tab order and the post instead. |

### `SliderMark`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `V` | - | Where the tick sits on the track. |
| `label` | `String` | - | Caption below the tick. Leave it out for a bare tick. |

`SliderMark::new(value)` makes a bare tick, `SliderMark::labeled(value, label)`
one with a caption.

Like every component, `Slider` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. The attributes land on the
field wrapper.

## Theme defaults

`theme.slider` is a `SliderDefaults`. Per-size values live in its `sizes` scale.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted, `md`. |
| `sizes` | `Sizes<SliderSizeLevel>` | `track_size`, `thumb_size`, `font_size` per size. |
| `step` | `f64` | Steps moved per arrow key press. |
| `big_step` | `f64` | Steps moved per Shift+arrow, PageUp or PageDown. |

## CSS variables

The size in effect resolves on the root as `--lsx-slider-track` and
`--lsx-slider-thumb`, and the track, thumb and marks read those.

The track is always a pill, so a slider has no `radius`.

| Variable | Description |
|---|---|
| `--lsx-slider-track-size-<size>` | Track and filled-bar thickness for that size step. |
| `--lsx-slider-thumb-size-<size>` | Thumb diameter for that size step. |
| `--lsx-slider-font-size-<size>` | Label bubble and mark caption size for that size step. |
| `--lsx-slider-track` | The track thickness in effect, resolved on the root. |
| `--lsx-slider-thumb` | The thumb diameter in effect, resolved on the root. |
| `--lsx-slider-color` | Accent color of the track fill and thumb. |
| `--lsx-slider-filled` | Filled fraction of the track, `0` to `1`. |
| `--lsx-slider-mark-at` | A mark's position along the track, `0` to `1`. |

## Data attributes

The field wrapper carries `size-*`, `radius-*`, and `disabled`, `required` and
the status token when they apply. The slider's own root carries the tokens
below, space separated.

| Token | Condition |
|---|---|
| `size-<size>` | The `size` in effect. |
| `dragging` | A pointer drag is in progress. |
| `disabled` | `disabled` is set. |
| `readonly` | `readonly` is set. The thumb keeps its tab stop, but neither a key nor a drag moves it. |
| `marks-labeled` | At least one mark carries a label, so the captions need room. |

The unfilled track is `muted.6`, 3:1 on the page (WCAG 1.4.11). A mark is an
open `surface` dot ringed in `muted.6`. A mark on the filled bar carries
`data-state="filled"` and drops the ring, so it stays visible on the bar.
