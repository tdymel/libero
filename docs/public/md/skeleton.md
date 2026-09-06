# Skeleton

Crate: `libero`
Import: `use libero::components::Skeleton;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/feedback/skeleton.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A placeholder for loading content - a standalone grey shape, or a wrapper that covers the real content until it is ready.

A placeholder for content that is still loading, used two ways. Without
children it is a grey shape, and a few of them stand in for a layout. Wrapped
around the real content, it covers that content while `visible` and steps aside
when `visible` turns `false`: the layout is written once, and the placeholder is
exactly its size because it *is* the content underneath. The grey pulses; under
`prefers-reduced-motion: reduce` it stops half-way, which still reads as a
placeholder rather than a disabled control.

## Usage

The wrapper - the layout is written once:

```rust
use dioxus::prelude::*;
use libero::components::{Button, Flex, Skeleton, Text};

#[component]
fn Profile(name: Option<String>) -> Element {
    rsx! {
        Skeleton { visible: name.is_none(),
            Flex { direction: "row", gap: "sm", align: "center",
                Text { {name.clone().unwrap_or_default()} }
                Button { size: "xs", "Follow" }
            }
        }
    }
}
```

Standalone shapes:

```rust
use dioxus::prelude::*;
use libero::components::{Flex, Skeleton};

#[component]
fn Placeholder() -> Element {
    rsx! {
        Flex { direction: "row", gap: "md", align: "center",
            Skeleton { circle: true, height: "40px" }
            Flex { gap: "xs",
                Skeleton { height: "12px", width: "180px" }
                Skeleton { height: "12px", width: "120px" }
            }
        }
    }
}
```

A standalone shape needs a `height`: with no children and no height it is zero
pixels tall.

A `circle` copies its `height` into its width. Without a `height` it wraps its
children and is as wide as they are - round around a square child such as an
`Avatar`, a pill around a wider one.

## Announcing it

A skeleton says nothing to a screen reader, on purpose. While it covers, its
content is hidden, and `aria-busy` on a hidden element reaches nobody. Mark the
region you are filling `aria-busy` while it waits - the same rule as `Loader`.

```rust
use dioxus::prelude::*;
use libero::components::{Box, Skeleton};

#[component]
fn Card() -> Element {
    let profile = use_resource(load_profile);

    rsx! {
        Box {
            "aria-busy": profile.read().is_none(),
            Skeleton { visible: profile.read().is_none(),
                ProfileCard { profile: profile.read().clone().unwrap_or_default() }
            }
        }
    }
}
#
# #[derive(Clone, PartialEq, Default)]
# struct Profile;
# async fn load_profile() -> Profile { Profile }
# #[component] fn ProfileCard(profile: Profile) -> Element { rsx! {} }
```

## How it hides the content

While `visible`, the root is `visibility: hidden` and only its `::after` - the
grey - is `visibility: visible`. Every descendant inherits `hidden`: it keeps its
layout, so the placeholder is exactly its size, but it is not painted, whatever
its `z-index`. Nothing covers it, so the grey pulses over whatever is really
behind the skeleton and looks right on a tinted surface as well as on paper.

The one hole: a descendant that sets `visibility: visible` on itself opts back
in and shows through the skeleton. Do not put one under a visible skeleton.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `visible` | `bool` | `true` | Cover the children, or draw the standalone shape. `false` shows the children as they are. |
| `height` | `ThemeAwareValue` | - | A CSS length. Unset, the height of the children. |
| `width` | `ThemeAwareValue` | `100%` | A CSS length. Ignored when `circle`. |
| `circle` | `bool` | `false` | Width equals `height`, corners fully round. Without `height`, as wide as the children. |
| `radius` | `Size` | `sm` | Corner. Ignored when `circle`. |
| `animate` | `bool` | `true` | Run the pulse. |
| `children` | `Element` | - | The real content, when the skeleton wraps it. |

Like every component, `Skeleton` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Theme defaults

`SkeletonDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `radius` | `Size` | `Sm`. |
| `color` | `ColorValue` | The grey: `grey.3`. |
| `duration` | `&'static str` | One full pulse: `1500ms`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-skeleton-color` | The grey, from `SkeletonDefaults::color`, on `:root`. |
| `--lsx-skeleton-duration` | One pulse, from `SkeletonDefaults::duration`, on `:root`. |
| `--lsx-skeleton-radius` | The active corner, resolved on the root. |
| `--lsx-skeleton-height` / `--lsx-skeleton-width` | `height`/`width`, per instance. |

## Data attributes

State tokens on the root's `data-state`.

| Token | Condition |
|---|---|
| `visible` | `visible` is `true`. |
| `animate` | `animate` is `true`. |
| `circle` | `circle` is `true`. |
| `radius-<size>` | The `radius` in effect; absent when `circle`. |
