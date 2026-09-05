# Drawer

Crate: `libero`
Import: `use libero::hooks::{DrawerOptions, ModalScope, use_drawer};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/drawer.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A dimmed, focus-trapped panel docked to one edge - `use_modal` with the docking around it, so it has the same handle, arguments and results.

A drawer is a hook, not a component. `use_drawer` is [`use_modal`](modal.md)
with a docked panel around the content: the same handle, the same per-opening
arguments, the same results. For an in-flow panel, see [Sidebar](sidebar.md).

## Usage

`DrawerOptions` carries what every opening shares - the edge it docks to, how
wide (or tall) it is, and its stacking order. The closure is the panel's
content.

```rust
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Button, Title},
    hooks::{DrawerOptions, ModalScope, use_drawer},
};

#[component]
fn Demo() -> Element {
    let nav = use_drawer(
        DrawerOptions {
            anchor: "right".into(),
            size: "sm".into(),
            aria_label: Some("Menu".into()),
            ..Default::default()
        },
        |s: ModalScope<()>| {
            rsx! {
                Title { size: "lg", "Menu" }
                Anchor { to: "/", "Home" }
                Button { variant: "text", onclick: move |_| s.close(), "Close" }
            }
        },
    );

    rsx! {
        Button { variant: "outlined", onclick: move |_| { nav.open(); }, "Open menu" }
    }
}
```

`anchor` picks the edge - `left`, `right`, `top` or `bottom`. `size` is the
width along a left/right edge and the height along a top/bottom one, off the
theme's drawer size scale; the other axis is always the full 100%. Both are read
on every render, so an anchor held in a signal switches a drawer that is already
open.

The argument and result types are the modal's, so a drawer takes per-opening
data and answers its caller exactly like any other dialog - see
[Modal](modal.md) for `open_with`, `Opening`, `on_result` and `.await`.

```rust,ignore
let details = use_drawer(
    DrawerOptions {
        anchor: "right".into(),
        aria_label: Some("Order details".into()),
        ..Default::default()
    },
    |s: ModalScope<Order, bool>| {
        let order = s.args();

        rsx! {
            Title { size: "lg", "Order {order.id}" }
            Button { onclick: move |_| s.resolve(true), "Mark shipped" }
        }
    },
);

if details.open_with(order).await == Some(true) {
    refresh().await;
}
```

## What it is made of

The hook supplies the portal, the backdrop and the focus trap; a
[Float](float.md) does the edge docking; and the panel itself is a
[Dialog](dialog.md) surface, which is where the closure's content lands. That
last part is why the panel carries the dialog role for free.

## Accessibility

Escape and a backdrop click close it, settling the `Opening` with `None`, so a
handler written for an answer never runs on a dismissal.

Set `DrawerOptions::aria_label`: the panel is a dialog and has no name of its
own. Unset, it warns in a debug build. Unlike a plain
`Dialog`, the drawer panel renders no header close button - a drawer's content
owns its own dismissal.

## API

### `use_drawer`

```rust,ignore
pub fn use_drawer<S: Clone + 'static, R: Clone + 'static>(
    options: DrawerOptions,
    render: impl FnMut(ModalScope<S, R>) -> Element + 'static,
) -> ModalHandle<S, R>
```

Returns the same `ModalHandle` as `use_modal`; every method on it, on
`ModalScope` and on `Opening` behaves identically. See [Modal](modal.md).

### `DrawerOptions`

`Default`, so a literal overrides only what it needs:
`DrawerOptions { anchor: "right".into(), ..Default::default() }`.

| Field | Type | Default | Description |
|---|---|---|---|
| `anchor` | `Input<DrawerAnchor>` | `left` | The edge the panel docks to. |
| `size` | `Input<Size>` | `md` | Width along the docked edge, height for top/bottom. |
| `z_index` | `Input<ThemeAwareValue>` | - | Stacking order for the docked panel. |
| `aria_label` | `Option<String>` | - | Names the panel, which is a dialog. Unset warns in a debug build. |

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
