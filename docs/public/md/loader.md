# Loader

Crate: `libero`
Import: `use libero::components::Loader;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/feedback/loader.rs>
Index: [index.md](index.md) lists every other page
Description: An indeterminate busy indicator, as a ring, bars or dots. It stays silent, so a status region says the wait. `Button`, `Combobox` and `FileField` show it while loading.

An indeterminate busy indicator. It says something is happening, not how much
is left. The root is a `<span>`, so it fits inside a paragraph or a button.
With reduced motion it stops moving but stays visible.

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

## Accessibility

### Libero handles

- The loader is hidden from screen readers.
- With reduced motion it stops moving but stays visible.

### You must

- Say the wait with something else. Beside its own text, the text says it.
  Inside a control, such as a `Button` with `loading`, the control does.
- As the only content of a region, mark the region `aria-busy` and put the
  text in a `role="status"` region outside it.
- Mount that status region up front and fill it only while loading, or a
  screen reader may skip it.

Beside its own text:

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

As the only content of a region:

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
#
# async fn search() -> Vec<String> { Vec::new() }
# #[component] fn ResultList(rows: Vec<String>) -> Element { rsx! {} }
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `variant` | `LoaderVariant` | `oval` | The shape: `oval`, `bars` or `dots`. |
| `size` | `Size` | `md` | The edge of the square, 18px at `xs` to 72px at `xxl`. |
| `color` | `ThemeAwareValue` | `primary` | A theme color name or any CSS color. |

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

Durations are fixed. Override `animation-duration` through `sx` to change one.

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
