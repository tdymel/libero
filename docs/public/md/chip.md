# Chip

Crate: `libero`
Import: `use libero::components::Chip;`
Feature: `chip` (on by default, turns on `action-icon`, `anchor`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/chip.rs>
Index: [index.md](index.md) lists every other page
Description: A compact token: a tag, a filter, a small action or a link.

A compact token. With `onchange` or a `name` it is a checkbox, with `onclick` a
button, with `to` a link, and with none of them a plain tag. `variant` sets the
unselected look. A checked chip is always a tinted container, with a check before
its label. `icon` and `trailing` take a glyph on either side;
`color: "neutral", variant: "tonal"` makes the grey info pill.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Chip, Icon};
use pictogram_icons_lucide as lucide;

#[component]
fn Demo() -> Element {
    rsx! {
        Chip {
            icon: rsx! { Icon { variant: "standard", color: "inherit", svg: lucide::info::outlined } },
            trailing: rsx! { Icon { variant: "standard", color: "inherit", svg: lucide::chevron_down::outlined } },
            "rust"
        }
    }
}
```

An unsized `Icon` in `icon` or `trailing` takes the chip's icon step, so it
needs no `size`. The grey info pill:

```rust
use dioxus::prelude::*;
use libero::components::{Chip, Icon};
use pictogram_icons_lucide as lucide;

#[component]
fn Demo() -> Element {
    rsx! {
        Chip {
            color: "neutral",
            variant: "tonal",
            icon: rsx! { Icon { variant: "standard", color: "inherit", svg: lucide::clock::outlined } },
            "Updated 2 h ago"
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `color` | `ThemeAwareValue` | `primary` | Accent color. A theme color name or any CSS color; `theme.chip.color` when unset. |
| `variant` | `Variant` | `filled` | The unselected look. A checked chip is always a tonal container, with a check before its label. |
| `size` | `Size` | `md` | Height, padding and font size. |
| `radius` | `ThemeAwareValue` | `xl` | Corner radius, or any CSS, e.g. `radius: "0"`. |
| `checked` | `bool` | - | Whether it is selected. Pair it with `onchange`. Left out, a chip with a `name` keeps its own state, or the form's when that name binds it. A debug build warns when one comes without the other. |
| `disabled` | `bool` | `false` | Disables and dims the chip. |
| `readonly` | `bool` | `false` | A selectable chip stays focusable and posted with the form, but clicks and Space no longer toggle it. |
| `onchange` | `EventHandler<bool>` | - | Called with the value `checked` should take next. Makes the chip a checkbox. |
| `name` | `FieldName<bool>` | - | Makes the chip a checkbox that posts under this name. A path such as `Filters::FIELDS.open()` also binds it to the surrounding `Form`'s value when it has no `onchange`. |
| `value` | `String` | `on` | What the chip posts under its `name` when checked, so a row of filter chips can share one name. |
| `onclick` | `EventHandler<MouseEvent>` | - | A plain action. Makes the chip a `<button>`; with `checked` it is a toggle button and says `aria-pressed`. Not with `onchange`, which ignores it and `to`. |
| `to` | `NavigationTarget` | - | Makes the chip a router-aware link. Wins over `onclick`. |
| `target` | `String` | - | Link target, such as `_blank`. Only with `to`. `"_blank"` adds a hidden "(opens in a new tab)". |
| `new_tab_hint` | `bool` | `true` | `false` drops the hidden text a `"_blank"` target adds, and the icon. |
| `new_tab_icon` | `bool` | `false` | With the hint, draws an external icon after the label. Unset, the theme's `anchor.new_tab_icon`, which is off. |
| `icon` | `Element` | - | Drawn before the label, with a gap. It never shrinks. An `Icon` with no `size` takes the chip's icon step: `xs` up to an `md` chip, `sm` for `lg` and `xl`, `md` for `xxl`. |
| `trailing` | `Element` | - | Drawn after the label, with a gap, such as a remove button or a glyph. It never shrinks, and sizes an unsized `Icon` as `icon` does. Not a control on an `onclick` or `to` chip, which is a button already. |
| `children` | `Element` | required | The label, cut at the chip's edge. Text and `Icon` only. |

Like every component, `Chip` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `ChipPart::Icon` | `chip-icon` | The leading glyph's wrapper. |
| `ChipPart::Trailing` | `chip-trailing` | The slot after the label, with `trailing`. |
| `ChipPart::NewTab` | `new-tab` | The new-tab icon after the label, on a `to` chip with `target: "_blank"` and `new_tab_icon` only. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Space` | Toggles a selectable chip. |

### Libero handles

- A `readonly` chip keeps its tab stop and ignores the toggle.
- An `onclick` chip with `checked` sets `aria-pressed` from it; without
  `checked` it is a plain button.
- A selectable chip's `aria_label` goes on its checkbox, not on the root `span`.
- A chip in a list field (`MultiSelect`, `TagsField`) cut with an ellipsis shows
  its full text in a `title`. That is a hover reveal only: a keyboard or touch
  user does not get it, so keep labels short.

### You must

- Keep `children` to text and `Icon`: a selectable chip is a `<label>`, which
  takes the clicks of any control inside it.
- Name an icon-only chip: its checkbox takes the name of its label, so give the
  `Icon` an `aria_label`, or the chip an `aria_label`.

### Limits

- With `selected_check`, picking a chip widens it by the check, so a row of
  chips reflows; theme `selected_check: false` keeps its width.

### Example

A filter chip, `Chip { checked, onchange, "Vegan" }`: Tab lands on it, Space
toggles it, and a `readonly` chip keeps its tab stop but stays as it is.

## Theme defaults

`ChipDefaults` on the theme. Per-size values live in its `sizes` scale.

| Field | Type | Description |
|---|---|---|
| `variant` | `Variant` | Default `variant` when the prop is omitted (`filled`). |
| `size` | `Size` | Default `size` when the prop is omitted. |
| `radius` | `Size` | Default `radius` when the prop is omitted. |
| `selected_check` | `bool` | Draws a check before a selected chip's label, so the selection is not a colour change alone (`true`). |
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
| `--lsx-chip-selected` | Background of a chip while pressed. |
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
