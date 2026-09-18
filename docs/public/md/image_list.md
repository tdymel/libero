# ImageList

Crate: `libero`
Import: `use libero::components::{ImageBar, ImageItem, ImageList};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/image_list/image_list.rs>
Index: [index.md](index.md) lists every other page
Description: A gallery of pictures with optional caption bars, rendered as a `ul`/`li` list over a `GridZone` - so `cols` is a span of the library's own twelve tracks and `masonry` is that zone's measuring engine.

A grid of pictures, each with an optional caption bar. Renders a
`<ul role="list">` of `<li>`s, so a gallery is announced with a count - and every
cell is a `GridItem` of a [GridZone](grid.md), so `cols` and `ImageItem::span`
are spans of the library's own twelve tracks rather than a second grid with its
own track count.

`masonry` is that zone's measuring engine, not a CSS multi-column - so the
reading order and the visual order agree. Each picture's accessible name is its
own `alt`; the bar never becomes one. A cell with a bar is a `figure` and the
bar its `figcaption`.

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
    // Ignored by `masonry`: a packed cell takes its height from its picture.
    ratio: 4.0 / 3.0,
    items: photos,
}
```

## The bar holds an Element

`ImageBar::new` takes whatever you render. There is no title, subtitle or action
slot: a caption is content, and a component that shapes it is only ever in the
way of the next design. `ImageBar` keeps the two things the *cell* owns - where
the strip sits, and the scrim behind it.

```rust,ignore
ImageBar::new(rsx! {
    Box { sx: sx().flex("1 1 auto").min_width("0"), "Breakfast" }
    // A `<button>` inherits no color, and an overlay bar's color comes from
    // the scrim - so say so. The same line MultiSelect's remove button carries.
    ActionIcon { variant: "standard", sx: sx().color("inherit"), /* .. */ }
})
.position(BarPosition::Top)
.scrim(false)
```

Three things to copy from that snippet:

- **`flex: 1 1 auto` and `min-width: 0`** on the text block. The bar is a flex
  row; without them a long title pushes the control out of the strip.
- **`color: inherit`** on any `<button>` in an overlay bar.
- **The bar is above the cell's stretched link** (`z-index: 1`, with no
  `position` - a grid item takes a z-index either way, and positioning it would
  break the link, see below). So a control in it works on a linked cell, and the
  bar strip is not part of the link's hit area.

`.scrim(false)` turns off the gradient *and* the light text color, which hands
you a bare transparent strip in the right place. `below` never had a scrim.

## Spans and rows

A cell's size is a per-item value, not a prop on the list:

```rust,ignore
ImageItem::new(rsx! { Image { src, alt, fit: "cover" } })
    .span(GridSpan::Half) // width, in twelfths - the same GridItem takes
    .rows(2)              // height, in rows - `quilted` only
```

`span` overrides whatever `cols` derived. `rows` is `quilted`'s vocabulary and
is ignored by every other variant, with a warning: `standard` has one row per
cell by definition, and `masonry` derives the row span from the measured height,
so honouring a manual one there would leave the cell overlapping its neighbours.

## Linking a cell

`ImageItem::to` makes the whole tile a hit target. **The anchor is the picture**,
and a stretched `::after { inset: 0 }` extends the hit area over the cell.

```rust,ignore
ImageItem::new(rsx! { Image { src, alt: "Breakfast", fit: "cover" } })
    .bar(ImageBar::new(caption))
    .to(Route::Photo { id })
```

Three consequences:

- **The accessible name is the image's `alt`**, in every linked cell. A
  decorative image (`alt: ""`) leaves the link unnamed. Give it a real `alt`, or
  put a link of your own in the bar.
- **The hit area resolves against the `<li>` only because nothing between the
  anchor and the `<li>` is positioned.** That is the whole contract, and
  breaking it is silent: the component shipped once with a `position: absolute`
  bar, and the hit area was the caption strip rather than the tile. If you style
  a cell through `sx`, do not make anything inside it positioned.
- **The link wins over a `zoomable` `Image`.** Inside a linked cell it draws no
  zoom button, and warns in a debug build: a `<button>` inside the `<a>` is
  invalid HTML and two controls on one tile. Zoom on the page the link opens.

## Columns

`cols` is snapped to a divisor of twelve - 1, 2, 3, 4, 6 or 12 - because a cell
is a span of a twelve-track `GridZone`. Anything else snaps to the nearest and
warns, naming the snap; a tie goes to the wider cell, so 5 becomes 4.

`cols` also takes one count per viewport breakpoint, mobile-first:

```rust,ignore
ImageList {
    // One column on a phone, two from `sm` (48rem), four from `md` (62rem).
    cols: responsive(1).sm(2).md(4),
    items: photos,
}
```

Each count is snapped on its own. The CSS is plain `min-width` media queries, so
server rendering gets it right and nothing measures. An `ImageItem::span` stays
the same at every width. The default is 2, which is the count that still works on
a phone.

## Variants

| Variant | What it does |
|---|---|
| `standard` | Every cell takes the list's `ratio`, so the rows are even. |
| `masonry` | Every cell keeps its picture's own height, and `GridZone`'s `ResizeObserver` packs them with no vertical dead space. |
| `quilted` | `standard`, plus `ImageItem::rows`: a cell may take more than one row, and the quilt's rows stay equal. |
| `woven` | `standard`, with every second cell shortened to 70% and centred. Decoration, and it crops one picture in two. |

`masonry` costs one `ResizeObserver` per cell. Without a browser - during SSR, or
on a target with no DOM - nothing measures and the gallery renders as an ordinary
grid: unpacked, but correct.

`quilted` is not a second layout engine. It is `grid-row: span n` per item plus
an aspect ratio scaled by that cell's own width and height (`ratio * columns /
rows`), so a 2x2 cell is exactly twice the size of a 1x1 one without anyone
naming a pixel height; `ratio` and [AspectRatio](aspect_ratio.md) do that
arithmetic.

Not implemented: a row height, an action position, a raw pixel `gap`, and the
compound `ImageList { ImageListItem {} }` children API.

## Accessibility

- Each picture's accessible name is its own `alt`. `ImageList` never invents
  one. A cell with a bar is a `figure`, the bar its `figcaption`: the caption
  describes the figure, not the image, so its text may repeat the `alt` - the
  caller's call.
- **A cell with `to` puts the anchor on the picture**, so a decorative image
  there leaves the link with no accessible name, with a bar or without one.

### Contrast on an overlay bar

The scrim is a gradient, so text on it has a range rather than a number, and the
backdrop is your picture. Worst case - a pure-white photograph under the default
`bar_background` - white text is **9.3:1** at the bar's bottom edge, 4.2:1 at 40%
up, 2.5:1 at the middle stop and **1.8:1 near the top**. Over a mid-grey picture
the same points are 15.5, 10.7 and 7.8:1.

So the common case is comfortable and the failure is specific: the upper part of
a bar over a bright picture, which the *second line* of a two-line caption
reaches. Use `position: "below"` (no scrim, page text color, always legible),
add a `text-shadow` through `sx`, or raise the scrim's opacity through the
theme - at the cost of it reading as a solid bar rather than a fade.

## Props

### `ImageList`

| Prop | Type | Default | Description |
|---|---|---|---|
| `items` | `Vec<ImageItem>` | required | One cell each, in render order. |
| `cols` | `Responsive<u8>` | `2` | Columns, snapped to a divisor of twelve. `3`, or `responsive(1).sm(2).md(4)`. |
| `variant` | `ImageListVariant` | `standard` | `standard`, `masonry`, `quilted` or `woven`. |
| `gap` | `Size` | `xs` | Between cells. |
| `radius` | `Size` | `sm` | Each cell's corner radius. |
| `ratio` | `f32` | `1.0`, from `theme.aspect_ratio` | Cell aspect ratio. Ignored by `masonry`, with a warning. Under `quilted`, the ratio of one cell of the quilt. |

Like every component, `ImageList` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. They land on the `<ul>`.

### `ImageItem`

A builder, the `Table::column()` shape.

| Method | Type | Description |
|---|---|---|
| `new(content)` | `Element` | The cell's content - an `Image` with `fit: "cover"`, usually. |
| `span(span)` | `GridSpan` | This cell's width, overriding the one `cols` derives. |
| `rows(rows)` | `u8` | This cell's height in rows. `quilted` only. |
| `bar(bar)` | `ImageBar` | The caption strip. |
| `to(target)` | `NavigationTarget` | Makes the cell a link, as a stretched link on the picture. A `zoomable` `Image` in it draws no zoom button. |

### `ImageBar`

| Method | Type | Description |
|---|---|---|
| `new(content)` | `Element` | The strip's content - anything. A flex row is all the bar adds. |
| `position(position)` | `BarPosition` | `bottom`, `top` or `below`, overriding the theme's per cell. |
| `scrim(on)` | `bool` | The gradient behind an overlay bar, and the light text color with it. On by default. |

`ImageItem` holds `Element`s, so `ImageListProps` never compares equal and the
list re-renders whenever its parent does. That is `Table`'s rows and
`SegmentedControl`'s segments by design.

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

There is no `ratio` field: the aspect ratio is [AspectRatio](aspect_ratio.md)'s
`--lsx-aspect-ratio`, so a caller who retunes `theme.aspect_ratio` gets galleries
that match.

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
| `<ul>` | `masonry` | `variant` is `masonry` - `GridZone`'s own token. |
| `<ul>` | `variant-<variant>` | The `variant` in effect. `woven`'s rules key off it. |
| `<li>` | `grid-item`, `span-<span>` | `GridItem`'s own tokens; the span is the cell's. |
| `<li>` | `rows` | The cell has an `ImageItem::rows` under `quilted`. |
| `<li>` | `radius-<size>` | The `radius` in effect. |
| media | `variant-<variant>`, `ratio-box` | The `variant` in effect; `ratio-box` on every variant but `masonry`. |
| bar | `bar-bottom` / `bar-top` / `bar-below` | That cell's bar position. |
| bar | `bar-scrim-bottom` / `bar-scrim-top` | An overlay bar that kept its scrim. |
