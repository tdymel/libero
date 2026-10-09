# BottomNavigation

Crate: `libero`
Import: `use libero::components::{BottomNavigation, BottomNavigationItem};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/bottom_navigation.rs>
Index: [index.md](index.md) lists every other page
Description: A phone's bar of three to five top-level destinations, each an icon over a label, the current one marked with `aria-current`.

A phone's bar of three to five top-level destinations. It is a `<nav>`
landmark; each `BottomNavigationItem` is an icon over a label and its own Tab
stop. The selected item gets `aria-current="page"` and a pill in a light tint
of `color` behind its icon. Left unset, `selected` compares `to` with the
current route.

## Usage

An item with `to` is a link; the one whose `to` matches the current route is
selected.

```rust
use dioxus::prelude::*;
use libero::components::{BottomNavigation, BottomNavigationItem};

#[component]
fn Demo() -> Element {
    rsx! {
        BottomNavigation { "aria-label": "Main",
            BottomNavigationItem { to: Route::Home {}, icon: rsx! { HomeIcon {} }, "Home" }
            BottomNavigationItem { to: Route::Search {}, icon: rsx! { SearchIcon {} }, "Search" }
        }
    }
}
#
# #[derive(Clone, PartialEq, Routable)]
# enum Route {
#     #[route("/0")]
#     Home {},
#     #[route("/1")]
#     Search {},
# }
# #[component] fn Home() -> Element { rsx! {} }
# #[component] fn Search() -> Element { rsx! {} }
# #[component] fn HomeIcon() -> Element { rsx! {} }
# #[component] fn SearchIcon() -> Element { rsx! {} }
```

Without `to`, an item is a button calling `onclick`; set `selected` yourself
then. `show_labels` hides the labels of all items (`"never"`) or of all but the
selected one (`"selected"`); a hidden label still names its item. A `badge`
sits over the icon; put its count in the item's `aria-label` too.

```rust
use dioxus::prelude::*;
use libero::components::{BottomNavigation, BottomNavigationItem, Indicator};

#[component]
fn Demo() -> Element {
    let mut tab = use_signal(|| 0);
    rsx! {
        BottomNavigation { "aria-label": "Main", show_labels: "selected", color: "success",
            BottomNavigationItem { selected: tab() == 0, onclick: move |_| tab.set(0),
                icon: rsx! { "⌂" }, "Home"
            }
            BottomNavigationItem { selected: tab() == 1, onclick: move |_| tab.set(1),
                "aria-label": "Inbox, 3 unread",
                icon: rsx! { "✉" },
                badge: rsx! { Indicator { label: 3u32 } },
                "Inbox"
            }
        }
    }
}
```

`position: "fixed"` docks the bar to the viewport's bottom edge, clear of a
phone's home indicator. Pad the page by `var(--lsx-bottom-navigation-height)`
so its end is not hidden under the bar. The page needs
`<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">`,
or the browser reports no safe area. A native app docks it to its window
the same way.

```rust
use dioxus::prelude::*;
use libero::{components::{BottomNavigation, BottomNavigationItem, Box}, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        Box { sx: sx().padding_bottom("var(--lsx-bottom-navigation-height)"), "Page" }
        BottomNavigation { "aria-label": "Main", position: "fixed",
            BottomNavigationItem { to: "/", icon: rsx! { "⌂" }, "Home" }
            BottomNavigationItem { to: "/search", icon: rsx! { "⌕" }, "Search" }
        }
    }
}
```

## Props

### BottomNavigation

| Prop | Type | Default | Description |
|---|---|---|---|
| `position` | `BottomNavigationPosition` | `static` | `"static"` stays in the flow. `"sticky"` holds it at the bottom of its scroller, `"fixed"` at the viewport's; both publish the bar's measured height as `--lsx-bottom-navigation-height` and keep focus clear of the bar. A native app docks both the same way. |
| `show_labels` | `LabelVisibility` | `always` | `"always"`, `"selected"` (only the selected item's) or `"never"`. A hidden label still names its item. |
| `color` | `ThemeAwareValue` | `primary` | Colours the selected item's pill. Only the color family counts: the pill is its lightest shade. |
| `z_index` | `ThemeAwareValue` | the header's | Stacking order of a sticky or fixed bar. |
| `parts` | `Parts<BottomNavigationPart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |
| `children` | `Element` | required | Three to five `BottomNavigationItem`s. |

### BottomNavigationItem

| Prop | Type | Default | Description |
|---|---|---|---|
| `to` | `NavigationTarget` | - | A path, a URL or a typed route: the item is a link. Wins over `onclick`. |
| `onclick` | `EventHandler<MouseEvent>` | - | Without `to`, the item is a button that calls this. |
| `selected` | `bool` | follows the route | Unset, it compares `to` with the current route. Set it for `onclick` items. |
| `icon` | `Element` | required | Drawn in the pill above the label, hidden from screen readers. |
| `badge` | `Element` | - | A count or dot over the icon, such as an `Indicator`. |
| `disabled` | `bool` | `false` | Dims the item and takes it out of the Tab order. |
| `children` | `Element` | required | The label. |

Like every component, both also take the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Styling](styling.md#style-api) explains how
parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `BottomNavigationPart::Item` | `item` | Every item. |
| `BottomNavigationPart::Icon` | `icon` | The pill holding an item's icon and badge. |
| `BottomNavigationPart::Label` | `label` | An item's label. |

## Accessibility

### Libero handles

- The bar is a `<nav>` landmark, and each item its own Tab stop, in reading
  order.
- The selected item carries `aria-current="page"`.
- The icon is hidden from screen readers, so the label names the item. A label
  hidden by `show_labels` stays in the accessibility tree.
- Every item is at least 56px tall and shares the bar's width equally. A long
  label wraps to two lines, then ends in an ellipsis.
- A sticky or fixed bar pads the page's scroll, so a focused element is not
  hidden under it.
- In forced colors the selected pill takes the system highlight.
- More than five items logs a warning in debug builds.

### You must

- Name the bar with an `aria-label`, such as "Main"; a debug build warns
  without one.
- Put a badge's count in the item's `aria-label`, starting with the visible
  label: "Inbox, 3 unread".
- With `position: "fixed"`, pad the page by `var(--lsx-bottom-navigation-height)`.
- A sticky bar closing a scrolling pane of its own: give the pane
  `scroll-padding-bottom: var(--lsx-bottom-navigation-height)`, so a focused
  element in it scrolls clear of the bar. In a native app set
  `--lsx-scroll-padding-bottom` to the same value beside it.

### Example

An app bar, `BottomNavigation { "aria-label": "Main" }` with Home, Search and
an Inbox item labelled `"aria-label": "Inbox, 3 unread"`: a screen reader
lists the "Main" navigation, and the current item reads as the current page.

### Limits

- A native app has no line clamp: a long label is cut at two lines without an
  ellipsis.

## Theme defaults

`BottomNavigationDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `color` | `Color` | Color family of the selected pill, `primary`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-bottom-navigation-height` | On `:root` while a sticky or fixed bar is mounted: its measured height, bottom safe area included. |
| `--lsx-bottom-navigation-pill` | Background of the selected item's pill, the lightest shade of the color family. |
| `--lsx-bottom-navigation-pill-color` | Icon colour on the pill, a darker shade of the color family that reaches 4.5:1 on it. |

## Data attributes

State tokens on the bar's `data-state`.

| Token | Condition |
|---|---|
| `static`, `sticky`, `fixed` | The `position`. |
| `labels-selected`, `labels-never` | `show_labels` is set to it. |

State tokens on an item's `data-state`.

| Token | Condition |
|---|---|
| `selected` | The item is selected, whether detected or set. |
| `disabled` | `disabled` is set. |
