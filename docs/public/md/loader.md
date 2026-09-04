# Loader

Crate: `libero`
Import: `use libero::components::Loader;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/feedback/loader.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: An indeterminate busy indicator - a rotating ring, three bars or three dots - always silent; an always-mounted status region says the wait.

An indeterminate busy indicator: it says something is happening, never how much
is left. The root is a `<span>`, so a loader is legal inside a paragraph or a
button. It is always silent - `aria-hidden="true"` - because something else
on screen usually already says what is going on. Under
`prefers-reduced-motion: reduce` all three shapes stop and stay visible: a still
ring with its gap, three full-height bars, three dots.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Loader;

#[component]
fn Demo() -> Element {
    rsx! {
        Loader { variant: "bars", size: "lg", color: "secondary" }
    }
}
```

## Announcing it

Who announces the wait depends on where the loader sits.

| The loader is… | write | the root renders |
|---|---|---|
| beside its own visible text | `Loader {}` | `aria-hidden="true"`, no role - the text is the message |
| inside an already-named control | `Loader {}` | the same; the control's name plus `aria-busy` carries it |
| the only content of a region | `Loader {}`, `aria-busy="true"` on the region, and the text in an always-mounted `role="status"` region outside it | `aria-hidden="true"`; the status region carries it |

```rust
use dioxus::prelude::*;
use libero::components::{Flex, Loader, Text};

#[component]
fn Demo() -> Element {
    rsx! {
        Flex { direction: "row", align: "center", gap: "sm",
            Loader { variant: "dots", size: "sm" }
            Text { "Uploading…" }
        }
    }
}
```

When the loader is the only content of a region, mark the region busy and keep
the loader silent. Say the wait in a `role="status"` region that is always
mounted and sits outside the busy element, and fill it only while loading:

```rust
use dioxus::prelude::*;
use libero::components::{Box, Loader, VisuallyHidden};

#[component]
fn Results() -> Element {
    let results = use_resource(search);
    let loading = results.read().is_none();

    rsx! {
        Box {
            "aria-busy": loading,
            match &*results.read() {
                None => rsx! { Loader {} },
                Some(rows) => rsx! { ResultList { rows: rows.clone() } },
            }
        }
        VisuallyHidden { role: "status", if loading { "Loading results" } }
    }
}
```

Both halves matter. Some screen readers skip a live region that mounts with its
text already in it, and some hold back changes inside an `aria-busy` subtree
until it is no longer busy - by then the loader is gone. `ComboboxCore` renders
its loading status this way.

## Accessibility

- **Always silent**: `aria-hidden="true"`, not `role="presentation"` - a
  `<span>` has no implicit role for `presentation` to strip.
- **No `label` prop.** A loader that is its own status region mounts together
  with its text, which some screen readers do not announce. Use the
  always-mounted status region above.
- **Not focusable, no keyboard contract.**
- **Reduced motion**: every animation stops, and each shape rests on its
  visible end - a bar's keyframes start at `opacity: 0`, so its arm restores
  `transform: scale(1)` and `opacity: 1` rather than only cancelling.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `variant` | `LoaderVariant` | `oval` | The shape: `oval`, `bars` or `dots`. |
| `size` | `Size` | `md` | The square edge, 18px at `xs` to 72px at `xxl`. Every part of the shape is a fraction of it. |
| `color` | `ThemeAwareValue` | `primary` | The ink; a theme color name or a literal CSS color. |

Like every component, `Loader` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Theme defaults

`LoaderDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `variant` | `LoaderVariant` | `Oval`. |
| `size` | `Size` | `Md`. |
| `color` | `Color` | `Primary`. |
| `sizes` | `Sizes<&'static str>` | The edge per step: `18px`, `22px`, `36px`, `44px`, `58px`, `72px`. |

Durations are design constants, not theme fields; override `animation-duration`
through `sx` for the exception.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-loader-size-<size>` | The edge for that step, from `LoaderDefaults`. |
| `--lsx-loader-size` | The active step's edge, republished unsuffixed on the root; the bars and dots inherit it. Override it for an off-scale size. |
| `--lsx-loader-color` | Resolved `color`, per instance. |

## Data attributes

State tokens on the root's `data-state`.

| Token | Condition |
|---|---|
| `oval` / `bars` / `dots` | The `variant` in effect. |
| `size-<size>` | The `size` in effect. |
