# ImageList

Crate: `libero`
Import: `use libero::components::{ImageBar, ImageItem, ImageList};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/image_list/image_list.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A gallery of pictures with optional caption bars, rendered as a `ul`/`li` list over a `GridZone` - so `cols` is a span of the library's own twelve tracks and `masonry` is that zone's measuring engine.

A grid of pictures, each with an optional caption bar. Renders a
`<ul role="list">` of `<li>`s, so a gallery is announced with a count - and every
cell is a `GridItem` of a [GridZone](grid.md), so `cols` is a span of the
library's own twelve tracks rather than a second grid with its own track count.

`masonry` is that zone's measuring engine, not a CSS multi-column - so the
reading order and the visual order agree. Each picture's accessible name is its
own `alt`; the bar is sibling content and never becomes one.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Image, ImageBar, ImageItem, ImageList};

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
                    .bar(ImageBar::new(photo.title.clone()).subtitle(photo.author.clone()))
                })
                .collect(),
        }
    }
}
```

Every prop set:

```rust
ImageList {
    cols: 3u8,
    variant: "masonry",
    gap: "sm",
    radius: "md",
    // Ignored by `masonry`: a packed cell takes its height from its picture.
    ratio: 4.0 / 3.0,
    items: photos,
}
```

### Spans

A cell's width is a per-item value. `ImageItem::span` takes the same twelfths a
`GridItem` does and overrides whatever `cols` derived:

```rust
ImageItem::new(rsx! { Image { src, alt, fit: "cover" } })
    .span(GridSpan::Half)
```

### Linking a cell

`ImageItem::to` makes the whole tile a hit target. The anchor wraps the bar's
title - or the picture, when there is no bar - and a stretched
`::after { inset: 0 }` extends the hit area over the cell. The bar's `action`
stays clickable because it is a sibling of the anchor, not inside it: a
`<button>` inside an `<a>` is invalid HTML.

```rust
ImageItem::new(rsx! { Image { src, alt: "Breakfast", fit: "cover" } })
    .bar(ImageBar::new("Breakfast").action(rsx! { ActionIcon { /* .. */ } }))
    .to(Route::Photo { id })
```

A `<button>` inherits no color of its own, so an action on the `bottom` or `top`
scrim needs to be told - `sx: sx().color("inherit")` on the `ActionIcon`, the
same line [MultiSelect](multi-select.md)'s remove button carries. The bar sets
`color` on itself; nothing inside it is reached by that.

## Columns

`cols` is snapped to a divisor of twelve - 1, 2, 3, 4, 6 or 12 - because a cell
is a span of a twelve-track `GridZone`. Anything else snaps to the nearest and
warns, naming the snap; a tie goes to the wider cell, so 5 becomes 4.

There is one value, not one per breakpoint. Until per-breakpoint props land, a
responsive gallery changes its columns through the caller's own
`sx().breakpoint(..)`. The default is 2, which is the count that still works on a
phone.

## Variants

| Variant | What it does |
|---|---|
| `standard` | Every cell takes the list's `ratio`, so the rows are even. |
| `masonry` | Every cell keeps its picture's own height, and `GridZone`'s `ResizeObserver` packs them with no vertical dead space. |

`masonry` costs one `ResizeObserver` per cell. Without a browser - during SSR, or
on a target with no DOM - nothing measures and the gallery renders as an ordinary
grid: unpacked, but correct.

Not implemented: MUI's `quilted` (it needs a per-item `grid-row: span n`, which
is the masonry engine's own property) and `woven`.

## Accessibility

- The list is a `<ul role="list">` of `<li>`s, so it is announced with a count.
  The explicit `role` is not redundant: Safari with VoiceOver drops list
  semantics from a `list-style: none` list.
- Each picture's accessible name is its own `alt`. `ImageList` never invents one.
- **The bar is not a label for the image.** It is sibling content, so there is no
  `aria-labelledby` wiring - the title may repeat the `alt`, and that is the
  caller's call. A cell with a bar and a decorative image (`alt: ""`) is
  legitimate.
- A cell with `to` and no bar puts the anchor on the picture, so a decorative
  image there would leave the link with no accessible name. Give such a cell a
  bar, or a real `alt`.
- No keyboard contract of its own: a gallery is not a composite widget, so there
  is no roving focus and no arrow keys. A cell's `to` and its bar's `action`
  contribute native tab stops in document order.

### Contrast on an overlay bar

The scrim is a gradient, so white text on it has a range rather than a number,
and the backdrop is your picture. Worst case - a pure-white photograph under the
default `bar_background` - white text is **9.3:1** at the bar's bottom edge,
4.2:1 at 40% up, 2.5:1 at the middle stop and 1.8:1 near the top. Over a
mid-grey picture the same points are 15.5, 10.7 and 7.8:1.

So the common case is comfortable and the failure is specific: the upper part of
a bar over a bright picture, which a two-line bar's `subtitle` can reach. If your
pictures are pale, either use `position: "below"` - no scrim, page text colour,
always legible - or add a `text-shadow` through `sx`. Raising the scrim's opacity
works too, at the cost of it reading as a solid bar rather than a fade.

## Props

### `ImageList`

| Prop | Type | Default | Description |
|---|---|---|---|
| `items` | `Vec<ImageItem>` | required | One cell each, in render order. |
| `cols` | `u8` | `2` | Columns, snapped to a divisor of twelve. |
| `variant` | `ImageListVariant` | `standard` | `standard` or `masonry`. |
| `gap` | `Size` | `xs` | Between cells. |
| `radius` | `Size` | `sm` | Each cell's corner radius. |
| `ratio` | `f32` | `1.0`, from `theme.aspect_ratio` | Cell aspect ratio. Ignored by `masonry`, with a warning. |

Like every component, `ImageList` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. They land on the `<ul>`.

### `ImageItem`

A builder, the `Table::column()` shape.

| Method | Type | Description |
|---|---|---|
| `new(content)` | `Element` | The cell's content - an `Image` with `fit: "cover"`, usually. |
| `span(span)` | `GridSpan` | This cell's width, overriding the one `cols` derives. |
| `bar(bar)` | `ImageBar` | The caption strip. |
| `to(target)` | `NavigationTarget` | Makes the cell a link, as a stretched link. |

### `ImageBar`

| Method | Type | Description |
|---|---|---|
| `new(title)` | `OptionLabel` | The caption. Sibling content, not a label for the image. |
| `subtitle(subtitle)` | `OptionLabel` | A second, dimmer line. |
| `action(action)` | `Element` | A control at the end of the bar. |
| `position(position)` | `BarPosition` | `bottom`, `top` or `below`, overriding the theme's per cell. |

`ImageItem` holds `Element`s, so `ImageListProps` never compares equal and the
list re-renders whenever its parent does. That is `Table`'s rows and
`SegmentedControl`'s segments by design.

## Theme defaults

`ImageListDefaults` on the theme, as `theme.image_list`.

| Field | Type | Default | Description |
|---|---|---|---|
| `cols` | `u8` | `2` | Columns when the prop is unset. |
| `variant` | `ImageListVariant` | `Standard` | |
| `gap` | `Size` | `Size::Xs` | |
| `radius` | `Size` | `Size::Sm` | |
| `bar_position` | `BarPosition` | `Bottom` | Used by any bar that names none. |
| `bar_background` | `&'static str` | a black-to-transparent gradient | The scrim behind a `bottom` bar. |
| `bar_background_top` | `&'static str` | the same, reversed | The scrim behind a `top` bar. |
| `bar_color` | `&'static str` | `"#fff"` | Bar text, on either scrim. |
| `bar_subtitle_opacity` | `&'static str` | `"0.72"` | How far the subtitle is dimmed. |
| `bar_padding` | `Size` | `Size::Sm` | The bar's inset. |

There is no `ratio` field: the aspect ratio is [AspectRatio](aspect-ratio.md)'s
`--lsx-aspect-ratio`, so a caller who retunes `theme.aspect_ratio` gets galleries
that match.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-image-list-bar-background` | The `bottom` scrim. Declared on `:root` from the theme. |
| `--lsx-image-list-bar-background-top` | The `top` scrim. Declared on `:root` from the theme. |
| `--lsx-image-list-bar-color` | Bar text color. Declared on `:root` from the theme. |
| `--lsx-image-list-bar-subtitle-opacity` | Subtitle opacity. Declared on `:root` from the theme. |
| `--lsx-image-list-bar-padding` | The bar's inset, resolved from a spacing step. |
| `--lsx-image-list-radius` | The cell's corner radius, picked on the cell from its own `radius-*` token. |
| `--lsx-aspect-ratio-override` | The list's `ratio`, over `AspectRatio`'s own pair. |
| `--lsx-grid-zone-gap` | `GridZone`'s, published from `gap`. |

## Data attributes

| Element | Token | Condition |
|---|---|---|
| `<ul>` | `masonry` | `variant` is `masonry` - `GridZone`'s own token. |
| `<li>` | `grid-item`, `span-<span>` | `GridItem`'s own tokens; the span is the cell's. |
| `<li>` | `radius-<size>` | The `radius` in effect. |
| media | `variant-standard` / `variant-masonry` | The `variant` in effect. |
| bar | `bar-bottom` / `bar-top` / `bar-below` | That cell's bar position. |
