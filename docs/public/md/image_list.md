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
                            Box { sx: sx().font_size("0.75rem").opacity("0.72"), "{photo.author}" }
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

## The bar

`ImageBar::new` takes any `Element`. The bar sets where the strip sits and the
scrim behind it.

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

The bar is a flex row. Give the text block `flex: 1 1 auto` and `min-width: 0`,
or a long title pushes the control out of the strip, and give any `<button>` in
an overlay bar `color: inherit`. The bar sits above a linked cell's hit area, so
a control in it still works. `.scrim(false)` drops the gradient and the light
text color and leaves a bare transparent strip. A `below` bar has no scrim.

## Spans and rows

A cell's size is set per item:

```rust,ignore
ImageItem::new(rsx! { Image { src, alt, fit: "cover" } })
    .span(GridSpan::Half) // width, in twelfths
    .rows(2)              // height, in rows, `quilted` only
```

`span` overrides the width `cols` gives. `rows` works only under `quilted`.
Other variants ignore it with a warning, since `standard` has one row per cell
and `masonry` takes the height from the picture.

## Linking a cell

`ImageItem::to` makes the whole tile a link. The picture is the anchor, and its
hit area stretches over the cell.

```rust,ignore
ImageItem::new(rsx! { Image { src, alt: "Breakfast", fit: "cover" } })
    .bar(ImageBar::new(caption))
    .to(Route::Photo { id })
```

The stretched hit area needs nothing between the anchor and the `<li>` to be
positioned. If you style a cell through `sx`, keep everything inside it
unpositioned. The link wins over a `zoomable` `Image`, which draws no zoom
button in a linked cell and warns in a debug build. Zoom on the page the link
opens.

## Columns

`cols` snaps to 1, 2, 3, 4, 6 or 12, since a cell spans twelfths. Any other
count snaps to the nearest and warns. A tie goes to the wider cell, so 5
becomes 4. `cols` also takes one count per breakpoint, mobile first:

```rust,ignore
ImageList {
    // One column on a phone, two from `sm` (48rem), four from `md` (62rem).
    cols: responsive(1).sm(2).md(4),
    items: photos,
}
```

Each count snaps on its own. The breakpoints are plain media queries, so server
rendering gets them right. An `ImageItem::span` stays the same at every width.
The default is 2, which still works on a phone.

## Variants

| Variant | What it does |
|---|---|
| `standard` | Every cell takes the list's `ratio`, so the rows are even. |
| `masonry` | Every cell keeps its picture's height, and the cells pack with no gaps. |
| `quilted` | `standard`, plus `ImageItem::rows`, so a cell may take more than one row. |
| `woven` | `standard`, with every second cell shortened to 70% and centred. It crops every second picture. |

`masonry` measures each cell in the browser. Without a DOM, as during server
rendering, the gallery renders as an ordinary grid. Under `quilted` a 2x2 cell
is exactly twice the size of a 1x1 one, since its ratio scales with its span
([AspectRatio](aspect_ratio.md)).

Not implemented: a row height, an action position, a raw pixel `gap`, and the
compound `ImageList { ImageListItem {} }` children API.

## Accessibility

Each picture's name is its own `alt`. A cell with a bar is a `figure`, and the
bar is its `figcaption`. The caption may repeat the `alt`. In a linked cell the
link's name is the image's `alt`, so a decorative image leaves it unnamed. Give
it a real `alt`, or put a link of your own in the bar.

The scrim is a gradient, and the backdrop is your picture. Over a white picture,
white text on the default scrim is 9.3:1 at the bottom edge, 4.2:1 at 40% up,
2.5:1 at the middle and 1.8:1 near the top. Over a mid-grey picture the same
points are 15.5, 10.7 and 7.8:1. A second caption line over a bright picture is
the case that fails. Use a `below` bar, add a `text-shadow` through `sx`, or
make the scrim darker in the theme.

## Props

### `ImageList`

| Prop | Type | Default | Description |
|---|---|---|---|
| `items` | `Vec<ImageItem>` | `vec![]` | One cell each, in render order. |
| `cols` | `Responsive<u8>` | `2` | Columns, as `cols: 3` or one count per breakpoint, `cols: responsive(1).sm(2).md(4)`. Each count snaps to 1, 2, 3, 4, 6 or 12. |
| `variant` | `ImageListVariant` | `standard` | `standard`, `masonry`, `quilted` or `woven`. |
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
| `to(target)` | `NavigationTarget` | Makes the whole cell a link. A `zoomable` `Image` in it draws no zoom button and warns. |

### `ImageBar`

| Method | Type | Description |
|---|---|---|
| `new(content)` | `Element` | The strip's content, laid out as a flex row. |
| `position(position)` | `BarPosition` | `bottom`, `top` or `below`, for this cell. |
| `scrim(on)` | `bool` | The gradient behind an overlay bar and its light text color. On by default. |

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
| `bar_background` | `&'static str` | a black-to-transparent gradient | The scrim behind a `bottom` bar. |
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
