# Pictogram

Crate: `libero`
Import: `use libero::components::{Pictogram, SvgData};`
Feature: `pictogram` (on by default, turns on `use-icon`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/pictogram.rs>
Index: [index.md](index.md) lists every other page
Description: An inline svg drawn from `SvgData`, such as a lucide icon, in `currentColor` and with no size of its own.

Need icons? [pictogram](https://github.com/tdymel/pictogram) ships lucide, Tabler, Material and more.

Draws an `SvgData` glyph as an inline svg. It has no size of its own: `Icon`
and `ActionIcon` size it, or set `width` and `height`. It draws in
`currentColor`, so it takes the text color. Your attributes win over the
glyph's own.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Pictogram;

#[component]
fn Demo() -> Element {
    rsx! {
        Pictogram { icon: pictogram_icons_lucide::house::outlined, width: "32px", height: "32px" }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `icon` | `SvgData` | required | The glyph: a const from a pictogram icon crate, or `SvgData::new(include_str!("x.svg"))`. Take icon crates from pictogram's 0.5 line, the one libero builds on: another minor, such as 0.4, brings a second `SvgData` type, and its icons fail with a type mismatch that does not name the version. |
| `aria_label` | `Option<String>` | `None` | Names the glyph: `role="img"` instead of `aria-hidden`. Leave unset next to a text label. |
| `attributes` | `Vec<Attribute>` | - | Any svg attribute, winning over the glyph's own: `width`, `height`, `stroke_width`, `class`. Others by name in quotes, as `"aria-labelledby": "logo-title"`. Attributes merge by name, a later one replacing an earlier: the defaults (`aria-hidden="true"`, or `role="img"` once named, and `fill="currentColor"` when the glyph sets no `fill`), the glyph's own root attributes, then yours. They are svg attributes, which the stylesheet beats: inside `Icon` or `ActionIcon` the host's CSS still sizes the glyph. |

## Accessibility

### Libero handles

- A pictogram is hidden from screen readers (`aria-hidden="true"`).
- `aria_label`, or an `aria-labelledby` attribute, makes it `role="img"`
  instead.

### You must

- Name a pictogram that means something on its own with `aria_label`. One
  next to a text label stays hidden.
- For a clickable glyph, use `ActionIcon { icon }`.

### Example

A status column with no text, `Pictogram { icon: .., aria_label: "Synced" }`:
a screen reader reads an image named "Synced". The same glyph next to the word
"Synced" stays hidden.
