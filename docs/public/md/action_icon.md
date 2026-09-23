# ActionIcon

Crate: `libero`
Import: `use libero::components::ActionIcon;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/buttons/action_icon.rs>
Index: [index.md](index.md) lists every other page
Description: An icon-only button, rendered as a `button` or a link, with a required `aria_label`.

Need icons? [pictogram](https://github.com/tdymel/pictogram) ships lucide, Tabler, Material and more.

An icon-only button for actions like copy, close or delete. It renders a
`<button>`, or a link when `to` is set. `aria_label` is required, because the
icon gives a screen reader nothing to read.

With neither `variant` nor `color` set, it has no background of its own and
takes the surrounding text color.

## Usage

`icon` takes a glyph as `SvgData`, drawn as a [`Pictogram`](pictogram.md).
Any other content, such as an svg component of yours, goes in as children.

```rust
use dioxus::prelude::*;
use libero::components::ActionIcon;

#[component]
fn Demo() -> Element {
    rsx! {
        ActionIcon { size: "md", radius: "sm", aria_label: "Confirm",
            icon: pictogram_icons_lucide::check::outlined,
        }
        ActionIcon { aria_label: "Done", CheckmarkIcon {} }
    }
}
#
# #[component] fn CheckmarkIcon() -> Element { rsx! {} }
```

With both `icon` and `children` set, `icon` wins and a debug build warns.

## Accessibility

### Libero handles

- Below 24px (a length such as `"20px"`) the button still takes presses in a
  24x24 box centred on it.
- `focusable_when_disabled` keeps a disabled button in the Tab order, with
  `aria-disabled`.

### You must

- Name the button with `aria_label`: the icon gives a screen reader nothing to
  read.
- Below 24px, keep other targets clear of that 24x24 box, or the one drawn
  later takes the overlap.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `variant` | `Variant` | - | Visual style, shared with `Button`: `filled`, `tonal`, `elevated`, `outlined`, `standard`. With `color` also unset, the button takes the surrounding text color. |
| `color` | `ThemeAwareValue` | - | Accent color. A theme color name or any CSS color. Set alone, it gives the theme's default variant, `filled`. Under a gradient, its first stop. |
| `gradient` | `Gradient` | - | With `variant: "gradient"`: the second stop and the angle, as `("info", 90)` or `Gradient::default().to("info").deg(90)`. The first stop is `color`. Ignored by the other variants. |
| `size` | `ThemeAwareValue` | `md` | A size word takes `Button`'s height at that step, so the two line up in a row, and sizes the icon inside as `Icon`'s. A length such as `"20px"` sizes the box, and the icon fills it. |
| `radius` | `ThemeAwareValue` | `sm` | Corner radius, independent of `size`. |
| `aria_label` | `String` | required | The button's accessible name. |
| `selected` | `bool` | - | Makes it a toggle button. The selected look shows once `variant` or `color` is set. Leave it unset for a plain action. |
| `disabled` | `bool` | `false` | Disables and dims the button. |
| `focusable_when_disabled` | `bool` | `false` | With `disabled`: keeps the button in the Tab order. It renders `aria-disabled` rather than `disabled` and ignores presses. |
| `loading` | `bool` | `false` | Shows a `Loader` over the icon and ignores clicks. The button stays focusable. Ignored on a link. |
| `onclick` | `EventHandler<MouseEvent>` | - | Click handler. Not called on a link. |
| `to` | `NavigationTarget` | - | Renders a link instead of a `<button>`. |
| `target` | `String` | - | The link's `target` attribute. |
| `icon` | `SvgData` | - | The glyph, drawn as a `Pictogram`: `pictogram_icons_lucide::x::outlined`. Wins over `children`. |
| `children` | `Element` | - | The icon, when it is not `SvgData`: your own svg, or any element. |

Like every component, `ActionIcon` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`ActionIconDefaults` on the theme; the size resolves through `Button`'s height
scale, the icon inside through the shared icon size scale, and the radius
through the theme's radius scale.

| Field | Type | Description |
|---|---|---|
| `variant` | `Variant` | Default `variant` when the prop is omitted (`filled`). |
| `size` | `Size` | Default `size` when the prop is omitted. |
| `radius` | `Size` | Default `radius` when the prop is omitted. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-action-icon-size` | Button size, from the theme. |
| `--lsx-action-icon-size-override` | The `size` prop, set per instance. |
| `--lsx-action-icon-glyph` | The icon's size inside the button, from the theme. |
| `--lsx-action-icon-glyph-override` | The icon's size for the `size` prop, set per instance. |
| `--lsx-action-icon-radius` | Corner radius, from the theme. |
| `--lsx-action-icon-radius-override` | The `radius` prop, set per instance. |
| `--lsx-action-icon-color` | Accent color of the current variant. Only set when the button has variant styling. |
| `--lsx-action-icon-contrast` | Text color on top of that accent. |
| `--lsx-action-icon-hover` | Accent color while hovered. |
| `--lsx-action-icon-container` | Container fill of `tonal`. |
| `--lsx-action-icon-on-container` | Label color on that container, black or white, whichever reads on it. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `filled` / `tonal` / `elevated` / `outlined` / `standard` | The `variant` in effect, written only when `variant` or `color` is set. |
| `checked` | `selected` is `true`. |
| `disabled` | `disabled` is set. |
| `loading` | `loading` is set, on a button. |
| `ripple-a` / `ripple-b` | A click is showing its ripple; the two tokens alternate so consecutive clicks restart the animation. |
