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
#
# #[derive(Clone, PartialEq, Routable)]
# enum Route {
#     #[route("/0")]
#     GettingStarted {},
#     #[route("/1")]
#     NavLinkPage {},
# }
# #[component] fn GettingStarted() -> Element { rsx! {} }
# #[component] fn NavLinkPage() -> Element { rsx! {} }
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

`description` adds a dimmed line under the label, and `nested` puts child links
behind a toggle beside the parent. `default_opened` sets where it starts;
`opened` with `onchange` controls it:

```rust
use dioxus::prelude::*;
use libero::components::NavLink;

#[component]
fn Demo() -> Element {
    let mut opened = use_signal(|| true);
    rsx! {
        NavLink {
            to: "/docs",
            description: "Guides and API",
            opened: opened(),
            onchange: move |next| opened.set(next),
            nested: rsx! {
                NavLink { to: "/docs/install", "Install" }
                NavLink { to: "/docs/theming", "Theming" }
            },
            "Docs"
        }
    }
}
```

## Accessibility

Wrap a list of them in a `<nav>` and they are a navigation landmark.

`description` is the link's accessible description, not part of its name, so
"Docs" stays "Docs" in a links list.

With `nested`, the link keeps its own tab stop and a disclosure button follows
it: `aria-expanded` for the state, `aria-controls` for the panel, and a name
built from the localized "Show links" plus the link's own name ("Show links
Docs"). Enter or Space on the button toggles the panel; the link still
navigates.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `to` | `NavigationTarget` | required | A plain path/URL or a typed route, same as `Anchor::to`. |
| `target` | `String` | - | The link's `target` attribute. |
| `color` | `ThemeAwareValue` | `primary` | Tints the active/hover background. Only the color *family* is used - the tint is always its lightest shade. |
| `active` | `bool` | follows the route | Unset, it compares `to` against the current route, so it is only ever true for an internal target with a router mounted. Set it explicitly for a section-level parent item, or anywhere auto-detection has nothing to compare against. |
| `disabled` | `bool` | `false` | Dims the link and disables navigation. |
| `scroll_into_view` | `bool` | `false` | Scrolls this link into view when it becomes active, if it isn't already visible. Acts on whatever scrollable ancestor happens to exist, which only suits a sidebar. |
| `description` | `String` | - | A dimmed line under the label, read as the link's description. |
| `nested` | `Element` | - | Child `NavLink`s, shown under this one by a toggle button beside it. The link itself still goes to `to`. |
| `opened` | `bool` | - | Whether `nested` shows. Set, it is controlled: pair it with `onchange`. |
| `default_opened` | `bool` | `false` | Whether `nested` shows at first, when `opened` is unset. |
| `onchange` | `EventHandler<bool>` | - | The toggle asks for this `opened`. |
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
