# Button

Crate: `libero`
Import: `use libero::components::Button;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/buttons/button.rs>
Index: [index.md](index.md) lists every other page
Description: A clickable action, a toggle, or a router-aware link.

A clickable action, a toggle, or a link when `to` is set. It defaults to
`type="button"`, so it never submits a form by accident. A submit button sets
`r#type: "submit"`. For an icon-only button, use [ActionIcon](action_icon.md).

A `Button` shows a label, with an optional `icon` before it. `ActionIcon` is the
same button reduced to a square icon: it requires an `aria_label`, keeps a 24px
target and defaults to no background. A `Button` with no text would be a wide
pill without an accessible name.

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

## Accessibility

### Libero handles

- A label cut at the edge is still the full accessible name.
- `selected: Some(false)` announces a toggle that is off. An unset `selected`
  announces no state.
- `focusable_when_disabled` keeps a disabled button in the Tab order, with
  `aria-disabled`.

### You must

- Pass a label that may be cut as `title` too, so sighted users can read it on
  hover.

```rust
use dioxus::prelude::*;
use libero::components::Button;

#[component]
fn Demo() -> Element {
    let label = "Download the quarterly report as a spreadsheet";
    rsx! {
        Button { title: label, "{label}" }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `color` | `ThemeAwareValue` | `primary` | Accent color. A theme color name or any CSS color. Under a gradient, its first stop. |
| `variant` | `Variant` | `filled` | Visual style, from most to least emphasis: `filled`, `tonal`, `elevated`, `outlined`, `standard`. `gradient` fills it from `color` into the theme's second stop. |
| `gradient` | `Gradient` | - | With `variant: "gradient"`: the second stop and the angle, as `("info", 90)` or `Gradient::default().to("info").deg(90)`. The first stop is `color`. Ignored by the other variants. |
| `radius` | `Size` | `md` | Corner radius, independent of `size`. |
| `size` | `Size` | `md` | Height, padding and font size. |
| `full_width` | `bool` | `false` | Stretches the button to fill its container. |
| `selected` | `bool` | - | Makes it a toggle button with the selected look. Leave it unset for a plain action. |
| `disabled` | `bool` | `false` | Disables and dims the button. |
| `focusable_when_disabled` | `bool` | `false` | With `disabled`: keeps the button in the Tab order. It renders `aria-disabled` rather than `disabled` and ignores presses. |
| `loading` | `bool` | `false` | Shows a `Loader` over the label and ignores clicks. The button stays focusable and keeps its width. Ignored on a link. |
| `onclick` | `EventHandler<MouseEvent>` | - | Click handler. Not called on a link. |
| `to` | `NavigationTarget` | - | Renders a link instead of a `<button>`. Takes a path, a URL or a typed route (`Route::Foo {}`). |
| `target` | `String` | - | The link's `target` attribute. |
| `icon` | `Element` | - | Drawn before the label. It never shrinks. |
| `children` | `Element` | required | The label, on one line. A long one is cut at the edge. |

`Button` also takes the `<button>` HTML attributes (`type`, `form`, `name`,
`value`, ...) and, like every component, the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`ButtonDefaults` on the theme; per-size values live in its `sizes` scale.

| Field | Type | Description |
|---|---|---|
| `variant` | `Variant` | Default `variant` when the prop is omitted (`filled`). |
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
| `--lsx-button-on-container` | Label color on that container, black or white, whichever reads on it. |

The corner radius comes from the shared radius scale, `--lsx-radius-<size>`,
and `elevated`'s shadow from the shared elevation scale, `--lsx-shadow-<size>`.

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `filled` / `tonal` / `elevated` / `outlined` / `standard` | The `variant` in effect. |
| `size-<size>` | The `size` in effect. |
| `radius-<size>` | The `radius` in effect. |
| `checked` | `selected` is `Some(true)`. |
| `disabled` | `disabled` is set. |
| `loading` | `loading` is set, on a `<button>`. |
| `full-width` | `full_width` is set. |
| `ripple-a` / `ripple-b` | A click ripple is playing; the two alternate so the animation restarts. |
