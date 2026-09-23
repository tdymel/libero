# Pictogram

Crate: `libero`
Import: `use libero::components::{Pictogram, SvgData};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/pictogram.rs>
Index: [index.md](index.md) lists every other page
Description: An inline svg drawn from `SvgData`, such as a lucide icon, in `currentColor` and with no size of its own.

Draws an `SvgData` glyph as an inline svg. It has no size of its own: `Icon`
and `ActionIcon` size it, or set `width` and `height`. It draws in
`currentColor`, so it takes the text color. Your attributes win over the
glyph's own.

`SvgData` is [pictogram](https://github.com/tdymel/pictogram)'s `Svg` type, so
every icon of a pictogram icon crate passes straight in:
`pictogram_icons_lucide::house::outlined`. libero depends on pictogram 0.4; an
icon crate of another 0.x version has its own, different type. For your own
glyph, `SvgData::new(include_str!("logo.svg"))` splits the file at compile time.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Pictogram, SvgData};

const LOGO: SvgData = SvgData::new(
    r#"<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"/></svg>"#,
);

#[component]
fn Demo() -> Element {
    rsx! {
        Pictogram { icon: pictogram_icons_lucide::house::outlined, width: "32px", height: "32px" }
        Pictogram { icon: LOGO, width: "32px", height: "32px", aria_label: "Acme" }
    }
}
```

Attributes merge by name, a later one replacing an earlier one:

1. the defaults: `aria-hidden="true"` (or `role="img"` once named), and
   `fill="currentColor"` when the glyph sets no `fill`;
2. the glyph's own root attributes (lucide's `stroke`, `stroke-width`, ...);
3. yours.

`"stroke-width": "3"` replaces the glyph's attribute. `width`, `height`,
`color` and the other shared HTML props are CSS properties in `style`, which
win over the stylesheet.

## Accessibility

### Libero handles

- A pictogram is hidden from screen readers (`aria-hidden="true"`).
- `aria_label` or `aria_labelledby` makes it `role="img"` instead.

### You must

- Name a pictogram that means something on its own with `aria_label`. One next
  to a text label stays hidden.
- For a clickable glyph, use `ActionIcon { icon }`.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `icon` | `SvgData` | required | The glyph: a const from a pictogram icon crate, or `SvgData::new(include_str!("x.svg"))`. |

`Pictogram` also takes any extra HTML attributes, and attribute names in
quotes (`"stroke-width": "3"`) as svg attributes.
