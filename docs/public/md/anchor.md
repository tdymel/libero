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
        Anchor { to: "https://dioxuslabs.com", target: "_blank", "Read the Dioxus docs" }
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

## Accessibility

### Libero handles

- A `target: "_blank"` link draws a small external icon and reads a hidden
  "(opens in a new tab)". `new_tab_hint: false` drops both, for a link whose
  text already says it.

### You must

- Make the link text say where the link goes: it is the accessible name.
- Keep the underline on a link inside a paragraph: with `underline: "never"`
  it stands out by color alone.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Text size. |
| `to` | `NavigationTarget` | required | A path, a URL or a typed route (`Route::Foo {}`). With a router mounted and `target` unset or `"_blank"`, an internal target navigates without a page reload. A `javascript:` URL runs script on click, so check the scheme of any URL from user data. |
| `target` | `String` | - | The link's `target` attribute. `"_blank"` adds a small external icon and a hidden "(opens in a new tab)". |
| `new_tab_hint` | `bool` | `true` | `false` drops the icon and the hidden text a `"_blank"` target adds. |
| `underline` | `AnchorUnderline` | `hover` | When the underline draws, `always`, `hover` or `never`. |
| `children` | `Element` | required | The link's content. |

Like every component, `Anchor` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`AnchorDefaults` on the theme. Sizing itself comes from `TextDefaults`, so an
`Anchor` and the `Text` around it step together.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size`, `md`. |
| `underline` | `AnchorUnderline` | Default `underline`, `hover`. |
| `color` | `Color` | The link color, `primary`. |

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
