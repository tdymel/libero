# Chip

Crate: `libero`
Import: `use libero::components::Chip;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/chip.rs>
Index: [index.md](index.md) lists every other page
Description: A compact token: a tag, a filter, a small action or a link.

A compact token. With `onchange` or a `name` it is a checkbox, with `onclick` a
button, with `to` a link, and with none of them a plain tag. `variant` sets the
unselected look. A checked chip is always a tinted container.

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

For a selectable chip, pass `checked` with `onchange`. A debug build warns when
one comes without the other.

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

A `name` alone also makes the chip a checkbox. Give each chip of a filter row
the same `name` and its own `value`, and the row posts one entry per selected
chip, such as `tags=rust&tags=css`. A chip with no `value` posts `name=on`.

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

`onclick` makes the chip a button, `to` a router-aware link that wins over
`onclick`. Neither combines with `onchange`.

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

An icon goes in `icon`, before the label, and a remove button in `trailing`,
after it. Neither shrinks. A long label is cut at the chip's edge. For an
ellipsis, give the text a span of its own:

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

Space toggles a selectable chip. A `readonly` one keeps its tab stop and
ignores the toggle. Keep `children` to text and `Icon`, because a selectable
chip is a `<label>`, which takes the clicks of any control inside it.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `color` | `ThemeAwareValue` | `primary` | Accent color. A theme color name or any CSS color. |
| `variant` | `Variant` | `filled` | The unselected look. A checked chip is always a tonal container. |
| `size` | `Size` | `md` | Height, padding and font size. |
| `radius` | `Size` | `xl` | Corner radius. |
| `checked` | `bool` | - | Whether it is selected. Pair it with `onchange`. Left out, a chip with a `name` keeps its own state, or the form's when that name binds it. |
| `disabled` | `bool` | `false` | Disables and dims the chip. |
| `readonly` | `bool` | `false` | A selectable chip stays focusable and posted with the form, but clicks and Space no longer toggle it. |
| `onchange` | `EventHandler<bool>` | - | Called with the value `checked` should take next. Makes the chip a checkbox. |
| `name` | `FieldName<bool>` | - | Makes the chip a checkbox that posts under this name. A path such as `Filters::FIELDS.open()` also binds it to the surrounding `Form`'s value when it has no `onchange`. |
| `value` | `String` | `on` | What the chip posts under its `name` when checked, so a row of filter chips can share one name. |
| `onclick` | `EventHandler<MouseEvent>` | - | A plain action. Makes the chip a `<button>`. |
| `to` | `NavigationTarget` | - | Makes the chip a router-aware link. Wins over `onclick`. |
| `target` | `String` | - | Link target, such as `_blank`. Only with `to`. `"_blank"` adds an external icon and a hidden "(opens in a new tab)". |
| `new_tab_hint` | `bool` | `true` | `false` drops the icon and the hidden text a `"_blank"` target adds. |
| `icon` | `Element` | - | Drawn before the label, with a gap. It never shrinks. |
| `trailing` | `Element` | - | Drawn after the label, with a gap, such as a remove button. It never shrinks. Not on an `onclick` or `to` chip, which is a button already. |
| `children` | `Element` | required | The label, cut at the chip's edge. Text and `Icon` only. |

Like every component, `Chip` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`ChipDefaults` on the theme. Per-size values live in its `sizes` scale.

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
| `--lsx-chip-on-container` | Label color on that container, black or white, whichever reads on it. |

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
| `selectable` | The chip is a checkbox (`onchange` or `name`). |
