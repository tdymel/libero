# Chip

Crate: `libero`
Import: `use libero::components::Chip;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/inputs/chip>
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
| `checked` | `bool` | - | Strictly controlled selection state - pair it with `onchange`. |
| `disabled` | `bool` | `false` | Disables interaction and dims the chip. |
| `onchange` | `EventHandler<bool>` | - | Called with the value `checked` should take next. Its presence makes the chip a real checkbox. |
| `onclick` | `EventHandler<MouseEvent>` | - | A plain action; its presence makes the chip a `<button>`. |
| `to` | `NavigationTarget` | - | Renders a router-aware link instead. Takes precedence over `onclick`. |
| `target` | `String` | - | Link target, e.g. `_blank`. Only with `to`. |
| `children` | `Element` | required | Text and `Icon` only - a `<label>` hijacks clicks on nested controls. |

Like every component, `Chip` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`ChipDefaults` on the theme; per-size values live in its `sizes` scale.

| Field | Type | Description |
|---|---|---|
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
