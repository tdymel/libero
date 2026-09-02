# Loader

Crate: `libero`
Import: `use libero::components::Loader;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/feedback/loader.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: An indeterminate busy indicator - a rotating ring, three bars or three dots - silent unless it is the only content of a region.

An indeterminate busy indicator: it says something is happening, never how much
is left. The root is a `<span>`, so a loader is legal inside a paragraph or a
button. It is silent by default - `aria-hidden="true"` - because something else
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
| the only content of a region | `Loader { label: "Loading results" }`, and `aria-busy="true"` on the region | `role="status"` and a visually hidden text node |

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

When the loader is the only content of a region, give it a `label` and mark the
region busy:

```rust
use dioxus::prelude::*;
use libero::components::{Box, Loader};

#[component]
fn Results() -> Element {
    let results = use_resource(search);

    rsx! {
        Box {
            "aria-busy": results.read().is_none(),
            match &*results.read() {
                None => rsx! { Loader { label: "Loading results" } },
                Some(rows) => rsx! { ResultList { rows: rows.clone() } },
            }
        }
    }
}
```

## Accessibility

- **Silent by default**: `aria-hidden="true"`, not `role="presentation"` - a
  `<span>` has no implicit role for `presentation` to strip.
- **`label` makes a live region.** The label is a visually hidden text node,
  not `aria-label`: a live region announces its content, so naming it announces
  nothing. `role="status"` implies `aria-live="polite"` and
  `aria-atomic="true"`, which are not restated.
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
| `label` | `Option<String>` | `None` | Makes the loader a `role="status"` live region announcing this text. Only for a loader that is the sole content of the area that is loading. |

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
