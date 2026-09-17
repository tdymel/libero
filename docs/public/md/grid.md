# Grid

Crate: `libero`
Import: `use libero::components::{Grid, GridZone, GridItem, GridSpan, GridArea, StaticGridTemplate, sp};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/grid>
Index: [index.md](index.md) - every other component's markdown page
Description: A layout matrix of named areas. `Grid` holds the shape, a `GridZone` is a twelve-column container with optional masonry, and a `GridItem` takes a fraction of it.

A layout matrix of named areas. `Grid` holds the shape, each `GridZone` is a
twelve-column container of its own, and a `GridItem` takes a fraction of its
zone. A zone also works alone, so a masonry wall needs no template.

`dense` fills gaps by moving items out of DOM order, but Tab still follows the
DOM. Skip it where the reading order matters. `masonry` alone keeps the order,
since each item starts no higher than the one before it.

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

## masonry and dense

`dense` fills the gaps a wider item left, moving items sideways only. The gap
under a short card needs `masonry`, which measures every item and packs it
against the one above. A wall of mixed spans wants both.

`masonry` watches the size of every item. Without a DOM, as in SSR, the zone
renders as an ordinary grid, unpacked but correct.

## Named areas

A `GridTemplate` is a matrix of enum variants. Each row shares its width equally
between its cells, and `cells(area, n)` gives one area several. A row of
`cell(Sidebar)` plus `cells(Content, 3)` splits one to three. Rows of different
lengths meet at their least common multiple, so a one-cell row above a four-cell
row gives four columns. `build` rejects an area that is not a rectangle, and
rows that need more than twelve columns.

Cells set how much width an area gets, not how many items it holds. Every zone
is its own twelve-column grid, so a one-cell header can hold two items or six.
In a narrow zone the gutter between columns shrinks so the zone still fits.

Zones land by name, so their order only sets the reading and tab order. The
example below puts the content first and the sidebar last, and the header still
renders on top. A zone whose `area` is not in the template warns and places
itself.

```rust
use dioxus::prelude::*;
use libero::{
    components::{Grid, GridArea, GridItem, GridSpan, GridZone, HtmlTag, StaticGridTemplate},
    sx::sx,
};

#[derive(Clone, Copy, PartialEq)]
enum PageArea { Header, Sidebar, Content }

impl GridArea for PageArea {
    fn name(&self) -> &'static str {
        match self {
            Self::Header => "header",
            Self::Sidebar => "sidebar",
            Self::Content => "content",
        }
    }
}

// A one-cell row spans the whole width. The row below splits into four, so
// the sidebar takes one column and the content three.
static PAGE: StaticGridTemplate<PageArea> = StaticGridTemplate::new(|template| {
    template
        .row(|row| row.cell(PageArea::Header))
        .row(|row| row.cell(PageArea::Sidebar).cells(PageArea::Content, 3))
});

// Written content-first, sidebar last. The template decides where each
// zone lands, so this order only sets the reading and tab order.
#[component]
fn Demo() -> Element {
    rsx! {
        Grid {
            template: PAGE.clone(),
            sx: sx().background("muted.1").padding("12px").border_radius("sm"),
            GridZone {
                area: PageArea::Content,
                masonry: true,
                GridItem { span: GridSpan::Half, "Card A" }
                GridItem { span: GridSpan::Half, "Card B" }
                GridItem { span: GridSpan::Half, "Card C" }
                GridItem { span: GridSpan::Half, "Card D" }
            }
            GridZone {
                area: PageArea::Header,
                component: HtmlTag::Header,
                GridItem { span: GridSpan::Half, "Logo" }
                GridItem { span: GridSpan::Half, "Nav" }
            }
            GridZone {
                area: PageArea::Sidebar,
                component: HtmlTag::Aside,
                GridItem { "Menu" }
            }
        }
    }
}
```

## Spans

Every `GridSpan` is an exact number of twelfths: `Full` (12), `ThreeQuarters`
(9), `TwoThirds` (8), `Half` (6), `Third` (4), `Quarter` (3), `Sixth` (2),
`Twelfth` (1).

`rows` sets an item's height in rows of the zone's grid. A masonry zone ignores
it and sets the rows from the measured height.

## Responsive spans

A zone is rarely as wide as the window, so `sp()` keys the span off the zone's
width, not the window's. Narrow the window and the three cards below go from
three across to two, with C wrapping, to one per row.

```rust
use dioxus::prelude::*;
use libero::components::{Grid, GridItem, GridSpan, GridZone, SpanValue, sp};

// Three cards, one span value. Wide: three across. Tablet: two, and C
// wraps. Phone: one per row.
const CARD_SPAN: SpanValue = sp().base(GridSpan::Full).sm(GridSpan::Half).md(GridSpan::Third);

#[component]
fn Demo() -> Element {
    rsx! {
        Grid { template: SPANS.clone(),
            GridZone { area: SpanArea::Row,
                GridItem { span: CARD_SPAN, "A" }
                GridItem { span: CARD_SPAN, "B" }
                GridItem { span: CARD_SPAN, "C" }
            }
        }
    }
}
#
# use libero::components::{GridArea, StaticGridTemplate};
# #[derive(Clone, Copy, PartialEq)]
# enum SpanArea { Row }
# impl GridArea for SpanArea { fn name(&self) -> &'static str { "row" } }
# static SPANS: StaticGridTemplate<SpanArea> =
#     StaticGridTemplate::new(|template| template.row(|row| row.cell(SpanArea::Row)));
```

The breakpoints are the same `Size` scale a viewport query uses, `sm` is 48rem.
They measure the zone, so a zone a quarter of a wide page still counts as small
and keeps its base span. `sp()` needs a zone with an `area`. Without one the
base span stands.

## Caveats

A masonry zone's height follows its items, so it cannot scroll. Put a
[`ScrollArea`](scroll_area.md) inside a `GridItem`, not around the zone. Don't
put a masonry zone in a container whose width follows its content either. A
scrollbar coming and going makes it measure again and again.

Set no `align-self` or `margin-bottom` on an item in a masonry zone. Masonry
relies on both, and an item that overrides them grows without bound.

A zone with an area is a stacking context and the containing block of any
absolutely positioned descendant. Something in a `GridItem` that positions
itself against the page cannot escape it.

A zone on its own has no width of its own. Give it one, as the usage example
does.

Build templates once, as a `StaticGridTemplate` static, like every example here.

## Props

### Grid

| Prop | Type | Default | Description |
|---|---|---|---|
| `template` | `GridTemplate` | required | The named-area matrix. Each row shares its width equally between its cells, and `cells(area, n)` gives one area several. An area must be a rectangle, and the rows need at most twelve columns. Build it once, as a `StaticGridTemplate` static. |
| `gap` | `Size` | `md` | Space between zones. |
| `component` | `HtmlTag` | `div` | The element to render. |
| `children` | `Element` | required | `GridZone`s. |

### GridZone

| Prop | Type | Default | Description |
|---|---|---|---|
| `area` | `AreaName` | - | The `Grid` area this zone fills. Zones land by name, so their order only sets the reading and tab order. A zone with an area is the containing block of any absolutely positioned descendant. Omit it to use the zone on its own, without a `Grid`. |
| `dense` | `bool` | `false` | Fills the gaps a wider item left, moving items sideways only. |
| `masonry` | `bool` | `false` | Packs items of different heights with no vertical gaps. The zone's height follows its items, so scroll inside a `GridItem`, not around the zone, and set no `align-self` or `margin-bottom` on an item. |
| `gap` | `Size` | `md` | Space between items. |
| `component` | `HtmlTag` | `div` | The element to render. |
| `children` | `Element` | required | `GridItem`s. |

### GridItem

| Prop | Type | Default | Description |
|---|---|---|---|
| `span` | `SpanValue` | `full` | Width in twelfths of the zone. A `GridSpan`, or `sp()` for a span that follows the zone's width, not the window's. `sp()` needs a zone with an `area`. |
| `rows` | `u8` | - | Height in rows of the zone's grid. Ignored in a masonry zone, which sets it from the measured height. |
| `component` | `HtmlTag` | `div` | The element to render. |
| `children` | `Element` | required | The item's content. |

Like every component, all three also take the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

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
