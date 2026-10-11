# Anchor

Crate: `libero`
Import: `use libero::components::Anchor;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/anchor.rs>
Index: [index.md](index.md) lists every other page
Description: A real link styled and sized like `Text`, router-aware through `to`.

A link styled and sized like [`Text`](text.md). A typed route in `to` navigates
through the app's router without a page reload. Without a router it renders a
plain `href`.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Anchor;

#[component]
fn Demo() -> Element {
    rsx! {
        Anchor {
            to: "https://dioxuslabs.com",
            target: "_blank",
            new_tab_icon: true,
            "Read the Dioxus docs"
        }
    }
}
```

A typed route in `to` navigates through the router.

```rust
use dioxus::prelude::*;
use libero::components::Anchor;

#[component]
fn Demo() -> Element {
    rsx! {
        Anchor { to: Route::GettingStarted {}, "Back to Getting Started" }
    }
}
#
# #[derive(Clone, PartialEq, Routable)]
# enum Route {
#     #[route("/0")]
#     GettingStarted {},
# }
# #[component] fn GettingStarted() -> Element { rsx! {} }
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Text size. |
| `to` | `NavigationTarget` | required | A path, a URL or a typed route (`Route::Foo {}`). With a router mounted and `target` unset, an internal target navigates without a page reload; with `"_blank"` it opens a new tab with a full page load. A `javascript:` URL runs script on click, so check the scheme of any URL from user data. |
| `target` | `String` | - | The link's `target` attribute. `"_blank"` adds a hidden "(opens in a new tab)". |
| `new_tab_hint` | `bool` | `true` | `false` drops the hidden text a `"_blank"` target adds, and the icon. |
| `new_tab_icon` | `bool` | `false` | With the hint, draws a small external icon after the text. Unset, the theme's `anchor.new_tab_icon`, which is off. `Button`, `ActionIcon`, `Chip`, `NavLink`, `Repository` and `MenuItem` take it too. |
| `underline` | `AnchorUnderline` | `hover` | When the underline draws, `always`, `hover` or `never`. |
| `children` | `Element` | required | The link's content. |
| `parts` | `Parts<AnchorPart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |

Like every component, `Anchor` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Styling](styling.md#style-api) explains how
parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `AnchorPart::NewTab` | `new-tab` | The new-tab icon after the text, with `target: "_blank"` and `new_tab_icon` only. |

## Accessibility

### Libero handles

- A `target: "_blank"` link reads a hidden "(opens in a new tab)".
  `new_tab_icon: true` also draws a small external icon, for sighted users.
  `new_tab_hint: false` drops both, for a link whose text already says it.
- In a filled, tonal or gradient `Alert`, a colored `Header` and a `Mark`,
  a link takes the text color, so `underline: "hover"` draws its underline at
  rest there too.

### You must

- Make the link text say where the link goes: it is the accessible name.
- Set `underline: "always"` on a link inside a paragraph: the default `hover`
  draws it only on hover and `never` not at all, so at rest the link stands out
  by color alone.

### Example

A link in a paragraph, `Anchor { to: "https://www.rust-lang.org", target:
"_blank", underline: "always", "The Rust website" }`: it keeps its underline,
and a screen reader reads "The Rust website (opens in a new tab)".

## Theme defaults

`AnchorDefaults` on the theme. Sizing itself comes from `TextDefaults`, so an
`Anchor` and the `Text` around it step together.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size`, `md`. |
| `underline` | `AnchorUnderline` | Default `underline`, `hover`. |
| `color` | `Color` | The link color, `primary`. |
| `new_tab_icon` | `bool` | Default `new_tab_icon` of `Anchor`, `Button`, `ActionIcon`, `Chip`, `NavLink`, `Repository` and `MenuItem`, `false`. |

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
