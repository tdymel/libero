# Burger

Crate: `libero`
Import: `use libero::components::Burger;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/burger.rs>
Index: [index.md](index.md) lists every other page
Description: Three bars that morph into an X, an `ActionIcon` with the ARIA a nav toggle needs.

Three bars that morph into an X. It renders an [`ActionIcon`](action_icon.md)
with `aria-expanded` and a name that stays the same in both states. You spread
`aria-controls` to name the panel it opens. Under reduced motion the bars snap
instead of morphing.

## Usage

The caller holds the open state and hands it to both the burger and the panel.

```rust
use dioxus::prelude::*;
use libero::components::{Burger, Sidebar};
use libero::sx::sx;
use libero::theme::Size;

#[component]
fn Demo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Burger {
            open: open(),
            "aria-controls": "site-nav",
            onclick: move |_| open.toggle(),
            sx: sx().breakpoint(Size::Sm, sx().display("none")),
        }
        Sidebar { id: "site-nav", side: "start", role: "navigation",
            // ..
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `open` | `bool` | - | `true` draws the X. Set either way, it makes the button a disclosure with `aria-expanded`. Leave it unset when the burger opens a modal. |
| `onclick` | `EventHandler<MouseEvent>` | - | Click handler. The caller holds the open state and toggles it here. |
| `label` | `Callback<bool, String>` | - | Replaces the localization's labels. Called with the open state during render, so it can read a live locale. |
| `size` | `ThemeAwareValue` | `md` | The glyph's width and height. The button is one `spacing.xs` step larger. Below 24px it still takes presses in a 24x24 box. |
| `color` | `ThemeAwareValue` | `currentColor` | The bars. Unset, they follow the button's `color`. |
| `disabled` | `bool` | `false` | Disables the button. |
| `parts` | `Parts<BurgerPart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |

Like every component, `Burger` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes, `aria-controls` among them.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Styling](styling.md#style-api) explains how
parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `BurgerPart::Glyph` | `glyph` | The middle bar. The outer two are its `::before` and `::after`. |

## Accessibility

### Libero handles

- The name is "Toggle navigation" while `open` is set, since `aria-expanded`
  carries the state, and "Open navigation" otherwise. Translate both in the
  [localization](localization.md)'s `BurgerLabels`, or per burger with `label`.
- `Burger` warns when `open` is set without `aria-controls`.
- Focus stays on the burger when the panel opens. Moving it is the panel's
  job, and [`Drawer`](drawer.md) already traps it.

```rust,ignore
Localization {
    burger: BurgerLabels {
        open: "Menü öffnen",
        toggle: "Menü umschalten",
    },
    ..Localization::ENGLISH
}

// Or, when the locale is only known at runtime:
Burger {
    open: open(),
    label: move |_| t("nav.toggle"),
}
```

### You must

- Set `open` only when the burger expands a panel, and spread `aria-controls`
  with that panel's id.
- Leave `open` unset for a burger that opens a modal, since a modal is not
  expanded by its trigger.

```rust,ignore
Burger { onclick: move |_| modal.open() }
```

## Theme defaults

`BurgerDefaults` on the theme.

| Field | Description |
|---|---|
| `size` | Default `size`, `md`. |
| `sizes` | The six-step size scale, 12, 18, 24, 34, 42 and 52px. |
| `transition_duration` | Morph duration, `300ms`. |
| `transition_timing` | Morph easing, `ease`. |

The bars read `var(--lsx-burger-color, currentColor)`. `color` sets that
variable, and without it a `color` on the button reaches the bars.
