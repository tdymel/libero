# Slider

Crate: `libero`
Import: `use libero::components::{Slider, SliderChangeEvent, SliderMark, SliderValue};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/inputs/slider>
Index: [index.md](index.md) - every other component's markdown page
Description: A value dragged along a track - continuous over `f64`, or discrete over an ordered enum that derives `SliderValue`.

A value dragged along a track. Controlled: it renders `value` and asks for a new
one through `on_change`. Pointer, touch and keyboard all drive it - the thumb is
a `role="slider"` with arrows, Page keys, Home and End.

What it slides over is a `SliderValue`: libero implements it for `f64` - a
continuous range - and an ordered enum of your own derives it. A type that lists
its options makes the slider discrete, and the range, step grid, marks and
captions all come from that list.

## Usage

A controlled continuous slider. `on_change` carries a `SliderChangeEvent`:
`Start` and `End` bracket one drag, `Change` carries every value in between.

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
                on_change: move |event: SliderChangeEvent| {
                    volume.set(event.value());
                    last.set(event)
                },
            }
            Text { size: "sm", "value: {volume()} - last event: {last():?}" }
        }
    }
}
```

## Discrete values

Deriving `SliderValue` on an ordered enum makes the slider discrete: the range,
the step grid and one mark per option all come from the option list, and
`#[slider(label = "..")]` renames a variant's caption. `min`, `max` and the
bounds are written in the value's own type; `step` is a count of options.

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
                on_change: move |event: SliderChangeEvent<Quality>| {
                    quality.set(event.value());
                    last.set(event)
                },
            }
            Text { size: "sm", "value: {quality():?} - last event: {last():?}" }
        }
    }
}
```

## Marks and labels

`marks` puts ticks on the track; a labeled one gets a caption below it. On a
discrete slider it *replaces* the one-per-option set the type derives; on a
continuous one there is nothing to derive, so it adds them. `label` formats the
bubble shown on hover, drag and keyboard focus, and becomes the thumb's
`aria-valuetext`.

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
            label: Callback::new(|value: f64| format!("{value}%")),
            on_change: move |event: SliderChangeEvent| volume.set(event.value()),
        }
    }
}
```

## Accessibility

The thumb, not the root, is the `role="slider"` element: it carries
`aria-orientation`, `aria-valuemin`, `aria-valuemax`, `aria-valuenow` and, when
`label` is set, `aria-valuetext` (a bare value is already in `aria-valuenow`).
Name it with the `aria_label` prop - an `aria_label` passed through
`attributes` would land on the root instead.

Keyboard: arrow keys move one `step` (the theme's `step` when no `step` prop is
set), Shift+arrow, PageUp and PageDown move `big_step` of them, Home and End
jump to `min` and `max`. Every key press calls `prevent_default`, so the page
does not scroll under the thumb. `disabled` sets `aria-disabled` and drops the
pointer and keyboard handlers.

`name` renders a hidden `<input>` alongside, so the value posts with a form.

## Props

`Slider`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `V` | required | Strictly controlled - pair it with `on_change`. |
| `min` | `V` | first option, or `0.0` | Lower bound, written in the value's own type. |
| `max` | `V` | last option, or `100.0` | Upper bound, written in the value's own type. |
| `step` | `V::Step` | - | Distance one step covers, measured from `min`: a count of options discretely, a value continuously. Also sets how many decimals an emitted value keeps. |
| `size` | `Size` | `md` | Controls track, thumb, and font size. |
| `radius` | `Size` | `xl` | Track corner radius; the thumb is always a circle. |
| `color` | `ThemeAwareValue` | `primary` | Accent color; a theme color name or a literal CSS color. |
| `disabled` | `bool` | `false` | Disables interaction and dims the slider. |
| `label` | `Callback<V, String>` | bare value, or `SliderValue::label` | Formats the bubble shown on hover, drag and keyboard focus, and sets the thumb's `aria-valuetext`. |
| `marks` | `Vec<SliderMark<V>>` | one per option, discretely | Ticks on the track; a labeled one gets a caption below it. Replaces the marks a discrete scale derives. |
| `aria_label` | `String` | - | Names the thumb, which is the `role="slider"` element - an `aria_label` in `attributes` would land on the root instead. |
| `name` | `String` | - | Emits a hidden input of that name, so the value posts with a form. |
| `on_change` | `EventHandler<SliderChangeEvent<V>>` | - | `Start`/`End` bracket a drag, `Change` carries every new value. |

Like every component, `Slider` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

`SliderMark`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `V` | required | Where the tick sits on the track. |
| `label` | `String` | - | Caption shown below the tick; omit for an unlabeled mark. |

## Theme defaults

`SliderDefaults` on the theme; per-size values live in its `sizes` scale.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted. |
| `radius` | `Size` | Default `radius` when the prop is omitted. |
| `sizes` | `Sizes<SliderSizeLevel>` | `track_size`, `thumb_size`, `font_size` per size. |
| `step` | `f64` | Steps moved per arrow key press. |
| `big_step` | `f64` | Steps moved per Shift+arrow, PageUp or PageDown. |

## CSS variables

The per-size scales are declared once; the picked level is resolved on the root
as `--lsx-slider-track`/`-thumb`/`-radius`, so the track, thumb and mark
children - which carry no `data-state` of their own - inherit it.

| Variable | Description |
|---|---|
| `--lsx-slider-track-size-<size>` | Track and filled-bar thickness for that size step. |
| `--lsx-slider-thumb-size-<size>` | Thumb diameter for that size step. |
| `--lsx-slider-font-size-<size>` | Label bubble and mark caption size for that size step. |
| `--lsx-slider-track` | The track thickness in effect, resolved on the root. |
| `--lsx-slider-thumb` | The thumb diameter in effect, resolved on the root. |
| `--lsx-slider-radius` | The track corner radius in effect, resolved on the root. |
| `--lsx-slider-color` | Accent color of the track fill and thumb. |
| `--lsx-slider-filled` | Filled fraction of the track, `0` to `1`. |
| `--lsx-slider-mark-at` | A mark's position along the track, `0` to `1`. |
| `--lsx-slider-mark-fill` | A mark's tick color; defaults to `grey.4` when it is past the value. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `size-<size>` | The `size` in effect. |
| `radius-<size>` | The `radius` in effect. |
| `dragging` | A pointer drag is in progress. |
| `disabled` | `disabled` is set. |
| `marks-labeled` | At least one mark carries a label, so the captions need room. |
