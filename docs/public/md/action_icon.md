# ActionIcon

Crate: `libero`
Import: `use libero::components::ActionIcon;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/inputs/action_icon.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: An icon-only button - `Icon`'s sizing, color and variant system rendered as a real `button` (or a link), with a required `aria_label`.

`Icon`'s sizing, color, and variant system, rendered as a real `<button>` with
click handling and required a11y - for icon-only actions like a copy, close, or
delete button. `aria_label` is required, not optional: an icon-only button has no
visible text for a screen reader to announce.

With neither `variant` nor `color` set, it contributes no background or color of
its own and inherits the surrounding text color, rather than defaulting to a
filled badge the way `Icon` does. That is how `Code`'s own copy button is built.

## Usage

`CheckmarkIcon` below stands for any component of your own that renders an
`svg` - `ActionIcon` only sizes and colors what it wraps.

```rust
use dioxus::prelude::*;
use libero::components::ActionIcon;

#[component]
fn Demo() -> Element {
    rsx! {
        ActionIcon { size: "md", radius: "sm", aria_label: "Confirm",
            CheckmarkIcon {}
        }
    }
}
```

With a variant and a color, it draws a badge of its own:

```rust
use dioxus::prelude::*;
use libero::components::ActionIcon;

#[component]
fn Demo() -> Element {
    rsx! {
        ActionIcon {
            variant: "filled",
            color: "primary",
            aria_label: "Confirm",
            onclick: move |_| {},
            CheckmarkIcon {}
        }
    }
}
```

## As a link

Renders as a real anchor, or a router `Link` when `to` matches an internal route.

```rust
use dioxus::prelude::*;
use libero::components::ActionIcon;

#[component]
fn Demo() -> Element {
    rsx! {
        ActionIcon {
            variant: "outlined",
            color: "primary",
            to: "https://dioxuslabs.com",
            target: "_blank",
            aria_label: "Open Dioxus docs",
            CheckmarkIcon {}
        }
    }
}
```

## Accessibility

`aria_label` is a required prop, because the button's only content is an `svg`.
As a button it renders a real `<button type="button">`, so Enter and Space
activate it, and `disabled` renders the native attribute. As a link it is an
`<a>`, which has no native `disabled`: a disabled link therefore drops `to` and
gets `aria-disabled="true"` plus `tabindex="-1"` so it leaves the tab order.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `variant` | `ButtonVariant` | - | Chrome, shared with `Button`: `filled`, `tonal`, `elevated`, `outlined`, `standard`. Unset, with `color` also unset, the button contributes no background or color of its own and inherits the surrounding text color. |
| `color` | `ThemeAwareValue` | - | Accent color; a theme color name or a literal CSS color. Setting it turns on variant styling even if `variant` itself is unset (as `filled`). |
| `size` | `ThemeAwareValue` | `md` | Button size, independent of the wrapped icon's own size. |
| `radius` | `ThemeAwareValue` | `sm` | Corner radius, independent of size. |
| `aria_label` | `String` | required | An icon-only button has no visible text for a screen reader to announce. |
| `disabled` | `bool` | `false` | Disables interaction and dims the button. |
| `onclick` | `EventHandler<MouseEvent>` | - | Click handler; not called when the button renders as a link. |
| `to` | `NavigationTarget` | - | Renders as a router-aware link instead of a `<button>`. |
| `target` | `String` | - | The link's `target` attribute, when `to` is set. |
| `children` | `Element` | required | The icon to show. |

Like every component, `ActionIcon` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`ActionIconDefaults` on the theme; the size resolves through the shared icon
size scale and the radius through the theme's radius scale.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted. |
| `radius` | `Size` | Default `radius` when the prop is omitted. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-action-icon-size` | Button size, from the theme. |
| `--lsx-action-icon-size-override` | The `size` prop, set per instance. |
| `--lsx-action-icon-radius` | Corner radius, from the theme. |
| `--lsx-action-icon-radius-override` | The `radius` prop, set per instance. |
| `--lsx-action-icon-color` | Accent color of the current variant. Only set when the button has variant styling. |
| `--lsx-action-icon-contrast` | Text color on top of that accent. |
| `--lsx-action-icon-hover` | Accent color while hovered. |
| `--lsx-action-icon-container` | Container fill of `tonal`. |
| `--lsx-action-icon-on-container` | Label color on that container - black or white, whichever reads on it. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `filled` / `tonal` / `elevated` / `outlined` / `standard` | The `variant` in effect - written only when `variant` or `color` is set. |
| `disabled` | `disabled` is set. |
| `ripple-a` / `ripple-b` | A click is showing its ripple; the two tokens alternate so consecutive clicks restart the animation. |
