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

Use it when there is nothing to preserve - content built from a closure, or a
long list you would rather not pay for while it is hidden - and expect the state
inside to be gone on reopen.

Under `prefers-reduced-motion: reduce`, and with `duration: 0`, there is no
animation to wait for, so the children go at once. A `Collapse` nested inside
another one's content ends only its own close, never the outer one's.

## Returning focus when it closes

If focus is inside the panel when it closes, it does not come back on its own -
the browser drops it to the document body, and a keyboard user loses their
place. `Collapse` cannot fix this for you: it never sees the trigger, so it has
no element to hand focus back to. Whoever owns both the trigger and the panel
owns the return.

Use `use_focus_return` rather than reaching for the element yourself. It is the
one focus-return contract in the library, and its `restore()` spawns the focus
call - which matters, because focusing inside the dispatch of the event that
closed the panel re-enters a handler whose click is still bubbling, and that
panics. Arm it on the opening edge with `remember_active()`, then call
`restore()` wherever you close the panel from inside it.

**Arm on every open, not once.** `restore()` takes the trigger and forgets it,
deliberately, so a second close cannot pull focus off whatever holds it by then.
A pattern that arms once therefore works once and then silently drops focus to
the body on every close after it. `use_modal` re-arms the same way, inside the
handler that opens.

Some browsers do not focus a button on a mouse click, so a panel opened with the
mouse may remember the document body instead of the trigger. That is acceptable
here: the return only matters to a keyboard user, and for them the trigger is
focused when they activate it.

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

A disclosure's semantics live on the trigger, which `Collapse` never sees: give
that button `aria-expanded` and an `aria-controls` pointing at an `id` you set
on the `Collapse` root. Returning focus when the panel closes is yours too - see
"Returning focus when it closes" above.

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

Both the root and the inner content element carry the same token, for the
component's own rules. Only the root is reachable from the outside: a caller's
`sx`, `class` and `states` all land there, and the content element's tokens are
`Collapse`'s own.

| Token | When |
|---|---|
| `open` | The panel is expanded, or is animating open. |
| `closed` | The panel is collapsed, or is animating closed. |
