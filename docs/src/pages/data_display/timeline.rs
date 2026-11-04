use crate::components::{Control, Demo, DemoFile, DemoValues, DocPage, Wrap, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, Text, Timeline, TimelinePart};
use libero::theme::{Responsive, Size, TimelineAlign, responsive};
use libero::use_theme;

mod demo;
use demo::demo_items;

/// `sm` below `md`, the chosen bullet_size from it, while `responsive` is on.
fn bullet_size_of(values: &DemoValues) -> Responsive<Size> {
    let bullet_size = Size::from(values.str("bullet_size").as_str());
    match values.str("responsive") == "true" {
        true => responsive(Size::Sm).md(bullet_size),
        false => bullet_size.into(),
    }
}

fn bullet_size_code(control: &Control, values: &DemoValues) -> Vec<String> {
    match (
        values.str("responsive") == "true",
        values.str("bullet_size"),
    ) {
        (true, bullet_size) => vec![format!(
            "bullet_size: responsive(Size::Sm).md(Size::{:?})",
            Size::from(bullet_size.as_str())
        )],
        (false, bullet_size) if bullet_size == control.default => vec![],
        (false, bullet_size) => vec![format!("bullet_size: {bullet_size:?}")],
    }
}

fn align_word(word: &str) -> TimelineAlign {
    match word {
        "end" => TimelineAlign::End,
        "alternate" => TimelineAlign::Alternate,
        _ => TimelineAlign::Start,
    }
}

/// `start` below `md`, the chosen align from it, while `responsive` is on.
fn align_of(values: &DemoValues) -> Responsive<TimelineAlign> {
    let align = align_word(&values.str("align"));
    match values.str("responsive") == "true" {
        true => responsive(TimelineAlign::Start).md(align),
        false => align.into(),
    }
}

fn align_code(control: &Control, values: &DemoValues) -> Vec<String> {
    match (values.str("responsive") == "true", values.str("align")) {
        (true, align) => vec![format!(
            "align: responsive(TimelineAlign::Start).md(TimelineAlign::{:?})",
            align_word(&align)
        )],
        (false, align) if align == control.default => vec![],
        (false, align) => vec![format!("align: {align:?}")],
    }
}

fn switches(values: &DemoValues) -> (bool, bool, bool) {
    (
        values.str("bullets") == "true",
        values.str("line") == "dashed",
        values.str("opposite") == "true",
    )
}

/// Splices `items`, which `generate_code` never prints, into the generated block.
fn wrap_page(values: &DemoValues, source: &str) -> String {
    let (bullets, dashed, opposite) = switches(values);
    let items = format!("    items: demo_items({bullets}, {dashed}, {opposite}),\n");
    match source.strip_suffix('}') {
        // The props form, `Timeline {\n    ..\n}`.
        Some(open) if source.contains('\n') => format!("{open}{items}}}"),
        // Every control at its default, so `generate_code` emitted
        // `Timeline {}` on one line.
        _ => format!("Timeline {{\n{items}}}"),
    }
}

#[component]
pub fn TimelinePage() -> Element {
    let theme = use_theme();
    rsx! {
        DocPage {
            title: "Timeline",
            source: "libero/src/components/data_display/timeline/timeline.rs",
            markdown: "/md/timeline.md",
            properties: vec![
                props("Timeline", vec![
                    prop("items", "Vec<TimelineEvent>")
                        .default("vec![]")
                        .doc("The events, in render order."),
                    prop("active", "Option<usize>")
                        .default("None")
                        .doc("The current event. Bullets up to and including it, and the connectors between them, draw in the accent. An index past the end clamps to the last event, so \"step 7 of 4\" means finished."),
                    prop("align", "Responsive<TimelineAlign>")
                        .default("start")
                        .doc("`start`, `end`, or `alternate` for content on both sides of a centred rail. Mirrored in a right-to-left layout. `alternate` fills its parent, so a narrower parent makes it narrower. With an event's `.opposite(..)`, every `align` draws as `start` below `sm`. Or one per breakpoint, `align: responsive(TimelineAlign::Start).md(TimelineAlign::Alternate)`. Breakpoints follow the window, not the parent. With `opposite` content, the reading order and each body's side follow it too; a server-rendered page shows the base until it loads."),
                    prop("color", "ThemeAwareValue")
                        .default("primary")
                        .doc("The active accent. An event's own `.color(..)` overrides it."),
                    prop("bullet_size", "Responsive<Size>")
                        .default("md")
                        .doc("Bullet diameter. Or one per breakpoint, `bullet_size: responsive(Size::Sm).md(Size::Lg)`. Breakpoints follow the window, not the parent."),
                    prop("radius", "Responsive<ThemeAwareValue>")
                        .default(theme.timeline.radius.as_str())
                        .doc("Bullet corner radius, or any CSS, e.g. `radius: \"0\"`. `xl` is a dot. Or one per breakpoint, `radius: responsive(Size::Xs).md(Size::Xl)`. Breakpoints follow the window, not the parent."),
                    prop("gap", "Responsive<ThemeAwareValue>")
                        .default("xl")
                        .doc("Space between events, which is also each connector's length, or any CSS, e.g. `gap: \"0\"`. Or one per breakpoint, `gap: responsive(Size::Sm).md(Size::Xl)`. Breakpoints follow the window, not the parent."),
                    prop("parts", "Parts<TimelinePart>")
                        .doc("Styles for the inner parts in the Style API tab, under `sx`."),
                ])
                .parts("TimelinePart", vec![
                    (TimelinePart::Item, "One event's `<li>`."),
                    (TimelinePart::Bullet, "The dot, or the ring around an event's `bullet`."),
                    (TimelinePart::Body, "Title and content, beside the rail."),
                    (TimelinePart::Title, "The event's title."),
                    (TimelinePart::Opposite, "An event's `.opposite(..)` content, across the rail from the body."),
                ]),
                props("TimelineEvent", vec![
                    prop("new(title)", "impl Into<OptionLabel>")
                        .default("required")
                        .doc("The event's name, optionally with its own rendering, which a screen reader skips for the name."),
                    prop(".content(Element)", "Element")
                        .doc("The body below the title."),
                    prop(".opposite(Element)", "Element")
                        .doc("Content across the rail from the body, such as a time. One event setting it centres the rail for the whole timeline from the `sm` breakpoint up, with the body on the `align` side (`alternate` swaps sides per event). Below `sm` the rail runs down the start edge and the content takes only the width it needs, beside the body. It is read in order, after the body."),
                    prop(".bullet(Element)", "Element")
                        .doc("An icon or avatar inside the bullet in place of the dot. It inverts when active, so use an outlined glyph: a solid one covers the fill and the done bullet reads as a pending ring."),
                    prop(".color(value)", "impl Into<ThemeAwareValue>")
                        .doc("This event's own accent, such as an error step in an ordinary run."),
                    prop(".line(TimelineLine)", "TimelineLine")
                        .default("Solid")
                        .doc("The connector below this event, `Solid`, `Dashed` or `Dotted`."),
                ]),
            ],
            accessibility: a11y()
                .handles([
                    "The list gives a screen reader the position and count the rail shows.",
                    "Bullets are `aria-hidden`, and the title is the text. A custom `.bullet(..)` is hidden too.",
                ])
                .must([
                    "Never put anything focusable in a `.bullet(..)`. It would stay a tab stop with no name. Interactive content belongs in `.content(..)`.",
                    "Keep a rich title (`OptionLabel::rich`) decorative: it is hidden from a screen reader, which hears the name instead, so no link or button in it. Interactive content belongs in `.content(..)`.",
                    "Say an error or other status in the title or content. The `.color(..)` accent alone does not carry it.",
                ])
                .example("An order's history in a `Timeline` with `active` on the \"Shipped\" event: a screen reader reads each event's place in the list and marks \"Shipped\" as the current step. A failed delivery says \"Failed\" in its title, not only in red.")
                .limits([
                    "A pending bullet is filled with the `Paper` colour. On a page of another colour, such as a dark page behind a lighter `Paper`, the ring shows a disc of that colour; set `bullet_background` on the theme to the surface the timeline sits on.",
                    "A screen reader hears only which event is current, as `aria-current=\"step\"`. Done and pending are visual; put them in the title or content where they matter. With `active` past the end, the last event still reads as current.",
                ]),
            lead: rsx! {
                Text {
                    "An ordered list of events drawn against a rail. "
                    Code { source: "active" }
                    " marks the current event. Bullets up to it fill with the accent and "
                    "the connectors between them draw in it, so the rail reads as progress. "
                    "Done and pending bullets differ by shape too, not by color alone."
                }
            },
            Demo {
                component: "Timeline",
                children_text: "",
                // `Alternate` needs more than the side-by-side preview's 498px.
                wide_preview: true,
                controls: vec![
                    Control::slider("active", ["none", "0", "1", "2", "3"])
                        .default("2")
                        .code(|_, values| match values.str("active").as_str() {
                            "none" => vec![],
                            value => vec![format!("active: {value}")],
                        }),
                    // `alternate`, so the responsive `start` below `md` shows at the default switches.
                    Control::toggle("align", ["start", "end", "alternate"])
                        .labels(["Start", "End", "Alternate"])
                        .default("alternate")
                        .code(align_code),
                    // No `UNSET` swatch: unset is `primary`, the default. Hidden with no
                    // `active`, since nothing is drawn in the accent then.
                    Control::color("color").hidden_when(|values| values.str("active") == "none"),
                    Control::sizes("bullet_size")
                        .default("md").code(bullet_size_code),
                    // On, so the breakpoint switch shows without another switch: resize the window.
                    Control::switch("responsive").default("true").code(|_, _| vec![]),
                    Control::sizes("radius")
                        .default(theme.timeline.radius.as_str()),
                    Control::slider("gap", ["0", "xs", "sm", "md", "lg", "xl", "xxl"]).default("xl"),
                    // Neither is a prop - both change what the `items` vec
                    // holds, which `Wrap` prints in full.
                    // On: a time across the rail shows `.opposite(..)` at the default switches.
                    Control::switch("opposite")
                        .default("true")
                        .code(|_, _| vec![]),
                    Control::switch("bullets").code(|_, _| vec![]),
                    Control::toggle("line", ["solid", "dashed"])
                        .labels(["Solid", "Dashed"])
                        .code(|_, _| vec![]),
                ],
                wrap: Wrap(wrap_page),
                file: DemoFile(include_str!("timeline/demo.rs")),
                render: move |values: DemoValues| {
                    let (bullets, dashed, opposite) = switches(&values);
                    let items = demo_items(bullets, dashed, opposite);
                    let active = values.str("active").parse::<usize>().ok();

                    rsx! {
                        Timeline {
                            items,
                            active,
                            align: align_of(&values),
                            color: values.str("color"),
                            bullet_size: bullet_size_of(&values),
                            radius: values.str("radius"),
                            gap: values.str("gap"),
                        }
                    }
                },
            }
        }
    }
}
