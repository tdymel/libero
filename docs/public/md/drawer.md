# Drawer

Crate: `libero`
Import: `use libero::components::Drawer;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/drawer.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A portaled, dimmed, focus-trapped panel docked to one edge of the viewport.

A portaled, dimmed, focus-trapped panel docked to one edge, closing on Escape or
a backdrop click. It is mounted only while open - there is no `opened` prop, the
caller's own state is the switch. For an in-flow panel, see
[Sidebar](sidebar.md).

## Usage

A drawer only exists while it is open, so the trigger and the state that opens it
are the example as much as the component is.

```rust
use dioxus::prelude::*;
use libero::{components::{Button, Drawer, Text, Title}, sx::sx};

#[component]
fn Demo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Button { variant: "outlined", onclick: move |_| open.set(true), "Open drawer" }
        if open() {
            Drawer {
                onclose: move |_| open.set(false),
                sx: sx().padding("16px"),
                Title { size: "lg", "Temporary drawer" }
                Text { "Closes on Escape or backdrop click." }
                Button { variant: "outlined", onclick: move |_| open.set(false), "Close" }
            }
        }
    }
}
```

`anchor` picks the edge - `left`, `right`, `top` or `bottom`. `size` is the width
along a left/right edge and the height along a top/bottom one, off the theme's
drawer size scale; the other axis is always the full 100%.

```rust
use dioxus::prelude::*;
use libero::{components::{Button, Drawer, Text}, sx::sx};

#[component]
fn Demo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Button { variant: "outlined", onclick: move |_| open.set(true), "Open drawer" }
        if open() {
            Drawer {
                anchor: "bottom",
                size: "lg",
                onclose: move |_| open.set(false),
                sx: sx().padding("16px"),
                Text { "Docked to the bottom edge." }
            }
        }
    }
}
```

The drawer is a [Modal](modal.md) wrapping a [Float](float.md) wrapping a
[Dialog](dialog.md) surface: the modal supplies the portal, the backdrop and the
focus trap, the float does the edge docking, and the dialog is the panel your
`sx` and `children` land on. That is also why `Drawer` itself renders nothing in
place - the panel lives in a portal.

## Accessibility

The panel is a `Dialog`, so it is a modal dialog: focus is trapped inside while
it is open and Escape closes it. `onclose` is a *request* - Escape and a
backdrop click both call it, and nothing happens until the caller drops its own
open state, so a drawer can refuse to close (an unsaved form) simply by not
acting on it.

Give the panel an accessible name where its content does not already provide one:
`aria_label` passes through to the dialog. Put the trigger's focus back where the
user expects it by unmounting the drawer from the same state the trigger set.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `anchor` | `DrawerAnchor` | `left` | The edge the drawer docks to. |
| `size` | `Size` | `md` | Width along the docked edge (or height, for top/bottom). |
| `z_index` | `ThemeAwareValue` | - | Stacking order for the drawer's `Modal` layer. |
| `onclose` | `EventHandler<()>` | - | Requested by Escape or a backdrop click. `Drawer` tracks no open/closed state. |
| `children` | `Element` | required | The panel's content, rendered inside a `Dialog` surface. |

Like every component, `Drawer` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes - all of which land on the
`Dialog` panel, not on an in-flow root.

## Theme defaults

`DrawerDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `size` | `Sizes<u16>` | Panel extent per size step, in px. |

The panel's surface - background, shadow, padding - comes from the theme's
`Dialog` defaults.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-drawer-size-<size>` | Panel extent for that size step. |

Stacking order goes through `Float`'s `--lsx-z-index-float-override`.

## Data attributes

State tokens on the panel's `data-state`, space separated.

| Token | Condition |
|---|---|
| `anchor-left` / `anchor-right` / `anchor-top` / `anchor-bottom` | The `anchor` in effect. |
| `size-<size>` | The `size` in effect. |
