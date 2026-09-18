# Drawer

Crate: `libero`
Import: `use libero::hooks::{DrawerOptions, ModalScope, use_drawer};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/use_drawer.rs>
Index: [index.md](index.md) lists every other page
Description: A dimmed, focus-trapped panel docked to one edge, `use_modal` with the docking around it.

A dimmed, focus-trapped panel docked to one edge. `use_drawer` is
[`use_modal`](modal.md) with the docking around it, so it has the same handle,
arguments and results. The panel is a [Dialog](dialog.md) surface. For a panel
in the page flow, use [Sidebar](sidebar.md).

## Usage

`DrawerOptions` holds what every opening shares. The closure is the panel's
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
            anchor: "end".into(),
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

`anchor` picks the edge, `start`, `end`, `top` or `bottom`. `size` is the
width on a start or end edge and the height on a top or bottom one. The other
axis is always full. Both are read on every render, so an anchor in a signal
moves an open drawer. `start` and `end` follow the text direction: `start` is
the right edge under `dir="rtl"`.

A drawer takes per-opening data and returns a result like any modal. See
[Modal](modal.md) for `open_with`, `Opening`, `onresult` and `.await`.

```rust,ignore
let details = use_drawer(
    DrawerOptions {
        anchor: "end".into(),
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

## Accessibility

Escape and a backdrop click close it and settle the `Opening` with `None`, so a
result handler never runs on a dismissal.

Set `DrawerOptions::aria_label`, since the panel is a dialog with no name of its
own. Unset, it warns in a debug build. It has no header close button, so give
its content a way to close it.

## API

### `use_drawer`

```rust,ignore
pub fn use_drawer<S: Clone + 'static, R: Clone + 'static>(
    options: DrawerOptions,
    render: impl FnMut(ModalScope<S, R>) -> Element + 'static,
) -> ModalHandle<S, R>
```

Returns the same `ModalHandle` as `use_modal`. It, `ModalScope` and `Opening`
behave the same. See [Modal](modal.md).

### `DrawerOptions`

`DrawerOptions` implements `Default`, so a literal sets only what it needs,
`DrawerOptions { anchor: "end".into(), ..Default::default() }`.

| Field | Type | Default | Description |
|---|---|---|---|
| `anchor` | `Input<DrawerAnchor>` | `start` | The edge the panel docks to. |
| `size` | `Input<Size>` | `md` | Width when docked start or end, height when docked top or bottom. |
| `z_index` | `Input<ThemeAwareValue>` | - | Stacking order of the panel. |
| `aria_label` | `Option<String>` | - | Names the panel, which is a dialog. Unset warns in a debug build. |

## Theme defaults

`DrawerDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size`, `md`. |
| `sizes` | `Sizes<u16>` | Panel extent per size step, in px. |

The panel's background, shadow and padding come from the `Dialog` defaults.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-drawer-size-<size>` | Panel extent for that size step. |

Stacking order goes through `Float`'s `--lsx-z-index-float-override`.

## Data attributes

State tokens on the panel's `data-state`, space separated.

| Token | Condition |
|---|---|
| `anchor-start` / `anchor-end` / `anchor-top` / `anchor-bottom` | The `anchor` in effect. |
| `size-<size>` | The `size` in effect. |
