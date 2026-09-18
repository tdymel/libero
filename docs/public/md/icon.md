# Icon

Crate: `libero`
Import: `use libero::components::Icon;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/icon.rs>
Index: [index.md](index.md) lists every other page
Description: A sized, colored box around an svg, which takes the box's color through `currentColor`.

Wraps an svg in a sized, colored box. An svg drawn in `currentColor` takes the
box's `color`. The box never stretches in a flex row. Any svg works, including
one from an icon crate.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Icon;

#[component]
fn Demo() -> Element {
    rsx! {
        Icon {
            variant: "filled",
            svg {
                view_box: "0 0 24 24",
                fill: "currentColor",
                path { d: "M9 16.2 4.8 12l-1.4 1.4L9 19 21 7l-1.4-1.4z" }
            }
        }
    }
}
```

With `standard`, which draws no container, the svg fills the box. Every other
variant insets it to 60% of the box, clear of the container's edges. Set
`--lsx-icon-glyph` in `sx` to pick another share.

## Accessibility

An icon is hidden from screen readers (`aria-hidden="true"`). One that means
something needs `aria_label` or `aria_labelledby`, which makes it `role="img"`.
A `<title>` inside the svg does not name it, since it is hidden with the rest.
For a clickable icon, use [`ActionIcon`](action_icon.md).

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `component` | `HtmlTag` | `span` | Element to render as. |
| `variant` | `Variant` | `filled` | The look, shared with `Button`. An icon is not interactive, so it has no hover state. |
| `color` | `ThemeAwareValue` | `primary` | The CSS color, which an svg drawn in `currentColor` inherits. Under `filled` a theme color also tints the background. |
| `size` | `ThemeAwareValue` | `md` | Width and height. |
| `radius` | `ThemeAwareValue` | `sm` | Corner radius. |
| `children` | `Element` | required | The svg. |

Like every component, `Icon` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Theme defaults

`IconDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `variant` | `Variant` | Default `variant` when the prop is omitted (`filled`). |
| `sizes` | `Sizes<u16>` | Width and height in px per size step, `16, 20, 24, 32, 40, 48` by default. |

The `color`, `size` and `radius` fallbacks (`primary`, `md`, `sm`) are not
theme fields.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-icon-size-<size>` | Width/height for that size step, from `IconDefaults`. |
| `--lsx-icon-size-override` | Set by the `size` prop; wins over the size step. |
| `--lsx-icon-color` | Resolved `color`; the svg inherits it as `currentColor`. |
| `--lsx-icon-contrast` | Text color on top of that accent, for variant `filled`. Unset for a literal CSS color. |
| `--lsx-icon-radius` | Set by the `radius` prop; falls back to `--lsx-radius-sm`. |
| `--lsx-icon-container` | Container fill of `tonal`. |
| `--lsx-icon-on-container` | Label color on that container, black or white, whichever reads on it. |

## Data attributes

State tokens on the root's `data-state`.

| Token | Condition |
|---|---|
| `filled` / `tonal` / `elevated` / `outlined` / `standard` | The `variant` in effect. |
