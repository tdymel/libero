# Collapse

Crate: `libero`
Import: `use libero::components::Collapse;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/collapse.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Animates its children's height open and closed, over a grid row rather than a measured pixel height.

Animates its children's height open and closed. It renders two `div`s - a grid
whose single row goes from `0fr` to `1fr`, and the clipped box holding your
content - so the animation re-measures itself for free whenever the content's
own height changes. `open` is strictly controlled, and `Collapse` renders no
role and no ARIA of its own: the disclosure semantics belong to whatever owns
the trigger.

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

`Collapse` never owns `open`. Hold it in a signal, flip it from whatever your
trigger is, and pass it down - the same shape `Tabs::value` uses.

`Collapse` renders no ARIA of its own, so the trigger carries the disclosure
semantics: `aria_expanded` on the button, and an `aria_controls` pointing at an
`id` you set on the `Collapse`. The root element is in the DOM whether the panel
is open, closed or unmounted, so that reference never dangles.

## What closed content costs

A closed panel keeps its children in the DOM by default, so a half-typed form
survives being collapsed and a trigger's `aria-controls` always resolves - the
root element is present whatever `keep_mounted` says, and a caller may put `id`,
`role="region"` and `aria-labelledby` on it through the usual attributes.

What closed content is not is reachable: the inner box is `visibility: hidden`,
which takes it out of the focus order and out of the accessibility tree. That
step is delayed by the animation's duration, so the panel stays visible and
announced for the whole close rather than vanishing on the first frame. No
`aria-hidden` is set - `visibility: hidden` already removes the subtree, and it
does it in step with the animation.

`keep_mounted: false` goes further and removes the children once the exit
transition ends:

```rust
use dioxus::prelude::*;
use libero::components::{Button, Collapse, Text};

#[component]
fn Demo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Button { onclick: move |_| open.toggle(), "Release notes" }
        Collapse {
            open: open(),
            keep_mounted: false,
            duration: 600,
            Text { "Rebuilt from scratch every time it opens." }
        }
    }
}
```

Use it when there is nothing to preserve - content built from a closure, or a
long list you would rather not pay for while it is hidden - and expect the state
inside to be gone on reopen.

Two limits of that mode, both about the `transitionend` it unmounts on:

- Under `@media (prefers-reduced-motion: reduce)` there is no transition, so no
  `transitionend` ever arrives and the children stay mounted after the close.
  They are still `visibility: hidden`, so unreachable and unannounced - the mode
  degrades to `keep_mounted: true` rather than breaking.
- `duration: 0` is handled: a zero-length transition never runs either, so
  `Collapse` unmounts straight from `open` instead of waiting for an event that
  cannot come.

## Accessibility

`Collapse` renders no `role`, no ARIA state and no keyboard handling. It never
sees the trigger, and a disclosure's semantics live on the trigger: give that
button `aria-expanded` and an `aria-controls` pointing at an `id` you set on the
`Collapse` root, which is in the DOM whether the panel is open, closed, or
unmounted.

Closed content is removed from the focus order and the accessibility tree by
`visibility: hidden`, so it is never a phantom tab stop. If focus is inside the
panel when it closes, it does not come back on its own - the browser drops it to
the document body. A component that owns both the trigger and the panel should
send focus back to the trigger when it closes one that contained focus.

Motion is guarded: under `prefers-reduced-motion: reduce` every transition is
`none`, and the panel opens and closes instantly.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `open` | `bool` | required | Whether the panel is expanded. Strictly controlled - `Collapse` holds no open state of its own. |
| `keep_mounted` | `bool` | `true` | Keep the children in the DOM while closed. `false` unmounts them when the exit transition ends. |
| `duration` | `u32` | `theme.collapse.duration` | Milliseconds. `0` disables the animation. |
| `children` | `Element` | required | The content that grows and shrinks. |

Like every component, `Collapse` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`CollapseDefaults` on the theme, as `theme.collapse`.

| Field | Type | Default | Description |
|---|---|---|---|
| `duration` | `u32` | `200` | Milliseconds the open and close animations take. |
| `easing` | `&'static str` | `"ease"` | The CSS timing function both transitions use. |
| `animate_opacity` | `bool` | `true` | Whether a closing panel fades as well as shrinking. One app-wide decision, so a theme field rather than a prop. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-collapse-duration` | Duration of both transitions, as a CSS time. |
| `--lsx-collapse-easing` | Timing function of both transitions. |
| `--lsx-collapse-opacity-closed` | A closed panel's opacity - `0` when the theme animates opacity, `1` when it does not. |

The `duration` prop writes `--lsx-collapse-duration-override` into the root's
`style` attribute, so a per-instance duration never mints a new class. Custom
properties inherit, which is how the value reaches the content element too.

## Data attributes

Both elements carry the same token, so either can be styled from a `states`
selector.

| Token | When |
|---|---|
| `open` | The panel is expanded, or is animating open. |
| `closed` | The panel is collapsed, or is animating closed. |
