# SegmentedControl

Crate: `libero`
Import: `use libero::components::{Options, SegmentedControl};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/inputs/segmented_control>
Index: [index.md](index.md) - every other component's markdown page
Description: A connected strip of segments over an enum, exactly one of them selected.

A connected strip of segments over an enum, exactly one of them selected. The
segments are the enum's variants - `#[derive(Options)]` lists them in
declaration order - so a misspelled segment is a compile error rather than a
selection that never matches. Strictly controlled: `value` drives the look,
`onchange` reports the segment that should become selected.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Options, SegmentedControl};

#[derive(Clone, Copy, PartialEq, Options)]
enum Alignment {
    Left,
    Center,
    Right,
}

#[component]
fn Demo() -> Element {
    let mut alignment = use_signal(|| Alignment::Left);

    rsx! {
        SegmentedControl {
            value: alignment(),
            onchange: move |next| alignment.set(next),
        }
    }
}
```

Nobody writes the list of segments, so nobody can write one that disagrees with
`value`. Setting `gap` stops the segments sharing borders - each keeps its own
border and its own radius.

## Renaming a segment

The derive names a segment after its variant. `#[option(label = "..")]` renames
it at the type, and the `label` prop renames it during render - which is what a
translated strip needs, since it repaints when the locale signal it reads does.

```rust
use dioxus::prelude::*;
use libero::components::{Options, SegmentedControl};

#[derive(Clone, Copy, PartialEq, Options)]
enum Density {
    Compact,
    Cosy,
    #[option(label = "Roomy")]
    Comfortable,
}

#[component]
fn Demo() -> Element {
    let mut density = use_signal(|| Density::Cosy);

    rsx! {
        SegmentedControl {
            value: density(),
            onchange: move |next| density.set(next),
        }
    }
}
```

## Drawing a segment

`OptionLabel::rich` draws a segment as rsx and names it separately. A segment is
a radio, so the name is what a screen reader reads - the rsx it cannot use. A
segment is a `<label>`, so its content has to stay phrasing content: an
[Icon](icon.md) is an inline-flex `span` (and it is what sizes the raw svg), a
[Flex](flex.md) is a `div`.

```rust
use dioxus::prelude::*;
use libero::components::{Icon, OptionLabel, Options, SegmentedControl};

#[derive(Clone, Copy, PartialEq, Options)]
enum Alignment {
    Left,
    Center,
    Right,
}

#[component]
fn Demo() -> Element {
    let mut alignment = use_signal(|| Alignment::Center);

    rsx! {
        SegmentedControl {
            value: alignment(),
            onchange: move |next| alignment.set(next),
            label: |alignment: Alignment| OptionLabel::rich(
                alignment.label(),
                rsx! {
                    Icon {
                        variant: "transparent",
                        size: "sm",
                        match alignment {
                            Alignment::Left => rsx! { AlignLeftIcon {} },
                            Alignment::Center => rsx! { AlignCenterIcon {} },
                            Alignment::Right => rsx! { AlignRightIcon {} },
                        }
                    }
                    "{alignment.label()}"
                },
            ),
        }
    }
}
```

The `Align*Icon`s there are your own icon components - any `svg` will do; `Icon`
is what sizes and colors it. The segment separates the icon from the text
itself, so no `sx` is needed for the gap.

## A runtime set of segments

`segments` narrows or reorders the strip. `String` implements `Options` and
lists nothing of its own, so a set that is data rather than a type passes the
segments here - the same shape as `f64: SliderValue` for a continuous
[Slider](slider.md).

```rust
use dioxus::prelude::*;
use libero::components::SegmentedControl;

#[component]
fn Demo(options: Vec<String>) -> Element {
    let mut selected = use_signal(|| options[0].clone());

    rsx! {
        SegmentedControl {
            value: selected(),
            segments: options.clone(),
            onchange: move |next| selected.set(next),
        }
    }
}
```

## Disabling a segment

`disabled` takes the segments that render but cannot be picked. It is a
`Vec<T>`, not a flag per segment, because the control owns the whole strip.

```rust
use dioxus::prelude::*;
use libero::components::{Options, SegmentedControl};

#[derive(Clone, Copy, PartialEq, Options)]
enum Alignment {
    Left,
    Center,
    Right,
}

#[component]
fn Demo() -> Element {
    let mut alignment = use_signal(|| Alignment::Left);

    rsx! {
        SegmentedControl {
            value: alignment(),
            onchange: move |next| alignment.set(next),
            disabled: vec![Alignment::Center],
        }
    }
}
```

## Accessibility

The root is a `role="radiogroup"` and every segment is a `<label>` around a
visually hidden `<input type="radio">`. That is what a segmented control is:
exactly one of a set, never none and never two - so a screen reader announces
"1 of 3", and arrow keys move the selection while Tab enters and leaves the
whole control. All of it is the browser's own, so there is no roving tabindex to
maintain.

The radio's click is cancelled and the selection written from `value` instead,
so the DOM property, `:checked` and the accessibility tree can never disagree
with Rust. Name the control with an `aria_label` where its purpose is not
obvious from the segments themselves.

```rust
use dioxus::prelude::*;
use libero::components::{Options, SegmentedControl};

#[derive(Clone, Copy, PartialEq, Options)]
enum Alignment {
    Left,
    Center,
    Right,
}

#[component]
fn Demo() -> Element {
    let mut alignment = use_signal(|| Alignment::Left);

    rsx! {
        SegmentedControl {
            aria_label: "Text alignment",
            value: alignment(),
            onchange: move |next| alignment.set(next),
        }
    }
}
```

For a row of *independent* toggles - bold, italic, underline - this is the wrong
component: those are separate booleans, not one selection. Use a
[Flex](flex.md) of [Button](button.md)s with `selected` set.

## Props

### SegmentedControl

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `T` | required | Strictly controlled - pair it with `onchange`. Exactly one segment is selected, which is what makes this a radio group and not a row of toggles. |
| `onchange` | `EventHandler<T>` | - | Called with the segment that should become selected. |
| `segments` | `Vec<T>` | `T::options()` | Narrows or reorders the strip. A runtime set of `String`s passes them here, since `String` lists no options of its own. |
| `label` | `Callback<T, OptionLabel>` | `T::label()` | Overrides what the derive named a segment. Runs during render, so it can read a locale from context. |
| `disabled` | `Vec<T>` | - | Segments that render but cannot be picked. |
| `orientation` | `Orientation` | `horizontal` | Row or column layout. |
| `variant` | `ButtonVariant` | `outlined` | The unselected look, shared by every segment. |
| `color` | `ThemeAwareValue` | `primary` | Accent color; a theme color name or a literal CSS color. |
| `size` | `Size` | `md` | Shared by every segment. |
| `radius` | `Size` | `md` | Corner radius of the control's outer corners; inner ones are square. |
| `gap` | `Size` | - | Space between the segments. Set it and they stop sharing borders - each keeps its own, and its own radius. |
| `full_width` | `bool` | `false` | Segments share the width evenly instead of sizing to their label. |

### OptionLabel

| Field | Type | Default | Description |
|---|---|---|---|
| `name` | `String` | required | The segment's accessible name, and its text when there is no `content`. |
| `content` | `Element` | - | Drawn in place of the name, via `OptionLabel::rich` - an icon or a badge. `name` still names the segment, since the rsx is what a screen reader cannot use. |

`OptionLabel` is a value, not a component - it takes no shared props. A bare
string is one (`"Konto".into()`), and it is shared with [Tabs](tabs.md).

Like every component, `SegmentedControl` also takes the shared props `sx`,
`class`, `style`, `states`, and any extra HTML attributes.

## Theme defaults

None of its own. Every visual prop is the segments', so the control reads
[Button](button.md)'s `ButtonDefaults` for `size` and `radius`; `gap` resolves
against the theme's spacing scale.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-button-color` | The accent, resolved from `color`. Set on the root; the segments inherit it. |
| `--lsx-button-contrast` | Label color on a `filled` segment. |
| `--lsx-button-hover` | Hover background. |
| `--lsx-button-selected` | Background of the selected segment. |
| `--lsx-spacing-<size>` | Read for `gap` when it is set. |
| `--lsx-spacing-xs` | The gap inside a segment, between a rich label's icon and its text. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `horizontal` / `vertical` | The `orientation` in effect. |
| `filled` / `outlined` / `text` | The `variant` in effect. |
| `full-width` | `full_width` is set. |
| `collapsed` | No `gap` - the segments share borders and square off their inner corners. |
| `size-<size>` | The `gap` step in effect, when `gap` is set. |

Each segment `<label>` carries its own `data-state`.

| Token | Condition |
|---|---|
| `size-<size>` / `radius-<size>` | The control's `size` and `radius`. |
| `checked` | The segment is the selected one. |
| `disabled` | The segment is in `disabled`. |
