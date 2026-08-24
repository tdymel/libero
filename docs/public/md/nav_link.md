# NavLink

Crate: `libero`
Import: `use libero::components::NavLink;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/nav_link.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A navigation list item - a link with a themed active/hover background and `aria-current`, for a sidebar or nav bar.

A navigation list item - [`Anchor`](anchor.md) plus a themed active/hover
background and `aria-current`, for a sidebar or nav bar link. Colors come from
the theme (`Theme::nav_link`) by default, and only show once a link is active.
Left unset, `active` compares `to` against the current route, so a link knows on
its own whether it is the page you are on.

## Usage

Two links, so `active`'s auto-detection can be seen deciding *between* them: the
one whose `to` matches the current route gets the tint and `aria-current="page"`.

```rust
use dioxus::prelude::*;
use libero::{components::{Flex, NavLink}, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        Flex { direction: "column", gap: "xs", sx: sx().width("240px"),
            NavLink { to: Route::GettingStarted {}, "Getting Started" }
            NavLink { to: Route::NavLinkPage {}, "NavLink" }
        }
    }
}
```

`active` can also be forced, for a section-level parent item or anywhere
auto-detection has nothing to compare against (an external target, or no router
mounted):

```rust
use dioxus::prelude::*;
use libero::{components::{Flex, NavLink}, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        Flex { direction: "column", gap: "xs", sx: sx().width("240px"),
            NavLink { to: "/docs", active: true, color: "success", "Docs" }
            NavLink { to: "/docs/changelog", disabled: true, "Changelog" }
        }
    }
}
```

## Accessibility

The active link carries `aria-current="page"`, so a screen reader announces
which item in the list is the current one. A `disabled` link renders an `<a>`
with `aria-disabled="true"` and `tabindex="-1"` and drops out of the tab order;
`pointer-events: none` stops the click. Hover is neutral grey rather than the
accent color on purpose - hovering must not look like a selection. Wrap a list
of them in a `<nav>` and they are a navigation landmark.

`scroll_into_view` scrolls the active link into view when it becomes active,
acting on whatever scrollable ancestor exists - useful for a long sidebar, and
off by default because it is wrong everywhere else.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `to` | `NavigationTarget` | required | A plain path/URL or a typed route, same as `Anchor::to`. |
| `target` | `String` | - | The link's `target` attribute. |
| `color` | `ThemeAwareValue` | `primary` | Tints the active/hover background. Only the color *family* is used - the tint is always its lightest shade. |
| `active` | `bool` | follows the route | Unset, it compares `to` against the current route, so it is only ever true for an internal target with a router mounted. Set it explicitly for a section-level parent item, or anywhere auto-detection has nothing to compare against. |
| `disabled` | `bool` | `false` | Dims the link and disables navigation. |
| `scroll_into_view` | `bool` | `false` | Scrolls this link into view when it becomes active, if it isn't already visible. Acts on whatever scrollable ancestor happens to exist, which only suits a sidebar. |
| `children` | `Element` | required | The link's content. |

Like every component, `NavLink` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Theme defaults

`NavLinkDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `color` | `Color` | Color family the active tint is derived from - `primary` by default. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-nav-link-active-background` | Background of the active link - the lightest shade (S1) of the resolved color family. |

## Data attributes

State tokens on the root's `data-state`.

| Token | Condition |
|---|---|
| `active` | The link is active, whether detected or forced. |
| `disabled` | `disabled` is set. |
