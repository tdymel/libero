# NavigationBar

Crate: `libero`
Import: `use libero::components::{NavigationBar, NavigationBarItem};`
Feature: `navigation-bar` (off by default, turns on `nav-link`, `float`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/navigation_bar.rs>
Index: [index.md](index.md) lists every other page
Description: A phone's bar of three to five top-level destinations, each an icon over a label, the current one marked with `aria-current`.

A phone's bar of three to five top-level destinations. It is a `<nav>`
landmark; each `NavigationBarItem` is an icon over a label and its own Tab
stop. The selected item gets `aria-current="page"` and a pill in a light tint
of `color` behind its icon. Left unset, `selected` compares `to` with the
current route.

## Usage

An item with `to` is a link; the one whose `to` matches the current route is
selected.

```rust
use dioxus::prelude::*;
use libero::components::{NavigationBar, NavigationBarItem, NavigationBarVariant, Pictogram};
use libero::theme::responsive;
use pictogram_icons_lucide as lucide;

#[component]
fn Demo() -> Element {
    rsx! {
        NavigationBar {
            "aria-label": "Main",
            variant: responsive(NavigationBarVariant::Bar).sm(NavigationBarVariant::Rail),
            NavigationBarItem {
                to: "/home",
                icon: rsx! { Pictogram { icon: lucide::house::outlined } },
                "Home"
            }
            NavigationBarItem {
                to: "/search",
                icon: rsx! { Pictogram { icon: lucide::search::outlined } },
                "Search"
            }
            NavigationBarItem {
                to: "/inbox",
                icon: rsx! { Pictogram { icon: lucide::inbox::outlined } },
                "Inbox"
            }
            NavigationBarItem {
                to: "/profile",
                icon: rsx! { Pictogram { icon: lucide::user::outlined } },
                "Profile"
            }
        }
    }
}
```

## Props

### NavigationBar

| Prop | Type | Default | Description |
|---|---|---|---|
| `variant` | `Responsive<NavigationBarVariant>` | `bar` | `"bar"`, a row along the bottom edge for a phone, or `"rail"`, an 88px column on the inline-start edge for a tablet or a desktop: the right edge in a right-to-left page. A rail spans the viewport's height and sticks to its top, or docks to the viewport with `position: "fixed"`, and publishes its measured width as `--lsx-navigation-rail-width` in place of the bar's height. Set one per breakpoint, as `responsive(NavigationBarVariant::Bar).sm(NavigationBarVariant::Rail)` for a bar on a phone and a rail from the theme's `sm` up: each switch is a CSS media query, so server-rendered HTML and the first paint are right without a script. `position` applies to the shape in effect at each width, and each publishes its own variable there. Keep the same items in the same order at every width. |
| `position` | `NavigationBarPosition` | `static` | `"static"` stays in the flow. `"sticky"` holds it at the bottom of its scroller, `"fixed"` at the viewport's; both publish the bar's measured height as `--lsx-navigation-bar-height` and keep focus clear of the bar. Pad the page's end by `var(--lsx-navigation-bar-height)`. `"fixed"` clears a phone's home indicator, and needs `viewport-fit=cover` in the page's viewport meta, or the browser reports no safe area. A native app docks both the same way. |
| `show_labels` | `LabelVisibility` | `always` | `"always"`, `"selected"` (only the selected item's) or `"never"`. A hidden label still names its item. |
| `color` | `ThemeAwareValue` | `primary` | Colours the selected item's pill. Only the color family counts: the pill is its lightest shade. |
| `z_index` | `ThemeAwareValue` | the header's | Stacking order of a sticky or fixed bar. |
| `parts` | `Parts<NavigationBarPart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |
| `children` | `Element` | required | Three to five `NavigationBarItem`s. |

### NavigationBarItem

| Prop | Type | Default | Description |
|---|---|---|---|
| `to` | `NavigationTarget` | - | A path, a URL or a typed route: the item is a link. Wins over `onclick`. |
| `onclick` | `EventHandler<MouseEvent>` | - | Without `to`, the item is a button that calls this. |
| `selected` | `bool` | follows the route | Unset, it compares `to` with the current route. Set it for `onclick` items. |
| `icon` | `Element` | required | Drawn in the pill above the label, hidden from screen readers. |
| `badge` | `Element` | - | A count or dot over the icon, such as an `Indicator`. Put its count in the item's `aria-label` too. |
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
| `NavigationBarPart::Item` | `item` | Every item. |
| `NavigationBarPart::Icon` | `icon` | The pill holding an item's icon and badge. |
| `NavigationBarPart::Label` | `label` | An item's label. |

## Accessibility

### Libero handles

- The bar is a `<nav>` landmark, and each item its own Tab stop, in reading
  order.
- The selected item carries `aria-current="page"`.
- The icon is hidden from screen readers, so the label names the item. A label
  hidden by `show_labels` stays in the accessibility tree.
- Every item is at least 56px tall and shares the bar's width equally. A long
  label wraps to two lines, then ends in an ellipsis.
- A rail keeps its items in the same reading and Tab order, top to bottom. At
  200% text it stays 88px wide: labels wrap and the rail scrolls rather than
  covering the page.
- A sticky or fixed bar pads the page's scroll, so a focused element is not
  hidden under it.
- In forced colors the selected pill takes the system highlight.
- More than five items logs a warning in debug builds.

### You must

- Name the bar with an `aria-label`, such as "Main"; a debug build warns
  without one.
- Put a badge's count in the item's `aria-label`, starting with the visible
  label: "Inbox, 3 unread".
- With `position: "fixed"`, pad the page by `var(--lsx-navigation-bar-height)`,
  or a fixed rail's inline-start by `var(--lsx-navigation-rail-width)`.
- Switch to the rail with the same items in the same order, so the page keeps
  its landmarks and current item at every width.
- A sticky bar closing a scrolling pane of its own: give the pane
  `scroll-padding-bottom: var(--lsx-navigation-bar-height)`, so a focused
  element in it scrolls clear of the bar. In a native app set
  `--lsx-scroll-padding-bottom` to the same value beside it.

### Example

An app bar, `NavigationBar { "aria-label": "Main" }` with Home, Search and
an Inbox item labelled `"aria-label": "Inbox, 3 unread"`: a screen reader
lists the "Main" navigation, and the current item reads as the current page.

### Limits

- A native app has no line clamp: a long label is cut at two lines without an
  ellipsis.

## Theme defaults

`NavigationBarDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `color` | `Color` | Color family of the selected pill, `primary`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-navigation-bar-height` | On `:root` while a sticky or fixed bar is mounted, under each width where a bar shows: its measured height, bottom safe area included. |
| `--lsx-navigation-rail-width` | On `:root` while a rail is mounted, under each width where a rail shows: its measured width, inline-start safe area included. |
| `--lsx-navigation-bar-pill` | Background of the selected item's pill, the lightest shade of the color family. |
| `--lsx-navigation-bar-pill-color` | Icon colour on the pill, a darker shade of the color family that reaches 4.5:1 on it. |

## Data attributes

State tokens on the bar's `data-state`.

| Token | Condition |
|---|---|
| `static`, `sticky`, `fixed` | The bar's `position`. |
| `rail` | `variant: "rail"`. A `variant` per breakpoint sets none of these position and shape tokens. |
| `rail-fixed` | A rail with `position: "fixed"`. |
| `labels-selected`, `labels-never` | `show_labels` is set to it. |

State tokens on an item's `data-state`.

| Token | Condition |
|---|---|
| `selected` | The item is selected, whether detected or set. |
| `disabled` | `disabled` is set. |
