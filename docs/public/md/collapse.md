# Collapse

Crate: `libero`
Import: `use libero::components::Collapse;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/collapse.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Animates its children's height open and closed, and follows the content when its height changes.

Animates its children's height open and closed, and follows the content when
its height changes. You own `open`. `Collapse` has no role or ARIA, so the
trigger carries `aria_expanded` and an `aria_controls` pointing at the panel's
`id`.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Button, Collapse, Flex, Text};

#[component]
fn Demo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Flex {
            direction: "column",
            align: "flex-start",
            gap: "sm",
            Button {
                onclick: move |_| open.toggle(),
                aria_expanded: open(),
                aria_controls: "shipping-details",
                "Shipping details"
            }
            Collapse {
                id: "shipping-details",
                open: open(),
                Text { "Shipping is calculated at checkout." }
            }
        }
    }
}
```

Hold `open` in a signal and flip it from your trigger. The panel's root stays in
the DOM even when closed, so `aria_controls` always resolves.

## Closed content

A closed panel keeps its children in the DOM, so a half-typed form survives.
They are hidden from the focus order and from screen readers once the close
animation ends.

`keep_mounted: false` removes the children once the panel has closed.

```rust
use dioxus::prelude::*;
use libero::components::{Button, Collapse, Text};

#[component]
fn Demo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Button {
            onclick: move |_| open.toggle(),
            aria_expanded: open(),
            aria_controls: "release-notes",
            "Release notes"
        }
        Collapse {
            id: "release-notes",
            open: open(),
            keep_mounted: false,
            duration: 600,
            Text { "Rebuilt from scratch every time it opens." }
        }
    }
}
```

Use it when there is nothing to keep, like a long list you would rather not
render while hidden. The state inside is gone on reopen. With
`prefers-reduced-motion: reduce` or `duration: 0` the children go at once.

## Focus return

Focus inside a closing panel does not return to the trigger on its own. The
browser drops it to the page body and a keyboard user loses their place.
`Collapse` never sees the trigger, so the code that owns both returns focus.

Use `use_focus_return`. Call `remember_active()` on every open, since
`restore()` forgets the trigger after one use. Call `restore()` wherever the
panel closes from inside.

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
        Button {
            onclick: move |_| {
                if !open() {
                    trigger.remember_active();
                }
                open.toggle();
            },
            aria_expanded: open(),
            aria_controls: "returning-panel",
            "Edit address"
        }
        Collapse {
            id: "returning-panel",
            open: open(),
            keep_mounted: false,
            Flex {
                direction: "column",
                align: "flex-start",
                gap: "sm",
                Text { "Focus this button, then close the panel with it." }
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
```

## Accessibility

Give the trigger `aria-expanded` and an `aria-controls` pointing at an `id` you
set on the `Collapse`. Return focus as in "Focus return" above.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `open` | `bool` | required | Whether the panel is expanded. You own this state. |
| `keep_mounted` | `bool` | `true` | Keeps the children in the DOM while closed. `false` removes them once the panel has closed. |
| `duration` | `u32` | `200` | Animation length in milliseconds. `0` turns the animation off. |
| `children` | `Element` | required | The content that grows and shrinks. |

Like every component, `Collapse` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

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
