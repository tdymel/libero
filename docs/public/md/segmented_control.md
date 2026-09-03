# SegmentedControl

Crate: `libero`
Import: `use libero::components::{Options, SegmentedControl};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/segmented_control>
Index: [index.md](index.md) - every other component's markdown page
Description: A connected strip of segments over an enum, exactly one of them selected, wearing the field slots.

A connected strip of segments over an enum, exactly one of them selected. The
segments are the enum's variants - `#[derive(Options)]` lists them in
declaration order - so a misspelled segment is a compile error rather than a
selection that never matches. Strictly controlled: `value` drives the look,
`onchange` reports the segment that should become selected.

It is a field like [RadioGroup](radio_group.md): a label, captions and a status
around the strip, a `name` that binds it to a [Form](form.md), and rules
through `validate`.

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
it at the type, and the `option_label` prop renames it during render - which is what a
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
            option_label: |alignment: Alignment| OptionLabel::rich(
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

`options` narrows or reorders the strip. `String` implements `Options` and
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
            options: options.clone(),
            onchange: move |next| selected.set(next),
        }
    }
}
```

## Disabling a segment

`disabled_options` takes the segments that render but cannot be picked. It is a
`Vec<T>`, not a flag per segment, because the control owns the whole strip.
`disabled` disables every segment, like on any other field.

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
            disabled_options: vec![Alignment::Center],
        }
    }
}
```

## In a form

With a label and captions it is laid out like every other field. Bound to a
[Form](form.md) through a path, it needs neither `value` nor `onchange`: the
selection is read from the form's value and written back into it.

```rust
use dioxus::prelude::*;
use libero::components::{Fields, Form, Options, SegmentedControl};

#[derive(Clone, Copy, PartialEq, Default, Options)]
enum Density {
    Compact,
    #[default]
    Cosy,
    Roomy,
}

#[derive(Clone, PartialEq, Default, Fields)]
struct Layout {
    density: Density,
}

#[component]
fn Demo() -> Element {
    let layout = use_store(Layout::default);

    rsx! {
        Form {
            value: layout,
            SegmentedControl {
                name: Layout::FIELDS.density(),
                label: "Density",
                helper: "Applies to every table.",
            }
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
with Rust. The `label` names the group through `aria-labelledby`, and the
description, helper and status describe it. Without a visible label, name the
control with an `aria_label` where its purpose is not obvious from the segments
themselves.

## Props

### SegmentedControl

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `T` | - | Strictly controlled - pair it with `onchange`. Exactly one segment is selected, which is what makes this a radio group and not a row of toggles. A control bound to a `Form` through `name` leaves it out. |
| `onchange` | `EventHandler<T>` | - | Called with the segment that should become selected. |
| `name` | `FieldName<T>` | - | What the control posts as. A path - `Settings::FIELDS.align()` - also binds it to the surrounding `Form`'s value when it has no `onchange`. |
| `validate` | `Validators<T>` | - | Rules over the selection, shown once the control loses focus or its form is submitted. |
| `options` | `Vec<T>` | `T::options()` | Narrows or reorders the strip. A runtime set of `String`s passes them here, since `String` lists no options of its own. |
| `option_label` | `Callback<T, OptionLabel>` | `T::label()` | Overrides what the derive named a segment. Runs during render, so it can read a locale from context. |
| `disabled_options` | `Vec<T>` | - | Segments that render but cannot be picked. |
| `orientation` | `Orientation` | `horizontal` | Row or column layout. |
| `variant` | `ButtonVariant` | `filled` | The unselected look, shared by every segment. |
| `color` | `ThemeAwareValue` | `primary` | Accent color; a theme color name or a literal CSS color. |
| `size` | `Size` | `md` | Shared by every segment, and by the captions around them. |
| `radius` | `Size` | `md` | Corner radius of the control's outer corners; inner ones are square. |
| `gap` | `Size` | - | Space between the segments. Set it and they stop sharing borders - each keeps its own, and its own radius. |
| `full_width` | `bool` | `false` | Segments share the width evenly instead of sizing to their label. |
| `focusable` | `bool` | `true` | `false` keeps the segments out of the tab order, and a click leaves focus where it is - for a control inside a field's dropdown. |
| `label` | `Caption` | - | The question. Names the group through `aria-labelledby`, since `for` cannot name a `role="radiogroup"`. |
| `description` | `Caption` | - | Between the label and the segments: how to choose. |
| `helper` | `Caption` | - | Under the segments. Consequences of the choice. |
| `status` | `FieldStatus` | `Valid` | Validation state, rendered under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Adds `aria-required` to the group and marks the label. |
| `disabled` | `bool` | `false` | Disables every segment and dims the captions. |

### OptionLabel

| Field | Type | Default | Description |
|---|---|---|---|
| `name` | `String` | required | The segment's accessible name, and its text when there is no `content`. |
| `content` | `Element` | - | Drawn in place of the name, via `OptionLabel::rich` - an icon or a badge. `name` still names the segment, since the rsx is what a screen reader cannot use. |

`OptionLabel` is a value, not a component - it takes no shared props. A bare
string is one (`"Konto".into()`), and it is shared with [Tabs](tabs.md).

Like every component, `SegmentedControl` also takes the shared props `sx`,
`class`, `style`, `states`, and any extra HTML attributes. `sx`, `class` and
`states` land on the field's wrapper; the attributes land on the
`radiogroup`.

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

The field's wrapper carries the usual field tokens - size, radius, status,
`disabled` and `required`. State tokens on the `radiogroup`'s `data-state`,
space separated:

| Token | Condition |
|---|---|
| `horizontal` / `vertical` | The `orientation` in effect. |
| `filled` / `tonal` / `elevated` / `outlined` / `standard` | The `variant` in effect. |
| `full-width` | `full_width` is set. |
| `collapsed` | No `gap` - the segments share borders and square off their inner corners. |
| `size-<size>` | The `gap` step in effect, when `gap` is set. |

Each segment `<label>` carries its own `data-state`.

| Token | Condition |
|---|---|
| `size-<size>` / `radius-<size>` | The control's `size` and `radius`. |
| `checked` | The segment is the selected one. |
| `disabled` | The segment is in `disabled_options`, or the whole control is `disabled`. |
