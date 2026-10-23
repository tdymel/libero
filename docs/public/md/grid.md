# Grid

Crate: `libero`
Import: `use libero::components::{Grid, GridZone, GridItem, GridSpan, GridArea, StaticGridTemplate}; use libero::theme::{Responsive, responsive};`
Feature: `grid` (on by default, turns on `box`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/grid>
Index: [index.md](index.md) lists every other page
Description: A layout matrix of named areas: `Grid` holds the shape, a `GridZone` is a twelve-column container with optional masonry, a `GridItem` takes a fraction of it.

A layout matrix of named areas. `Grid` holds the shape, each `GridZone` is a
twelve-column container of its own, and a `GridItem` takes a fraction of its
zone. A zone also works alone, so a masonry wall needs no template.

## Usage

A zone on its own, with six cards of mixed spans and heights. Turn on `masonry`
and the vertical gaps go away.

```rust
use dioxus::prelude::*;
use libero::{components::{Box, GridItem, GridSpan, GridZone, Text}, sx::sx};

#[component]
fn Card(lines: u32, label: String) -> Element {
    rsx! {
        Box {
            sx: sx()
                .padding("12px")
                .background("primary.1")
                .border_radius("sm")
                .height(format!("{}px", 28 + lines * 22)),
            Text { size: "sm", "{label}" }
        }
    }
}

#[component]
fn Demo() -> Element {
    rsx! {
        GridZone {
            sx: sx().width("100%"),
            GridItem { span: GridSpan::Third, Card { lines: 1, label: "A" } }
            GridItem { span: GridSpan::TwoThirds, Card { lines: 4, label: "B" } }
            GridItem { span: GridSpan::Third, Card { lines: 2, label: "C" } }
            GridItem { span: GridSpan::Third, Card { lines: 3, label: "D" } }
            GridItem { span: GridSpan::Half, Card { lines: 1, label: "E" } }
            GridItem { span: GridSpan::Third, Card { lines: 2, label: "F" } }
        }
    }
}
```

## Props

### Grid

| Prop | Type | Default | Description |
|---|---|---|---|
| `template` | `GridTemplate` | required | The named-area matrix. Each row shares its width equally between its cells, and `cells(area, n)` gives one area several. An area must be a rectangle, and the rows need at most twelve columns. Build it once, as a `StaticGridTemplate` static. |
| `gap` | `Responsive<ThemeAwareValue>` | `md` | Space between zones, or any CSS, e.g. `gap: "0"`, per breakpoint too. |
| `component` | `HtmlTag` | `div` | The element to render. |
| `children` | `Element` | required | `GridZone`s. |

### GridZone

| Prop | Type | Default | Description |
|---|---|---|---|
| `area` | `AreaName` | - | The `Grid` area this zone fills. Zones land by name, so their order only sets the reading and tab order. A zone with an area is a size container (`container-type: inline-size`), not a containing block: a `position: fixed` descendant, such as a `fixed` `Header`, stays on the viewport, and an absolutely positioned one needs `position: relative` on its `GridItem`. Omit it to use the zone on its own, without a `Grid`, and give it a width. |
| `dense` | `bool` | `false` | Fills the gaps a wider item left, so an item may move up or sideways out of source order. |
| `masonry` | `bool` | `false` | Packs items of different heights with no vertical gaps. The zone's height follows its items, so scroll inside a `GridItem`, not around the zone, and set no `align-self` or `margin-bottom` on an item. Keep it out of a container whose width follows its content: a scrollbar coming and going makes it measure again and again. |
| `gap` | `ThemeAwareValue` | `md` | Space between items, or any CSS, e.g. `gap: "12px"`. Masonry packs by a `px` or `rem` gap; other CSS packs by the theme's gap. |
| `component` | `HtmlTag` | `div` | The element to render. |
| `children` | `Element` | required | `GridItem`s. |

### GridItem

| Prop | Type | Default | Description |
|---|---|---|---|
| `span` | `Responsive<GridSpan>` | `full` | Width in twelfths of the zone. A `GridSpan` (`Full` 12, `ThreeQuarters` 9, `TwoThirds` 8, `Half` 6, `Third` 4, `Quarter` 3, `Sixth` 2, `Twelfth` 1), or `responsive(..)` for a span that follows the zone's width, not the window's. Its breakpoints need a zone with an `area`. |
| `rows` | `u8` | - | Height in rows of the zone's grid. Ignored in a masonry zone, which sets it from the measured height. |
| `component` | `HtmlTag` | `div` | The element to render. |
| `children` | `Element` | required | The item's content. |

Like every component, all three also take the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Accessibility

### Libero handles

- `Grid`, `GridZone` and `GridItem` add no roles: `component` names the tag,
  such as `HtmlTag::Header` or `HtmlTag::Aside` for a landmark. A `header`
  is a banner only outside `main`, `article`, `aside`, `nav` and `section`.

### You must

- Write the zones in reading order: the template places them anywhere, but
  screen readers and Tab follow the code.
- Use `dense` only where order means nothing, such as a photo wall: a later
  item fills an earlier gap and shows before items it follows in the code.
- Give two landmarks of the same kind an `aria-label` each, such as two
  `Aside` zones. An `Aside` zone inside `main` needs one to be a landmark at
  all.
- Use `responsive(..)` for a span that must stack on a narrow screen: a fixed
  `GridSpan` and the named-area template keep their fractions at 320 px, where
  a `Quarter` is under 74 px wide.

### Example

A page `Grid` with a header, a main zone and two `Aside` zones named "Filters"
and "Related": a screen reader lists the two complementary landmarks apart,
and Tab follows the code order.

## Theme defaults

`GridDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `gap` | `Size` | Gap between zones, `md` by default. |
| `zone_gap` | `Size` | Gap between items inside a zone, `md` by default. |
| `row_unit` | `u32` | Height of one masonry row in px, `2` by default. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-grid-gap` | Gap between zones, from `GridDefaults::gap`. |
| `--lsx-grid-zone-gap` | Gap between items in a zone. |
| `--lsx-grid-row-unit` | Height of one masonry row. |
| `--lsx-grid-areas` | The template's `grid-template-areas`. |
| `--lsx-grid-columns` | The template's column count. |
| `--lsx-grid-zone-area` | The zone's area name. Unset for a zone on its own. |
| `--lsx-grid-zone-container` | The zone's `@container` name. Unset when the zone has no area. |
| `--lsx-grid-item-rows` | Rows an item spans. |

## Data attributes

State tokens on each root's `data-state`, space separated.

| Token | On | Condition |
|---|---|---|
| `size-<size>` | `Grid` | The `gap` in effect. |
| `container` | `GridZone` | The zone has an area. |
| `dense` | `GridZone` | `dense` is set. |
| `masonry` | `GridZone` | `masonry` is set. |
| `grid-item` | `GridItem` | Always. |
| `full` / `three-quarters` / `two-thirds` / `half` / `third` / `quarter` / `sixth` / `twelfth` | `GridItem` | The base span in effect. |
| `measured` | `GridItem` | Masonry has measured this item's height. |
| `rows` | `GridItem` | `rows` is set and the zone is not a masonry one. |
