use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{
        Box, Code, Grid, GridArea, GridItem, GridSpan, GridZone, HtmlTag, StaticGridTemplate, Text,
    },
    sx::sx,
    theme::{Responsive, responsive},
};

/// The zone's children, printed verbatim - their point is that they are
/// *different heights*, which is what `masonry` reacts to.
// snippet: item #[component] fn Card(lines: usize, children: Element) -> Element { children }
// snippet: in GridZone { .. }
const CARDS: &str = r#"GridItem { span: GridSpan::Third, Card { lines: 1, "A" } }
GridItem { span: GridSpan::TwoThirds, Card { lines: 4, "B" } }
GridItem { span: GridSpan::Third, Card { lines: 2, "C" } }
GridItem { span: GridSpan::Third, Card { lines: 3, "D" } }
GridItem { span: GridSpan::Half, Card { lines: 1, "E" } }
GridItem { span: GridSpan::Third, Card { lines: 2, "F" } }"#;

/// The preview pane shrink-wraps, and a zone has no width of its own.
// snippet: in GridZone { .. }
const WALL_SX: &str = r#"sx: sx().width("100%")"#;

/// Span, height and label per card, mixed so `dense` has a hole to backfill: E can't follow
/// C + D, leaving four columns that F fits.
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

// Written content-first, sidebar last. The template decides where each
// zone lands, so this order only sets the reading and tab order.
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
        // On at first, as the layout is labelled a masonry wall; the prop's own default is off.
        Control::switch("masonry")
            .default("true")
            .code(|_, values| match values.str("masonry").as_str() {
                "true" => vec!["masonry: true".to_string()],
                _ => vec![],
            })
            .hidden_when(wall),
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
                    prop("template", "GridTemplate").default("required").doc("The named-area matrix. Each row shares its width equally between its cells, and `cells(area, n)` gives one area several. An area must be a rectangle, and the rows need at most twelve columns. Build it once, as a `StaticGridTemplate` static."),
                    prop("gap", "Size").default("md").doc("Space between zones."),
                    prop("component", "HtmlTag").default("div").doc("The element to render."),
                    prop("children", "Element").doc("`GridZone`s."),
                ]),
                props("GridZone", vec![
                    prop("area", "AreaName").doc("The `Grid` area this zone fills. Zones land by name, so their order only sets the reading and tab order. A zone with an area is the containing block of any absolutely positioned descendant. Omit it to use the zone on its own, without a `Grid`, and give it a width."),
                    prop("dense", "bool").default("false").doc("Fills the gaps a wider item left, moving items sideways only."),
                    prop("masonry", "bool").default("false").doc("Packs items of different heights with no vertical gaps. The zone's height follows its items, so scroll inside a `GridItem`, not around the zone, and set no `align-self` or `margin-bottom` on an item. Keep it out of a container whose width follows its content: a scrollbar coming and going makes it measure again and again."),
                    prop("gap", "Size").default("md").doc("Space between items."),
                    prop("component", "HtmlTag").default("div").doc("The element to render."),
                    prop("children", "Element").doc("`GridItem`s."),
                ]),
                props("GridItem", vec![
                    prop("span", "Responsive<GridSpan>").default("full").doc("Width in twelfths of the zone. A `GridSpan` (`Full` 12, `ThreeQuarters` 9, `TwoThirds` 8, `Half` 6, `Third` 4, `Quarter` 3, `Sixth` 2, `Twelfth` 1), or `responsive(..)` for a span that follows the zone's width, not the window's. Its breakpoints need a zone with an `area`."),
                    prop("rows", "u8").doc("Height in rows of the zone's grid. Ignored in a masonry zone, which sets it from the measured height."),
                    prop("component", "HtmlTag").default("div").doc("The element to render."),
                    prop("children", "Element").doc("The item's content."),
                ]),
            ],
            accessibility: a11y()
                .handles(["`Grid`, `GridZone` and `GridItem` add no roles: `component` names the tag, such as `HtmlTag::Header` or `HtmlTag::Aside` for a landmark."])
                .must([
                    "Write the zones in reading order: the template places them anywhere, but screen readers and Tab follow the code.",
                    "Use `dense` only where order means nothing, such as a photo wall: a later item fills an earlier gap and shows before items it follows in the code.",
                    "Give two landmarks of the same kind an `aria-label` each, such as two `Aside` zones.",
                ]),
            lead: rsx! {
                Text {
                    "A layout matrix of named areas. "
                    Code { source: "Grid" }
                    " holds the shape, each "
                    Code { source: "GridZone" }
                    " is a twelve-column container of its own, and a "
                    Code { source: "GridItem" }
                    " takes a fraction of its zone. A zone also works alone, so a masonry "
                    "wall needs no template."
                }
                Text {
                    Code { source: "dense" }
                    " fills gaps by moving items out of DOM order, but Tab still follows "
                    "the DOM. Skip it where the reading order matters. "
                    Code { source: "masonry" }
                    " alone keeps the order, since each item starts no higher than the one "
                    "before it."
                }
            },
            // snippet: item #[component] fn Card(lines: usize, children: Element) -> Element { rsx! { {children} } }
            Demo {
                component: "GridZone",
                children_text: "",
                children_code: CARDS,
                fixed: vec![WALL_SX.to_string()],
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
                            sx: sx().width("100%").background("muted.1").padding("12px").border_radius("sm"),
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
                            sx: sx().width("100%").background("muted.1").padding("12px").border_radius("sm"),
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
                            sx: sx().width("100%"),
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

const CARD_SPAN: Responsive<GridSpan> = responsive(GridSpan::Full)
    .sm(GridSpan::Half)
    .md(GridSpan::Third);

// snippet: ignore - `SPANS` is a template the page builds out of sight
const RESPONSIVE_SOURCE: &str = r#"// Three cards, one span value. Wide: three across. Tablet: two, and C
// wraps. Phone: one per row.
const CARD_SPAN: Responsive<GridSpan> =
    responsive(GridSpan::Full).sm(GridSpan::Half).md(GridSpan::Third);

rsx! {
    Grid { template: SPANS.clone(),
        GridZone { area: SpanArea::Row,
            GridItem { span: CARD_SPAN, "A" }
            GridItem { span: CARD_SPAN, "B" }
            GridItem { span: CARD_SPAN, "C" }
        }
    }
}"#;
