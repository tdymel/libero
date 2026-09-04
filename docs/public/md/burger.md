# Burger

Crate: `libero`
Import: `use libero::components::Burger;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/burger.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Three bars that morph into an X - an `ActionIcon` carrying the glyph and the three ARIA facts a nav toggle needs.

Three bars that morph into an X. It renders an [`ActionIcon`](action-icon.md) -
a real `<button type="button">` with the ripple and the disabled handling
already right - and adds the glyph plus the three ARIA facts a burger is
usually missing: `aria-expanded`, an accessible name that changes with the
state, and the `aria-controls` you spread to name the panel. The morph animates
`background-color` and `transform` only, and is dropped entirely under
`prefers-reduced-motion: reduce` - the two states differ in shape, not just in
position, so snapping between them stays legible.

## Props

| Prop | Type | Default | Does |
|---|---|---|---|
| `opened` | `bool` | unset | `true` draws the X, and either value emits `aria-expanded`, which makes the button a disclosure. Omit it for a burger that opens something that is not one |
| `onclick` | `EventHandler<MouseEvent>` | | `Burger` never owns the open state |
| `label` | `Callback<bool, String>` | theme | Replaces the theme's two labels, keyed by `opened` |
| `size` | `ThemeAwareValue` | `md` | The glyph's width and height; the bars are a twelfth of it thick, and the button is one `spacing.xs` larger |
| `color` | `ThemeAwareValue` | `currentColor` | The bars. Unset they inherit |
| `disabled` | `bool` | `false` | Passed through to the button |

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Burger, Sidebar};
use libero::sx::sx;
use libero::theme::Size;

#[component]
fn Demo() -> Element {
    let mut opened = use_signal(|| false);

    rsx! {
        Burger {
            opened: opened(),
            "aria-controls": "site-nav",
            onclick: move |_| opened.toggle(),
            sx: sx().breakpoint(Size::Sm, sx().display("none")),
        }
        Sidebar { id: "site-nav", side: "left", role: "navigation",
            // ..
        }
    }
}
```

The panel owns the state. `Burger` is told what it is, and tells the
accessibility tree - which is why `aria-controls` has to name a real element:
without it the state is announced but the thing in that state is not.

Focus stays on the burger when the panel opens. Moving it into the panel is the
panel's decision - [`Drawer`](drawer.md) traps already, and a burger that opens
a static sidebar must not steal focus.

## Accessibility

**Leave `opened` unset when the burger opens a modal.** `opened` makes the
button a disclosure with `aria-expanded`; a modal is not expanded by its
trigger, it replaces the page.

```rust
// No `opened`: nothing is expanded, so nothing announces a state.
Burger { onclick: move |_| modal.open() }
```

**Spread `aria-controls` whenever `opened` is set**, naming the element that
opens, or `Burger` warns. Nothing sets it internally.

**The accessible name is the theme's**, and it changes with `opened`. Translate
it once in `BurgerLabels`, or per burger with `label` when the locale is only
known at runtime:

```rust
Theme {
    burger: BurgerDefaults {
        labels: BurgerLabels { open: "Menü öffnen", close: "Menü schließen" },
        ..BurgerDefaults::DEFAULT
    },
    ..Theme::DEFAULT
}

// Or, when the locale is only known at runtime:
Burger {
    opened: opened(),
    label: move |opened| t(if opened { "nav.close" } else { "nav.open" }),
}
```

## Styling

The bars read `var(--lsx-burger-color, currentColor)`, and the theme never
declares that var. So `color: "primary"` sets it, `sx().color("white")` on the
button reaches the bars through the fallback, and neither silently wins over
the other.

`BurgerDefaults` carries `size` (`md`), the six-step `sizes` scale
(12/18/24/34/42/52px - Mantine's five plus an `xxl` of ours),
`transition_duration` (`300ms`), `transition_timing` (`ease`) and `labels`.
Motion is a theme decision, not a per-call-site one; an off-scale one-off goes
through `sx` like any other off-scale value.

There is no radius, on the prop or in the defaults: a radius is meaningless on
three bars, so `Burger` neither takes one nor overrides `ActionIcon`'s.

The bar thickness is derived (`size / 12`) rather than a prop: it is right at
every size, and a `line_size` override is one more thing to get wrong.

## Burger or ActionIcon?

An `ActionIcon` with an icon child is the right answer for any icon-only
button. `Burger` is the one where the glyph has two states that morph, the name
changes with the state, and `aria-expanded` has to follow - three things a
caller has to get right at once, bundled so they are right by default.
