# Grid

Crate: `libero`
Import: `use libero::components::{Grid, GridZone, GridItem, GridSpan, GridArea, StaticGridTemplate, sp};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/grid>
Index: [index.md](index.md) - every other component's markdown page
Description: A named-area layout matrix - `Grid` holds the shape, a `GridZone` is a twelve-column packing container with optional masonry, and a `GridItem` takes a fraction of it.

A named-area matrix. `Grid` holds the shape, each `GridZone` is an independent
twelve-column packing container, and a `GridItem` takes a fraction of its zone.
A zone works on its own too - a masonry wall needs no template.

## Usage

A zone with no `area` and no `Grid` around it: six cards of mixed spans and
mixed heights. Turn on `masonry` and the vertical dead space goes away.

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

`dense` backfills gaps a wider item left behind - pure CSS, no measurement, and
it only moves items sideways. Vertical dead space under a short card needs
`masonry`, which measures every item and packs it against the column above. They
compose: a wall of mixed spans wants both.

`masonry` costs a `ResizeObserver` per item. Without a browser - during SSR, or
on a target with no DOM - nothing measures and the zone renders as an ordinary
grid: unpacked, but correct.

Both make visual order diverge from DOM order. Tab order always follows the DOM,
so don't reach for either where the reading order carries meaning.

## Named areas

A `GridTemplate` is a matrix of enum variants. Each row shares its width equally
between its cells, and `cells(area, n)` gives one area several of them - so a row
of `cell(Sidebar)` plus `cells(Content, 3)` splits one to three. Rows of
different lengths reconcile to their least common multiple: a one-cell row above
a four-cell row gives four columns, and a three-cell row beside a two-cell one
would give six. `build` rejects a shape CSS cannot express: an area that is not a
rectangle, or rows needing more than twelve columns.

Cells say how much *width* an area gets, not how many items it holds. Every zone
is its own twelve-column grid regardless of the template, so a one-cell header
holds two items and would just as happily hold six.

A zone subdivides into twelfths however narrow it is, so its column gutter
shrinks with the zone once there is no room for the full one - twelve tracks
always carry eleven gaps, and a fixed gutter would make a narrow zone overflow
its own area.

A zone is placed by name, so the order the zones appear in has no effect on where
they land - it is only the reading and tab order. The example below is written
content-first and the sidebar last, and still renders header on top. A zone whose
`area` names nothing in the template warns and auto-places instead.

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

// Written content-first, sidebar last - the template decides where each
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

`GridSpan` is closed, so an invalid width is not representable. Every value is an
exact twelfth of the zone: `Full` (12), `ThreeQuarters` (9), `TwoThirds` (8),
`Half` (6), `Third` (4), `Quarter` (3), `Sixth` (2), `Twelfth` (1).

## Responsive spans

A zone is almost never as wide as the window, so `bp()` is the wrong question for
a span. `sp()` keys the span off the *zone's* width instead, through a container
query. Narrow the window and the three cards below walk down the ladder: three
across, then two with C wrapping under them, then one per row.

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

The breakpoints are the same `Size` scale a viewport query uses - `sm` is 48rem
either way - but they measure the zone, so a zone that is a quarter of a wide
page still counts as small and keeps its base span. A plain `GridSpan` still
costs no query at all: only an item with breakpoints gets a rule of its own.

## Caveats

A masonry zone's height is derived from its items, so it cannot also be a
viewport. To scroll, put a [`ScrollArea`](scroll_area.md) inside a `GridItem` -
not around the zone. Nesting a masonry zone directly in a container whose width
follows its content is the same trap from the other side: taller items make the
zone taller, a scrollbar appears, the width changes, and everything re-measures.

Masonry packs by giving each item a row span and cancelling the row gap, so an
item's `sx` that sets `align-self` or `margin-bottom` overwrites the mechanism.
At the default `stretch` an item's border box becomes its whole row span, the
next measurement reports that, and the item grows without bound.

A zone filling a named area is a query container, which means
`contain: layout style inline-size`: it is a stacking context and the containing
block for any absolutely positioned descendant. Something inside a `GridItem`
that positions itself against the page needs a portal, not a
`position: absolute`.

A zone used on its own, outside a `Grid`, is deliberately not a container:
inline-size containment zeroes an element's own contribution to its width, which
collapses a shrink-to-fit box to nothing. That also means `sp()` needs a zone
with an area - there is nothing to query otherwise, and the base span stands.

Build templates outside the render - a `StaticGridTemplate` static, as every
example here does. Rebuilding one per render re-parses the matrix and hands
`Grid` a new value every time.

## Props

### Grid

| Prop | Type | Default | Description |
|---|---|---|---|
| `template` | `GridTemplate` | required | The named-area matrix. Build it once outside the render. |
| `gap` | `Size` | `md` | Between zones. |
| `component` | `HtmlTag` | `div` | Overrides the root element. |
| `children` | `Element` | required | `GridZone`s. |

### GridZone

| Prop | Type | Default | Description |
|---|---|---|---|
| `area` | `AreaName` | - | Which of the parent `Grid`'s areas this fills. Omit it to use the zone on its own, without a `Grid` - a plain masonry wall needs no template. |
| `dense` | `bool` | `false` | Backfill gaps a wider item left behind. Pure CSS, no measurement. |
| `masonry` | `bool` | `false` | Measure item heights and pack them with no vertical dead space. Costs a `ResizeObserver` per item. |
| `gap` | `Size` | `md` | Between items. |
| `component` | `HtmlTag` | `div` | Overrides the root element. |
| `children` | `Element` | required | `GridItem`s. |

### GridItem

| Prop | Type | Default | Description |
|---|---|---|---|
| `span` | `SpanValue` | `full` | Width, in twelfths of the zone. A `GridSpan`, or `sp()` for a span that changes with the zone's width. |
| `component` | `HtmlTag` | `div` | Overrides the root element. |
| `children` | `Element` | required | The item's content. |

Like every component, all three also take the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Theme defaults

`GridDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `gap` | `Size` | Gap between zones - `md` by default. |
| `zone_gap` | `Size` | Gap between items inside a zone - `md` by default. |
| `row_unit` | `u32` | Masonry row quantum in px (`2`). A number, not a CSS length: an item's row span is `ceil(height / row_unit)`, computed in Rust. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-grid-gap` | Gap between zones, from `GridDefaults::gap`. |
| `--lsx-grid-zone-gap` | Gap between items; republished by every zone so a nested one does not inherit its parent's. |
| `--lsx-grid-row-unit` | Masonry row quantum. |
| `--lsx-grid-areas` | The template's `grid-template-areas` string, set per instance in `style`. |
| `--lsx-grid-columns` | The template's reconciled column count. |
| `--lsx-grid-zone-area` | The zone's area name; unset for a zone used on its own. |
| `--lsx-grid-zone-container` | The zone's `@container` name, so an item can key a span off its zone's width. Unset when the zone has no area. |
| `--lsx-grid-item-rows` | Row span a masonry item was measured into. |

## Data attributes

State tokens on each root's `data-state`, space separated.

| Token | On | Condition |
|---|---|---|
| `size-<size>` | `Grid` | The `gap` in effect. |
| `container` | `GridZone` | The zone has an area, so it is a query container. |
| `dense` | `GridZone` | `dense` is set. |
| `masonry` | `GridZone` | `masonry` is set. |
| `grid-item` | `GridItem` | Always - it is how a zone selects its own items. |
| `full` / `three-quarters` / `two-thirds` / `half` / `third` / `quarter` / `sixth` / `twelfth` | `GridItem` | The base span in effect. |
| `measured` | `GridItem` | Masonry has measured this item's height. |
