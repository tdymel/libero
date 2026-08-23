use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, or_unset, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{
        Box, Code, CodeBlock, Grid, GridArea, GridItem, GridSpan, GridZone, HtmlTag, SpanValue,
        StaticGridTemplate, Text, sp,
    },
    sx::sx,
};

/// The zone's children, printed verbatim - their point is that they are
/// *different heights*, which is what `masonry` reacts to.
const CARDS: &str = r#"GridItem { span: GridSpan::Third, Card { lines: 1, "A" } }
GridItem { span: GridSpan::TwoThirds, Card { lines: 4, "B" } }
GridItem { span: GridSpan::Third, Card { lines: 2, "C" } }
GridItem { span: GridSpan::Third, Card { lines: 3, "D" } }
GridItem { span: GridSpan::Half, Card { lines: 1, "E" } }
GridItem { span: GridSpan::Third, Card { lines: 2, "F" } }"#;

/// Span, height and label per card. The spans are deliberately mixed: `dense`
/// backfills *holes*, and a wall of equal spans leaves none - a uniform set
/// makes the switch look broken. Here E (six twelfths) cannot follow C + D
/// (four each), leaving a four-wide hole that F fits exactly.
const DEMO_CARDS: [(GridSpan, u32, &str); 6] = [
    (GridSpan::Third, 1, "A"),
    (GridSpan::TwoThirds, 4, "B"),
    (GridSpan::Third, 2, "C"),
    (GridSpan::Third, 3, "D"),
    (GridSpan::Half, 1, "E"),
    (GridSpan::Third, 2, "F"),
];

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

fn cards() -> Element {
    rsx! {
        for (span, lines, label) in DEMO_CARDS {
            GridItem { span, Card { lines, label } }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum PageArea {
    Header,
    Sidebar,
    Content,
}

impl GridArea for PageArea {
    fn name(&self) -> &'static str {
        match self {
            Self::Header => "header",
            Self::Sidebar => "sidebar",
            Self::Content => "content",
        }
    }
}

// Built once: the shape never changes between renders, and `build`'s `Result`
// cannot travel through a component body.
static PAGE: StaticGridTemplate<PageArea> = StaticGridTemplate::new(|template| {
    template
        .row(|row| row.cell(PageArea::Header))
        .row(|row| row.cell(PageArea::Sidebar).cells(PageArea::Content, 3))
});

const TEMPLATE_SOURCE: &str = r#"#[derive(Clone, Copy, PartialEq)]
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
rsx! {
    Grid {
        template: PAGE.clone(),
        sx: sx().background("grey.1").padding("12px").border_radius("sm"),
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
}"#;

fn panel(label: &str, height: u32) -> Element {
    rsx! {
        Box {
            sx: sx()
                .padding("10px 12px")
                .background("primary.1")
                .border_radius("sm")
                .height(format!("{height}px")),
            Text { size: "sm", "{label}" }
        }
    }
}

fn controls() -> Vec<Control> {
    vec![
        Control::switch("masonry"),
        Control::switch("dense"),
        Control::slider("gap", ["auto", "xs", "sm", "md", "lg", "xl", "xxl"]),
    ]
}

#[component]
pub fn GridPage() -> Element {
    rsx! {
        DocPage {
            title: "Grid",
            properties: vec![
                props("Grid", vec![
                    prop("template", "GridTemplate").doc("The named-area matrix. Build it once outside the render."),
                    prop("gap", "Size").default("md").doc("Between zones."),
                    prop("component", "HtmlTag").default("div").doc("Overrides the root element."),
                    prop("children", "Element").doc("`GridZone`s."),
                ]),
                props("GridZone", vec![
                    prop("area", "AreaName").doc("Which of the parent `Grid`'s areas this fills. Omit it to use the zone on its own, without a `Grid` - a plain masonry wall needs no template."),
                    prop("dense", "bool").default("false").doc("Backfill gaps a wider item left behind. Pure CSS, no measurement."),
                    prop("masonry", "bool").default("false").doc("Measure item heights and pack them with no vertical dead space. Costs a `ResizeObserver` per item."),
                    prop("gap", "Size").default("md").doc("Between items."),
                    prop("component", "HtmlTag").default("div").doc("Overrides the root element."),
                    prop("children", "Element").doc("`GridItem`s."),
                ]),
                props("GridItem", vec![
                    prop("span", "SpanValue").default("full").doc("Width, in twelfths of the zone. A `GridSpan`, or `sp()` for a span that changes with the zone's width."),
                    prop("component", "HtmlTag").default("div").doc("Overrides the root element."),
                    prop("children", "Element").doc("The item's content."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A named-area matrix. `Grid` holds the shape, each `GridZone` is an "
                    "independent twelve-column packing container, and a `GridItem` takes a "
                    "fraction of its zone. A zone works on its own too - a masonry wall needs "
                    "no template."
                }
            },
            Demo {
                component: "GridZone",
                children_text: "",
                children_code: CARDS,
                controls: controls(),
                render: move |values: DemoValues| rsx! {
                    GridZone {
                        masonry: values.str("masonry") == "true",
                        dense: values.str("dense") == "true",
                        gap: or_unset(values.str("gap")),
                        {cards()}
                    }
                },
            }
            DocSection {
                title: "masonry and dense",
                Text {
                    Code { source: "dense" }
                    " backfills gaps a wider item left behind - pure CSS, no measurement, "
                    "and it only moves items sideways. Vertical dead space under a short "
                    "card needs "
                    Code { source: "masonry" }
                    ", which measures every item and packs it against the column above. "
                    "They compose: a wall of mixed spans wants both."
                }
                Text {
                    Code { source: "masonry" }
                    " costs a "
                    Code { source: "ResizeObserver" }
                    " per item. Without a browser - during SSR, or on a target with no DOM "
                    "- nothing measures and the zone renders as an ordinary grid: unpacked, "
                    "but correct."
                }
                Text {
                    "Both make visual order diverge from DOM order. Tab order always follows "
                    "the DOM, so don't reach for either where the reading order carries "
                    "meaning."
                }
            }
            DocSection {
                title: "Named areas",
                Text {
                    "A `GridTemplate` is a matrix of enum variants. Each row shares its width "
                    "equally between its cells, and `cells(area, n)` gives one area several of "
                    "them - so a row of `cell(Sidebar)` plus `cells(Content, 3)` splits one "
                    "to three. Rows of different lengths reconcile to their least common "
                    "multiple: a one-cell row above a four-cell row gives four columns, and a "
                    "three-cell row beside a two-cell one would give six. `build` rejects a "
                    "shape CSS cannot express: an area that is not a rectangle, or rows "
                    "needing more than twelve columns."
                }
                Text {
                    "Cells say how much *width* an area gets, not how many items it holds. "
                    "Every zone is its own twelve-column grid regardless of the template, so "
                    "the one-cell header below holds two items and would just as happily hold "
                    "six."
                }
                Text {
                    "A zone subdivides into twelfths however narrow it is, so its column "
                    "gutter shrinks with the zone once there is no room for the full one - "
                    "twelve tracks always carry eleven gaps, and a fixed gutter would make a "
                    "narrow zone overflow its own area."
                }
                Text {
                    "A zone is placed by name, so the order the zones appear in has no effect "
                    "on where they land - it is only the reading and tab order. The example "
                    "below is written content-first and the sidebar last, and still renders "
                    "header on top. A zone whose `area` names nothing in the template warns "
                    "and auto-places instead."
                }
                Grid {
                    template: PAGE.clone(),
                    sx: sx().background("grey.1").padding("12px").border_radius("sm"),
                    GridZone {
                        area: PageArea::Content,
                        masonry: true,
                        GridItem { span: GridSpan::Half, {panel("Card A", 60)} }
                        GridItem { span: GridSpan::Half, {panel("Card B", 120)} }
                        GridItem { span: GridSpan::Half, {panel("Card C", 90)} }
                        GridItem { span: GridSpan::Half, {panel("Card D", 50)} }
                    }
                    GridZone {
                        area: PageArea::Header,
                        component: HtmlTag::Header,
                        GridItem { span: GridSpan::Half, {panel("Logo", 44)} }
                        GridItem { span: GridSpan::Half, {panel("Nav", 44)} }
                    }
                    GridZone {
                        area: PageArea::Sidebar,
                        component: HtmlTag::Aside,
                        GridItem { {panel("Menu", 220)} }
                    }
                }
                CodeBlock { source: TEMPLATE_SOURCE, language: "rust" }
            }
            DocSection {
                title: "Spans",
                Text {
                    Code { source: "GridSpan" }
                    " is closed, so an invalid width is not representable. Every value is "
                    "an exact twelfth of the zone."
                }
                Grid {
                    template: SPANS.clone(),
                    sx: sx().background("grey.1").padding("12px").border_radius("sm"),
                    GridZone {
                        area: SpanArea::Row,
                        gap: "xs",
                        for span in GridSpan::ALL {
                            GridItem { span: *span, {panel(span.as_str(), 32)} }
                        }
                    }
                }
            }

            DocSection {
                title: "Responsive spans",
                Text {
                    "A zone is almost never as wide as the window, so "
                    Code { source: "bp()" }
                    " is the wrong question for a span. "
                    Code { source: "sp()" }
                    " keys the span off the "
                    Code { source: "zone's" }
                    " width instead, through a container query. Narrow the window and "
                    "the three cards below walk down the ladder: three across, then two "
                    "with C wrapping under them, then one per row."
                }
                Grid {
                    template: SPANS.clone(),
                    sx: sx().background("grey.1").padding("12px").border_radius("sm"),
                    GridZone {
                        area: SpanArea::Row,
                        gap: "xs",
                        for name in ["A", "B", "C"] {
                            GridItem { key: "{name}", span: CARD_SPAN, {panel(name, 32)} }
                        }
                    }
                }
                CodeBlock { source: RESPONSIVE_SOURCE, language: "rust" }
                Text {
                    "The breakpoints are the same "
                    Code { source: "Size" }
                    " scale a viewport query uses - "
                    Code { source: "sm" }
                    " is 48rem either way - but they measure the zone, so a zone that is "
                    "a quarter of a wide page still counts as small and keeps its base "
                    "span. A plain "
                    Code { source: "GridSpan" }
                    " still costs no query at all: only an item with breakpoints gets a "
                    "rule of its own."
                }
            }

            DocSection {
                title: "Caveats",
                Text {
                    "A masonry zone's height is "
                    Code { source: "derived" }
                    " from its items, so it cannot also be a viewport. To scroll, put a "
                    Code { source: "ScrollArea" }
                    " inside a "
                    Code { source: "GridItem" }
                    " - not around the zone. Nesting a masonry zone directly in a "
                    "container whose width follows its content is the same trap from the "
                    "other side: taller items make the zone taller, a scrollbar appears, "
                    "the width changes, and everything re-measures."
                }
                Text {
                    "Masonry packs by giving each item a row span and cancelling the row "
                    "gap, so an item's "
                    Code { source: "sx" }
                    " that sets "
                    Code { source: "align-self" }
                    " or "
                    Code { source: "margin-bottom" }
                    " overwrites the mechanism. At the default "
                    Code { source: "stretch" }
                    " an item's border box becomes its whole row span, the next "
                    "measurement reports that, and the item grows without bound."
                }
                Text {
                    "A zone filling a named area is a query container, which means "
                    Code { source: "contain: layout style inline-size" }
                    ": it is a stacking context and the containing block for any "
                    "absolutely positioned descendant. Something inside a "
                    Code { source: "GridItem" }
                    " that positions itself against the page needs a portal, not a "
                    Code { source: "position: absolute" }
                    "."
                }
                Text {
                    "A zone used on its own, outside a "
                    Code { source: "Grid" }
                    ", is deliberately not a container: inline-size containment zeroes an "
                    "element's own contribution to its width, which collapses a "
                    "shrink-to-fit box to nothing. That also means "
                    Code { source: "sp()" }
                    " needs a zone with an area - there is nothing to query otherwise, and "
                    "the base span stands."
                }
                Text {
                    "Build templates outside the render - a "
                    Code { source: "StaticGridTemplate" }
                    " static, as every example here does. Rebuilding one per render "
                    "re-parses the matrix and hands "
                    Code { source: "Grid" }
                    " a new value every time."
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum SpanArea {
    Row,
}

impl GridArea for SpanArea {
    fn name(&self) -> &'static str {
        "row"
    }
}

static SPANS: StaticGridTemplate<SpanArea> =
    StaticGridTemplate::new(|template| template.row(|row| row.cell(SpanArea::Row)));

const CARD_SPAN: SpanValue = sp()
    .base(GridSpan::Full)
    .sm(GridSpan::Half)
    .md(GridSpan::Third);

const RESPONSIVE_SOURCE: &str = r#"// Three cards, one span value. Wide: three across. Tablet: two, and C
// wraps. Phone: one per row.
const CARD_SPAN: SpanValue = sp().base(GridSpan::Full).sm(GridSpan::Half).md(GridSpan::Third);

rsx! {
    Grid { template: SPANS.clone(),
        GridZone { area: SpanArea::Row,
            GridItem { span: CARD_SPAN, "A" }
            GridItem { span: CARD_SPAN, "B" }
            GridItem { span: CARD_SPAN, "C" }
        }
    }
}"#;
