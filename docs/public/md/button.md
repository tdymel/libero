# Button

Crate: `libero`
Import: `use libero::components::Button;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/inputs/button.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A clickable action, a toggle, or a router-aware link.

A clickable control, or a router-aware link when `to` is set. A plain
`<button>` submits an enclosing form; ours defaults to `type="button"`
instead, so a submit or reset button says so. `variant` picks the chrome,
`color` the accent, and `selected` turns it into a toggle. The five variants
are Material 3's, in descending emphasis: `filled`, `tonal`, `elevated`,
`outlined`, `text`. For an icon-only button see
[action_icon.md](action_icon.md).

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Button;

#[component]
fn Demo() -> Element {
    rsx! {
        Button {
            color: "primary",
            variant: "filled",
            size: "md",
            radius: "md",
            onclick: move |_| {},
            "Save changes"
        }
    }
}
```

`type` is not a prop but a `<button>` attribute passed straight through, so it
is written `r#type: "submit"` in rsx.

## As a link

Renders as a real anchor, or a router `Link` when `to` matches an internal
route. A link-mode button has no ripple and never calls `onclick`; `disabled`
drops `to` and falls back to `aria-disabled` plus `tabindex="-1"`, since `<a>`
has no native disabled state.

```rust
use dioxus::prelude::*;
use libero::components::Button;

#[component]
fn Demo() -> Element {
    rsx! {
        Button {
            variant: "outlined",
            to: "https://dioxuslabs.com",
            target: "_blank",
            "Open Dioxus docs"
        }
    }
}
```

## Accessibility

A default `Button` is a native `<button>`, so Enter and Space activate it and
it is focusable. `selected: Some(..)` makes it a toggle button and emits
`aria-pressed="true"` / `"false"` - `None` emits neither, which is why an
unset `selected` and `Some(false)` are not the same. `disabled` sets the native
`disabled` attribute on a button and `aria-disabled` on a link. Focus is shown
with a `:focus-visible` ring, so pointer clicks do not draw it. `selected` on a
link keeps the selected look but not `aria-pressed`, and the library warns
about that pairing.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `color` | `ThemeAwareValue` | `primary` | Accent color; a theme color name or a literal CSS color. |
| `variant` | `ButtonVariant` | `filled` | Visual style, in Material 3's descending emphasis order: `filled`, `tonal`, `elevated`, `outlined`, `text`. |
| `radius` | `Size` | `md` | Corner radius, independent of `size`. |
| `size` | `Size` | `md` | Controls height, padding, and font size. |
| `full_width` | `bool` | `false` | Stretches the button to fill its container. |
| `selected` | `bool` | - | Turns the button into a toggle, rendering `aria-pressed` and the selected look. Omit to keep it a plain action. |
| `disabled` | `bool` | `false` | Disables interaction and dims the button. |
| `onclick` | `EventHandler<MouseEvent>` | - | Click handler; not called when the button renders as a link. |
| `to` | `NavigationTarget` | - | Renders as a router-aware link instead of a `<button>`. |
| `target` | `String` | - | The link's `target` attribute, when `to` is set. |
| `children` | `Element` | required | The button's label. |

`Button` also takes the `<button>` HTML attributes (`type`, `form`, `name`,
`value`, ...) and, like every component, the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`ButtonDefaults` on the theme; per-size values live in its `sizes` scale.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted. |
| `radius` | `Size` | Default `radius` when the prop is omitted. |
| `sizes` | `Sizes<ButtonSizeLevel>` | `font_size`, `height`, `padding_x` per size. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-button-font-size-<size>` | `font-size` for that size step. |
| `--lsx-button-height-<size>` | `height` for that size step. |
| `--lsx-button-padding-x-<size>` | Horizontal padding for that size step. |
| `--lsx-button-color` | Accent color of the current variant. |
| `--lsx-button-contrast` | Text color on top of that accent. |
| `--lsx-button-hover` | Accent color while hovered. |
| `--lsx-button-selected` | Background of a selected toggle button. |
| `--lsx-button-container` | Container fill of `tonal`. |
| `--lsx-button-on-container` | Label color on that container - black or white, whichever reads on it. |

The corner radius comes from the shared radius scale, `--lsx-radius-<size>`,
and `elevated`'s shadow from the shared elevation scale, `--lsx-shadow-<size>`.

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `filled` / `tonal` / `elevated` / `outlined` / `text` | The `variant` in effect. |
| `size-<size>` | The `size` in effect. |
| `radius-<size>` | The `radius` in effect. |
| `checked` | `selected` is `Some(true)`. |
| `disabled` | `disabled` is set. |
| `full-width` | `full_width` is set. |
| `ripple-a` / `ripple-b` | A click ripple is playing; the two alternate so the animation restarts. |
