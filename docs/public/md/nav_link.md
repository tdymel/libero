# NavLink

Crate: `libero`
Import: `use libero::components::NavLink;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/nav_link.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A navigation list item for a sidebar or nav bar, a link that marks the current page with `aria-current`.

A navigation list item for a sidebar or nav bar. It is an [`Anchor`](anchor.md)
that marks the current page with an active style and `aria-current`. Left
unset, `active` compares `to` with the current route.

## Usage

The link whose `to` matches the current route is active and gets
`aria-current="page"`.

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

Set `active` for a section's parent item, or where there is no route to compare,
such as an external target or an app without a router.

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

`description` adds a dimmed line under the label. `nested` puts child links
behind a toggle beside the parent. `default_opened` sets whether they show at
first, and `opened` with `onchange` controls it.

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

Wrap a list of them in a `<nav>` to make a navigation landmark.

`description` is read as the link's description, not its name, so "Docs" stays
"Docs" in a links list.

With `nested`, a disclosure button follows the link with its own tab stop. It
carries `aria-expanded` and `aria-controls`, and its name is the localized
"Show links" plus the link's name ("Show links Docs"). Enter or Space on the
button toggles the panel, and the link still navigates.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `to` | `NavigationTarget` | required | A path, a URL or a typed route, as in `Anchor::to`. |
| `target` | `String` | - | The link's `target` attribute. |
| `color` | `ThemeAwareValue` | `primary` | Tints the active and hovered link. Only the color family counts, and the tint is its lightest shade. |
| `active` | `bool` | follows the route | Unset, it compares `to` with the current route, which needs an internal target and a router. Set it for a section's parent item, or where there is no route to compare. |
| `disabled` | `bool` | `false` | Dims the link and stops navigation. |
| `scroll_into_view` | `bool` | `false` | Scrolls the link into view when it becomes active. It scrolls the nearest scrollable ancestor, so use it in a sidebar. |
| `description` | `String` | - | A dimmed line under the label, read as the link's description. |
| `nested` | `Element` | - | Child `NavLink`s, shown under this one by a toggle button beside it. The link itself still goes to `to`. |
| `opened` | `bool` | - | Whether `nested` shows. Setting it makes it controlled, so pair it with `onchange`. |
| `default_opened` | `bool` | `false` | Whether `nested` shows at first, when `opened` is unset. |
| `onchange` | `EventHandler<bool>` | - | Called with the new `opened` when the toggle is pressed. |
| `children` | `Element` | required | The link's content. |

Like every component, `NavLink` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Theme defaults

`NavLinkDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `color` | `Color` | Color family of the active tint, `primary`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-nav-link-active-background` | Background of the active link, the lightest shade of the color family. |

## Data attributes

State tokens on the root's `data-state`.

| Token | Condition |
|---|---|
| `active` | The link is active, whether detected or forced. |
| `disabled` | `disabled` is set. |
