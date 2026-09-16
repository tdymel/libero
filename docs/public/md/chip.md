# Chip

Crate: `libero`
Import: `use libero::components::Chip;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/chip.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A compact token - a tag, a filter, or a small inline action.

A compact token. With `onchange` it is a real checkbox - a visually hidden
`<input>` plus a `<label>` - so it gets checked semantics, Space-to-toggle and
focus for free. With `onclick` it is a `<button>`, with `to` a router-aware
link, and with none of them a plain `<span>`.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Chip;

#[component]
fn Demo() -> Element {
    rsx! {
        Chip { color: "primary", variant: "filled", size: "md", radius: "xl", "rust" }
    }
}
```

A selectable chip is strictly controlled: `checked` drives the look, `onchange` reports the value it
should take next. A checked chip is a tinted container - Material 3's selected
filter chip, which also drops the outline - whatever its `variant`, so `variant`
describes the unselected state. `checked` without `onchange` can never change,
and `onchange` without `checked` can never look selected - the library warns
about either alone.

```rust
use dioxus::prelude::*;
use libero::components::{Chip, Flex};

#[component]
fn Demo() -> Element {
    let mut selected = use_signal(|| vec!["rust".to_string()]);

    rsx! {
        Flex { direction: "row", gap: "md",
            for language in ["rust", "css", "html"] {
                Chip {
                    key: "{language}",
                    checked: selected().iter().any(|s| s == language),
                    onchange: move |next: bool| {
                        selected.with_mut(|selected| {
                            if next {
                                selected.push(language.to_string());
                            } else {
                                selected.retain(|s| s != language);
                            }
                        });
                    },
                    "{language}"
                }
            }
        }
    }
}
```

A `name` alone also makes the chip a checkbox: it now has something to post.
Give each chip of a filter row the same `name` and its own `value`, and the row
posts one entry per selected chip - `tags=rust&tags=css`. A chip with no
`value` posts `name=on`, as `Checkbox` does.

```rust
use dioxus::prelude::*;
use libero::components::{Chip, Flex};

#[component]
fn Demo() -> Element {
    rsx! {
        Flex { direction: "row", gap: "md",
            for language in ["rust", "css", "html"] {
                Chip { key: "{language}", name: "tags", value: "{language}", "{language}" }
            }
        }
    }
}
```

`onclick` makes the chip a `<button>`, `to` a router-aware link that takes
precedence over `onclick`. Neither combines with `onchange`.

```rust
use dioxus::prelude::*;
use libero::components::{Chip, Flex};

#[component]
fn Demo() -> Element {
    let mut clicks = use_signal(|| 0);

    rsx! {
        Flex { direction: "row", gap: "md",
            Chip { onclick: move |_| clicks += 1, "Clicked {clicks}x" }
            Chip { to: "https://dioxuslabs.com", target: "_blank", "Dioxus" }
        }
    }
}
```

An icon goes in `icon`: it sits before the label with a gap, and never
shrinks. The children are the chip's own flex items, never wrapped, so an
icon among them keeps the gap too. A long label is cut at the chip's edge; for
an ellipsis, give the text a span of its own, as below. A remove x goes in
`trailing`, after the label; a label that ellipsizes never clips it.

```rust
use dioxus::prelude::*;
use libero::components::Chip;

#[component]
fn Demo() -> Element {
    rsx! {
        Chip {
            icon: rsx! {
                svg { width: "14", height: "14", view_box: "0 0 24 24", fill: "currentColor",
                    circle { cx: "12", cy: "12", r: "6" }
                }
            },
            style: "max-width: 160px",
            span {
                style: "min-width: 0; overflow: hidden; text-overflow: ellipsis",
                "Versandkostenberechnungsgrundlage"
            }
        }
    }
}
```

## Accessibility

Space toggles a selectable chip. Keep `children` to text and `Icon` - a
`<label>` hijacks clicks on any nested control.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `color` | `ThemeAwareValue` | `primary` | Accent color; a theme color name or a literal CSS color. |
| `variant` | `Variant` | `filled` | The unselected look; a checked chip is a tonal container whatever its variant. |
| `size` | `Size` | `md` | Controls height, padding, and font size. |
| `radius` | `Size` | `xl` | Corner radius, independent of `size`. |
| `checked` | `bool` | - | Selection state - pair it with `onchange`. Left out, a chip with a `name` keeps its own state unless that name binds it to the form around it. |
| `disabled` | `bool` | `false` | Disables interaction and dims the chip. |
| `onchange` | `EventHandler<bool>` | - | Called with the value `checked` should take next. Its presence makes the chip a real checkbox. |
| `name` | `FieldName<bool>` | - | Makes the chip a checkbox that posts under this name. A path - `Filters::FIELDS.open()` - also binds it to the surrounding `Form`'s value when the chip has no `onchange`, as on `Checkbox`. |
| `value` | `String` | `on` | What the chip posts under its `name` when it is checked, so a row of filter chips can share one name. Left out, it posts the browser's `on`, as `Checkbox` does. |
| `onclick` | `EventHandler<MouseEvent>` | - | A plain action; its presence makes the chip a `<button>`. |
| `to` | `NavigationTarget` | - | Renders a router-aware link instead. Takes precedence over `onclick`. |
| `target` | `String` | - | Link target, e.g. `_blank`. Only with `to`. |
| `icon` | `Element` | - | Drawn before the label, with a gap; it never shrinks. |
| `trailing` | `Element` | - | Drawn after the label, with a gap; it never shrinks - a remove x. Outside a checkbox chip's `<label>`, so it may be a button, but not on an `onclick` or `to` chip. |
| `children` | `Element` | required | The label, laid out as the chip's own flex items and cut at its edge. Text and `Icon` only - a `<label>` hijacks clicks on nested controls. |

Like every component, `Chip` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`ChipDefaults` on the theme; per-size values live in its `sizes` scale.

| Field | Type | Description |
|---|---|---|
| `variant` | `Variant` | Default `variant` when the prop is omitted (`filled`). |
| `size` | `Size` | Default `size` when the prop is omitted. |
| `radius` | `Size` | Default `radius` when the prop is omitted. |
| `sizes` | `Sizes<ChipSizeLevel>` | `font_size`, `height`, `padding_x` per size. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-chip-font-size-<size>` | `font-size` for that size step. |
| `--lsx-chip-height-<size>` | `height` for that size step. |
| `--lsx-chip-padding-x-<size>` | Horizontal padding for that size step. |
| `--lsx-chip-color` | Accent color of the current variant. |
| `--lsx-chip-contrast` | Text color on top of that accent. |
| `--lsx-chip-hover` | Accent color while hovered. |
| `--lsx-chip-container` | Container fill of `tonal`. |
| `--lsx-chip-on-container` | Label color on that container - black or white, whichever reads on it. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `filled` / `tonal` / `elevated` / `outlined` / `standard` | The `variant` in effect. |
| `size-<size>` | The `size` in effect. |
| `radius-<size>` | The `radius` in effect. |
| `checked` | `checked` is set. |
| `disabled` | `disabled` is set. |
| `clickable` | The chip is a button or link, not a checkbox. |
| `selectable` | The chip is a checkbox (`onchange` or `name`). Its root does not clip, so the focus ring drawn out by its border shows; its label clips instead. Every other chip clips at its root. |
