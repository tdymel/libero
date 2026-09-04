# Anchor

Crate: `libero`
Import: `use libero::components::Anchor;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/anchor.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A real link styled and sized like `Text`, router-aware through `to`.

Text styled and sized like [`Text`](text.md), rendered as a real link -
router-aware via `to`, falling back to a plain `href` when no router is mounted. A
typed route navigates through the app's router, so clicking it is a client-side
navigation, not a full page reload.

It is deliberately not a `Text` with an `href`: `Text` has no href/target/rel
escape hatch, so `Anchor` borrows `Text`'s theme-level sizing and renders its own
`<a>`.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Anchor;

#[component]
fn Demo() -> Element {
    rsx! {
        Anchor { to: "https://dioxuslabs.com", target: "_blank", "Read the Dioxus docs" }
    }
}
```

## Internal routes

Pass a typed route and the link navigates through the router instead of reloading
the page. With a router mounted and `target` unset or `"_blank"`, an internal
target gets SPA navigation; otherwise it degrades to a plain `href`.

```rust
use dioxus::prelude::*;
use libero::components::Anchor;

#[component]
fn Demo() -> Element {
    rsx! {
        Anchor { to: Route::GettingStarted {}, "Back to Getting Started" }
    }
}
```

## Underline

`underline` decides when the rule draws: `hover` (the default) underlines only on
hover, `always` keeps it, `never` drops it. `never` is worth a second thought - an
unadorned link inside a paragraph is only distinguishable by color.

```rust
use dioxus::prelude::*;
use libero::components::Anchor;

#[component]
fn Demo() -> Element {
    rsx! {
        Anchor {
            to: "https://dioxuslabs.com",
            target: "_blank",
            underline: "always",
            size: "lg",
            "Read the Dioxus docs"
        }
    }
}
```

## Accessibility

The link text is the accessible name - keep it descriptive rather than "here".
A `target: "_blank"` link opens a new context, which is worth saying in the
label or a `Tooltip` for anyone who cannot see it happen.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Text size. |
| `to` | `NavigationTarget` | required | A path/URL or a typed route (`Route::Foo {}`). With a router mounted and `target` unset or `"_blank"`, an internal target gets SPA navigation; otherwise a plain `href`. A `javascript:` URL runs script on click and warns in a debug build, so check the scheme of any URL that comes from user data. |
| `target` | `String` | - | The anchor's `target` attribute. |
| `underline` | `AnchorUnderline` | `hover` | When the underline draws: `always`, `hover`, or `never`. |
| `children` | `Element` | required | The link's content. |

Like every component, `Anchor` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`AnchorDefaults` on the theme. Sizing itself comes from `TextDefaults`, so an
`Anchor` and the `Text` around it step together.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted; `md`. |
| `underline` | `AnchorUnderline` | Default `underline`; `hover`. |
| `color` | `Color` | The link color; `primary`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-anchor-color` | The link color, from `AnchorDefaults::color`. |

Font size and family come from `Text`'s own `--lsx-text-*` variables.

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `size-<size>` | The `size` in effect. |
| `underline-always` / `underline-hover` / `underline-never` | The `underline` in effect. |
