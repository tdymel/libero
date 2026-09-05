use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{
        Box, Grid, GridArea, GridItem, GridSpan, GridZone, HtmlTag, SpanValue, StaticGridTemplate,
        Text, sp,
    },
    sx::sx,
};

/// The zone's children, printed verbatim - their point is that they are
/// *different heights*, which is what `masonry` reacts to.
// snippet: item #[component] fn Card(lines: usize, children: Element) -> Element { rsx! { {children} } }
// snippet: in GridZone { .. }
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
    let wall = |values: &DemoValues| values.str("layout") != "wall";
    vec![
        // Not a prop: which of three layouts the preview shows. Only the
        // masonry wall has knobs; the other two print their whole source.
        Control::toggle("layout", ["wall", "page", "responsive"])
            .labels(["Masonry wall", "Named areas", "Responsive spans"])
            .code(|_, _| vec![]),
        Control::switch("masonry").hidden_when(wall),
        Control::switch("dense").hidden_when(wall),
        Control::slider("gap", ["xs", "sm", "md", "lg", "xl", "xxl"])
            .default("md")
            .hidden_when(wall),
    ]
}

fn wrap_layout(values: &DemoValues, source: &str) -> String {
    match values.str("layout").as_str() {
        "page" => TEMPLATE_SOURCE.to_string(),
        "responsive" => RESPONSIVE_SOURCE.to_string(),
        _ => source.to_string(),
    }
}

#[component]
pub fn GridPage() -> Element {
    rsx! {
        DocPage {
            title: "Grid",
            source: "libero/src/components/layout/grid",
            markdown: "/md/grid.md",
            properties: vec![
                props("Grid", vec![
                    prop("template", "GridTemplate").doc("The named-area matrix. Each row shares its width equally between its cells, and `cells(area, n)` gives one area several; rows of different lengths reconcile to their least common multiple. `build` rejects an area that is not a rectangle, or rows needing more than twelve columns. Build it once outside the render, as a `StaticGridTemplate` static."),
                    prop("gap", "Size").default("md").doc("Between zones."),
                    prop("component", "HtmlTag").default("div").doc("Overrides the root element."),
                    prop("children", "Element").doc("`GridZone`s."),
                ]),
                props("GridZone", vec![
                    prop("area", "AreaName").doc("Which of the parent `Grid`'s areas this fills. Zones land by name, so their order is only the reading and tab order; an unknown area warns and auto-places. A zone in an area is a query container, so it is a stacking context and the containing block of any absolutely positioned descendant. Omit it to use the zone on its own, without a `Grid` - a plain masonry wall needs no template."),
                    prop("dense", "bool").default("false").doc("Backfill gaps a wider item left behind, moving items sideways only. Pure CSS, no measurement."),
                    prop("masonry", "bool").default("false").doc("Measure item heights and pack them with no vertical dead space. Costs a `ResizeObserver` per item; without a DOM the zone renders as an ordinary grid. Its height follows its items, so scroll inside a `GridItem`, not around the zone, and never set `align-self` or `margin-bottom` on an item."),
                    prop("gap", "Size").default("md").doc("Between items."),
                    prop("component", "HtmlTag").default("div").doc("Overrides the root element."),
                    prop("children", "Element").doc("`GridItem`s."),
                ]),
                props("GridItem", vec![
                    prop("span", "SpanValue").default("full").doc("Width, in twelfths of the zone. A `GridSpan`, which is closed, so every value is an exact twelfth; or `sp()` for a span keyed off the zone's own width through a container query, not the window's. `sp()` needs a zone with an `area`."),
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
                Text {
                    "`masonry` and `dense` both make the visual order diverge from the DOM order, "
                    "and Tab follows the DOM - don't reach for either where the reading order "
                    "carries meaning."
                }
            },
            Demo {
                component: "GridZone",
                children_text: "",
                children_code: CARDS,
                controls: controls(),
                wrap: Wrap(wrap_layout),
                // A span breakpoint measures the zone, and `sm` is 48rem: beside
                // the controls the zone never gets that wide.
                wide_preview: true,
                render: move |values: DemoValues| match values.str("layout").as_str() {
                    // Written content-first and the sidebar last, and still
                    // rendered header on top: zones land by name.
                    "page" => rsx! {
                        Grid {
                            template: PAGE.clone(),
                            sx: sx().width("100%").background("grey.1").padding("12px").border_radius("sm"),
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
                    },
                    // The zone is as wide as the preview, so the cards walk
                    // down the ladder as the page narrows.
                    "responsive" => rsx! {
                        Grid {
                            template: SPANS.clone(),
                            sx: sx().width("100%").background("grey.1").padding("12px").border_radius("sm"),
                            GridZone {
                                area: SpanArea::Row,
                                gap: "xs",
                                for name in ["A", "B", "C"] {
                                    GridItem { key: "{name}", span: CARD_SPAN, {panel(name, 32)} }
                                }
                            }
                        }
                    },
                    _ => rsx! {
                        GridZone {
                            masonry: values.str("masonry") == "true",
                            dense: values.str("dense") == "true",
                            gap: values.str("gap"),
                            {cards()}
                        }
                    },
                },
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

// snippet: ignore - `SPANS` is a template the page builds out of sight
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
