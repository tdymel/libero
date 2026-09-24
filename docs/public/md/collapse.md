# Collapse

Crate: `libero`
Import: `use libero::components::Collapse;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/collapse.rs>
Index: [index.md](index.md) lists every other page
Description: Animates its children's height open and closed, and follows the content when its height changes.

Animates its children's height open and closed, and follows the content when
its height changes. You own `open`. `Collapse` has no role or ARIA, so the
trigger carries `aria_expanded` and an `aria_controls` pointing at the panel's
`id`.

Focus inside a closing panel does not return to the trigger on its own. Use
[`use_focus_return`](use_focus_return.md), with `remember_active()` on every
open and `restore()` where the panel closes from inside.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Collapse, Flex, Text},
    hooks::use_focus_return,
};

#[component]
fn Demo() -> Element {
    let mut open = use_signal(|| false);
    let trigger = use_focus_return();

    rsx! {
        Flex {
            direction: "column",
            align: "flex-start",
            gap: "sm",
            Button {
                onclick: move |_| {
                    if !open() {
                        trigger.remember_active();
                    }
                    open.toggle();
                },
                aria_expanded: open(),
                aria_controls: "shipping-details",
                "Shipping details"
            }
            Collapse {
                id: "shipping-details",
                open: open(),
                Flex {
                    direction: "column",
                    align: "flex-start",
                    gap: "sm",
                    Text { "Shipping is calculated at checkout." }
                    Button {
                        onclick: move |_| {
                            open.set(false);
                            trigger.restore();
                        },
                        "Done"
                    }
                }
            }
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `open` | `bool` | required | Whether the panel is expanded. You own this state. |
| `keep_mounted` | `bool` | `true` | Keeps the children in the DOM while closed, out of the focus order and hidden from screen readers, so a half-typed form survives. `false` removes them once the panel has closed. |
| `duration` | `u32` | `200` | Animation length in milliseconds. `0` turns the animation off. |
| `children` | `Element` | required | The content that grows and shrinks. |

Like every component, `Collapse` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Accessibility

### Libero handles

- With `keep_mounted`, the closed children stay in the DOM but out of the
  focus order and hidden from screen readers.

### You must

- Give the trigger `aria_expanded` and an `aria_controls` pointing at the
  panel's `id`: `Collapse` has no role or ARIA.
- Return focus yourself when the panel closes from inside: use
  [`use_focus_return`](use_focus_return.md), with `remember_active()` on every
  open and `restore()` where the panel closes.

## Theme defaults

`CollapseDefaults` on the theme, as `theme.collapse`.

| Field | Type | Default | Description |
|---|---|---|---|
| `duration` | `u32` | `200` | Milliseconds the open and close animations take. |
| `easing` | `&'static str` | `"ease"` | The CSS timing function both transitions use. |
| `animate_opacity` | `bool` | `true` | Whether a closing panel fades as well as shrinking. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-collapse-duration` | Duration of both transitions, as a CSS time. |
| `--lsx-collapse-easing` | Timing function of both transitions. |
| `--lsx-collapse-opacity-closed` | A closed panel's opacity, `0` when the theme animates opacity and `1` when it does not. |

## Data attributes

On the root's `data-state`, where a caller's `sx`, `class` and `states` land.

| Token | When |
|---|---|
| `open` | The panel is expanded, or is animating open. |
| `closed` | The panel is collapsed, or is animating closed. |
