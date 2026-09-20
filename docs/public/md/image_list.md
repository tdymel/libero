# ImageList

Crate: `libero`
Import: `use libero::components::{ImageBar, ImageItem, ImageList};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/image_list/image_list.rs>
Index: [index.md](index.md) lists every other page
Description: A gallery of pictures with optional caption bars, laid out on a `GridZone`, so `cols` counts the library's twelve tracks.

A grid of pictures, each with an optional caption bar. It renders a
`<ul role="list">`, so a screen reader announces the count. Each cell is a
`GridItem` of a [GridZone](grid.md), so `cols` and `ImageItem::span` count
twelfths, as the rest of the grid does. `masonry` packs cells on that grid, so
the reading order matches the visual order.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Box, Image, ImageBar, ImageItem, ImageList};
use libero::sx::sx;

#[component]
fn Demo(photos: Vec<Photo>) -> Element {
    rsx! {
        ImageList {
            items: photos
                .iter()
                .map(|photo| {
                    ImageItem::new(rsx! {
                        Image { src: photo.url.clone(), alt: photo.alt.clone(), fit: "cover" }
                    })
                    .bar(ImageBar::new(rsx! {
                        Box { sx: sx().flex("1 1 auto").min_width("0"),
                            Box { sx: sx().font_weight("500"), "{photo.title}" }
                            Box { sx: sx().font_size("0.75rem"), "{photo.author}" }
                        }
                    }))
                })
                .collect(),
        }
    }
}
#
# #[derive(Clone, PartialEq)]
# struct Photo { url: String, alt: String, title: String, author: String }
```

Every prop set:

```rust,ignore
ImageList {
    cols: 3,
    variant: "masonry",
    gap: "sm",
    radius: "md",
    // Ignored by `masonry`, where a cell takes its picture's height.
    ratio: 4.0 / 3.0,
    items: photos,
}
```

A bar at the top, or without its scrim:

```rust,ignore
ImageBar::new(rsx! {
    Box { sx: sx().flex("1 1 auto").min_width("0"), "Breakfast" }
    // A `<button>` inherits no color, and an overlay bar's color comes from
    // the scrim.
    ActionIcon { variant: "standard", sx: sx().color("inherit"), /* .. */ }
})
.position(BarPosition::Top)
.scrim(false)
```

A cell's size, set per item:

```rust,ignore
ImageItem::new(rsx! { Image { src, alt, fit: "cover" } })
    .span(GridSpan::Half) // width, in twelfths
    .rows(2)              // height, in rows, `quilted` only
```

A linked cell:

```rust,ignore
ImageItem::new(rsx! { Image { src, alt: "Breakfast", fit: "cover" } })
    .bar(ImageBar::new(caption))
    .to(Route::Photo { id })
```

One column count per breakpoint:

```rust,ignore
ImageList {
    // One column on a phone, two from `sm` (48rem), four from `md` (62rem).
    cols: responsive(1).sm(2).md(4),
    items: photos,
}
```

## Accessibility

### Libero handles

- Each picture's name is its own `alt`.
- A cell with a bar is a `figure`, and the bar is its caption.
- `ImageItem::to` makes the picture the link and stretches it over the tile, so
  the link's name is the image's `alt`. The bar sits above the link, so a
  control in it still works.
- The scrim never drops below 60% black, so the bar's white text holds at least
  5.7:1 even over a white picture. A `below` bar has no scrim and always reads.

### You must

- Give a linked picture an `alt`: a decorative image leaves the link unnamed.
- Check the contrast of text you dim yourself in a bar: it can still fall
  short.

## Props

### `ImageList`

| Prop | Type | Default | Description |
|---|---|---|---|
| `items` | `Vec<ImageItem>` | `vec![]` | One cell each, in render order. |
| `cols` | `Responsive<u8>` | `2` | Columns, as `cols: 3` or one count per breakpoint, `cols: responsive(1).sm(2).md(4)`. Each count snaps to 1, 2, 3, 4, 6 or 12, since a cell spans twelfths of a `GridZone`. Any other count snaps to the nearest, a tie to the wider cell, and warns. An `ImageItem::span` stays the same at every width. |
| `variant` | `ImageListVariant` | `standard` | `standard` gives every cell the same height, `masonry` keeps each picture's own and packs them, `quilted` lets a cell take more than one row, and `woven` shortens every second cell to 70%. `masonry` measures in the browser, and without a DOM draws an ordinary grid. |
| `gap` | `Size` | `xs` | Between cells. |
| `radius` | `Size` | `sm` | Each cell's corner radius. |
| `ratio` | `f32` | `1.0`, from `theme.aspect_ratio` | Cell aspect ratio, such as `16.0 / 9.0`. Ignored by `masonry`. Under `quilted` it is the ratio of one cell, and a bigger cell scales from it. |

Like every component, `ImageList` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. They land on the `<ul>`.

### `ImageItem`

A builder, like `Table::column()`.

| Method | Type | Description |
|---|---|---|
| `new(content)` | `Element` | The cell's content, usually an `Image` with `fit: "cover"`. |
| `span(span)` | `GridSpan` | This cell's width, in the twelfths a `GridItem` takes. Overrides the one `cols` gives. |
| `rows(rows)` | `u8` | This cell's height in rows. `quilted` only. Other variants ignore it with a warning. |
| `bar(bar)` | `ImageBar` | The caption strip. |
| `to(target)` | `NavigationTarget` | Makes the whole cell a link. Keep everything inside the cell unpositioned, or the hit area stops there. A `zoomable` `Image` in it draws no zoom button and warns. |

### `ImageBar`

| Method | Type | Description |
|---|---|---|
| `new(content)` | `Element` | The strip's content, laid out as a flex row. Give a text block `flex: 1 1 auto; min-width: 0` and a `<button>` `color: inherit`. |
| `position(position)` | `BarPosition` | `bottom`, `top` or `below`, for this cell. |
| `scrim(on)` | `bool` | The dark gradient behind an overlay bar and its light text color. Off leaves a bare transparent strip. A `below` bar has none. |

`ImageItem` holds `Element`s, so the list re-renders whenever its parent does.

## Theme defaults

`ImageListDefaults` on the theme, as `theme.image_list`.

| Field | Type | Default | Description |
|---|---|---|---|
| `cols` | `Responsive<u8>` | `responsive(2)` | Columns when the prop is unset. |
| `variant` | `ImageListVariant` | `Standard` | |
| `gap` | `Size` | `Size::Xs` | |
| `radius` | `Size` | `Size::Sm` | |
| `bar_position` | `BarPosition` | `Bottom` | Used by any bar that names none. |
| `bar_background` | `&'static str` | a gradient from 72% to 60% black | The scrim behind a `bottom` bar. |
| `bar_background_top` | `&'static str` | the same, reversed | The scrim behind a `top` bar. |
| `bar_color` | `&'static str` | `"#fff"` | Bar text, on either scrim. |
| `bar_padding` | `Size` | `Size::Sm` | The bar's inset. |

There is no `ratio` field. Cells take [AspectRatio](aspect_ratio.md)'s
`--lsx-aspect-ratio`, so retuning `theme.aspect_ratio` changes galleries too.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-image-list-bar-background` | The `bottom` scrim. Declared on `:root` from the theme. |
| `--lsx-image-list-bar-background-top` | The `top` scrim. Declared on `:root` from the theme. |
| `--lsx-image-list-bar-color` | Bar text color. Declared on `:root` from the theme. |
| `--lsx-image-list-bar-padding` | The bar's inset, resolved from a spacing step. |
| `--lsx-image-list-radius` | The cell's corner radius, picked on the cell from its own `radius-*` token. |
| `--lsx-aspect-ratio-override` | The cell's `ratio`, over `AspectRatio`'s own pair. Per cell under `quilted`, once on the list otherwise. |
| `--lsx-grid-item-row-span` | `GridItem`'s, published from `ImageItem::rows` under `quilted`. |
| `--lsx-grid-zone-gap` | `GridZone`'s, published from `gap`. |

## Data attributes

| Element | Token | Condition |
|---|---|---|
| `<ul>` | `masonry` | `variant` is `masonry`. `GridZone`'s own token. |
| `<ul>` | `variant-<variant>` | The `variant` in effect. `woven`'s rules key off it. |
| `<li>` | `grid-item`, `span-<span>` | `GridItem`'s own tokens, with the cell's span. |
| `<li>` | `rows` | The cell has an `ImageItem::rows` under `quilted`. |
| `<li>` | `radius-<size>` | The `radius` in effect. |
| media | `variant-<variant>`, `ratio-box` | The `variant` in effect. `ratio-box` on every variant but `masonry`. |
| bar | `bar-bottom` / `bar-top` / `bar-below` | That cell's bar position. |
| bar | `bar-scrim-bottom` / `bar-scrim-top` | An overlay bar that kept its scrim. |
