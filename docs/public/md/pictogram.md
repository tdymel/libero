# Pictogram

Crate: `libero`
Import: `use libero::components::{Pictogram, SvgData};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/pictogram.rs>
Index: [index.md](index.md) lists every other page
Description: An inline svg drawn from `SvgData`, such as a lucide icon, in `currentColor` and with no size of its own.

Need icons? [pictogram](https://github.com/tdymel/pictogram) ships lucide, Tabler, Material and more.

Draws an `SvgData` glyph as an inline svg. It has no size of its own: `Icon`
and `ActionIcon` size it, or set `width` and `height`. It draws in
`currentColor`, so it takes the text color. Your attributes win over the
glyph's own.

`SvgData` is pictogram's `Svg` type, so every icon of a pictogram icon crate
passes straight in: `pictogram_icons_lucide::house::outlined`. For your own
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
        h2 { id: "logo-title", "Acme" }
        Pictogram { icon: LOGO, width: "32px", height: "32px", "aria-labelledby": "logo-title" }
    }
}
```

`aria_label` names the glyph; any other aria attribute goes by its name in
quotes, as `"aria-labelledby"` above.

Attributes merge by name, a later one replacing an earlier one:

1. the defaults: `aria-hidden="true"` (or `role="img"` once named), and
   `fill="currentColor"` when the glyph sets no `fill`;
2. the glyph's own root attributes (lucide's `stroke`, `stroke-width`, ...);
3. yours.

`stroke_width: "3"` replaces the glyph's attribute. `width`, `height`, `fill`,
`stroke` and the other svg props are svg attributes, which the stylesheet
beats: inside `Icon` or `ActionIcon` the host's CSS still sizes the glyph.

## Accessibility

### Libero handles

- A pictogram is hidden from screen readers (`aria-hidden="true"`).
- `aria_label`, or `"aria-labelledby"` by its name in quotes, makes it
  `role="img"` instead.
- A leading `<title>` in the glyph (lobe and simple icons carry one) is
  dropped: it would add a hover tooltip and stray words to the host's text.
  `aria_label` is the name.

### You must

- Name a pictogram that means something on its own with `aria_label`. One
  next to a text label stays hidden.
- For a clickable glyph, use `ActionIcon { icon }`.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `icon` | `SvgData` | required | The glyph: a const from a pictogram icon crate, or `SvgData::new(include_str!("x.svg"))`. |
| `aria_label` | `Option<String>` | `None` | Names the glyph: `role="img"` instead of `aria-hidden`. Leave unset next to a text label. |

`Pictogram` also takes every svg attribute (`stroke_width`, `width`, `class`,
...), and any other attribute by its name in quotes (`"aria-labelledby": "logo-title"`).
