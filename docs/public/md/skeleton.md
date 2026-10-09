# Skeleton

Crate: `libero`
Import: `use libero::components::Skeleton;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/feedback/skeleton.rs>
Index: [index.md](index.md) lists every other page
Description: A placeholder for loading content, as a standalone grey shape or a wrapper that hides the real content until it is ready.

A placeholder for content that is still loading. Without children it is a grey
shape, and a few of them stand in for a layout. Wrapped around the real
content, it hides that content while `visible` is set, so the placeholder has
exactly its size. Hidden content is not announced and not reachable with Tab.

## Usage

Wrapped around the real content, so the layout is written once:

```rust
use dioxus::prelude::*;
use libero::components::{Button, Flex, Skeleton, Text};

#[component]
fn Profile(name: Option<String>) -> Element {
    rsx! {
        // The region you fill is busy while it waits.
        div { "aria-busy": name.is_none(),
            Skeleton { visible: name.is_none(),
                Flex { direction: "row", gap: "sm", align: "center",
                    Text { {name.clone().unwrap_or_default()} }
                    Button { size: "xs", "Follow" }
                }
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

A standalone shape needs a `height`, or it is zero pixels tall.

With a grace period before the placeholder shows:

```rust
use std::time::Duration;

use dioxus::prelude::*;
use libero::components::Skeleton;
use libero::platform::timer;

/// How long a fetch may take before its placeholder shows.
const GRACE: Duration = Duration::from_millis(200);

#[component]
fn Card() -> Element {
    let profile = use_resource(load_profile);
    let loading = profile.read().is_none();
    let mut slow = use_signal(|| false);
    // Dropping the timer cancels it, so an unmounted card never writes `slow`.
    let mut grace = use_signal(|| {
        timer().map(|timer| timer.after(GRACE, Box::new(move || slow.set(true))))
    });
    use_drop(move || grace.set(None));

    rsx! {
        div {
            "aria-busy": loading,
            // `opacity`, not `visibility`, which would not hide the grey.
            opacity: if loading && !slow() { "0" } else { "1" },
            Skeleton { visible: loading,
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

The grace runs once, from mount. A card that fetches again starts a new timer
and sets `slow` back to `false`.

## Showing through

A descendant that sets `visibility: visible` on itself shows through, so avoid
one under a visible skeleton.

## Grace period

A fetch that answers in 50 ms should not flash a placeholder. Keep the region
transparent until a grace period ends, and end it with `timer()`. Use `opacity`
for that, since `visibility` would not hide the grey.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `visible` | `bool` | `true` | Hides the children behind the placeholder, or draws the standalone shape. `false` shows the children. |
| `height` | `ThemeAwareValue` | - | A CSS length. Unset, the children's height. |
| `width` | `ThemeAwareValue` | `100%` | A CSS length. Ignored with `circle`. |
| `circle` | `bool` | `false` | A circle as wide as `height`. Without `height`, as wide as the children. |
| `radius` | `ThemeAwareValue` | `sm` | Corner radius, or any CSS, e.g. `radius: "0"`. Ignored with `circle`. |
| `animate` | `bool` | `true` | Runs the pulse. With reduced motion it stops half-way. |
| `children` | `Element` | - | The real content, when the skeleton wraps it. |

Like every component, `Skeleton` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Accessibility

### Libero handles

- A skeleton says nothing to a screen reader.
- Content it hides is not announced and not reachable with Tab.
- With reduced motion the pulse stops half-way.

### You must

- Mark the region you are filling `aria-busy` while it waits, as on
  [Loader](loader.md).
- Avoid a descendant that sets `visibility: visible` on itself under a visible
  skeleton: it shows through.
- Move focus out of the content before setting `visible` back to `true`, or
  use the skeleton for the first load only: covered content is `inert`, so
  focus inside it drops to the page.

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

### Example

A profile card loading, `Skeleton { visible: loading(), .. }` around the name
and bio, inside a region with `aria-busy`: a screen reader says nothing about
the grey shapes, and Tab skips the hidden content until it loads.

## Theme defaults

`SkeletonDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `radius` | `Size` | `Sm`. |
| `color` | `ColorValue` | The grey: `muted.3`. |
| `duration` | `&'static str` | One full pulse: `1500ms`. |
| `animate` | `bool` | Default `animate` when the prop is omitted (`true`). |

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
