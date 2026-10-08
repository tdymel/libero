use crate::components::{Control, Demo, DemoFile, DemoValues, DocPage, Wrap, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, Text, Timeline, TimelinePart};

mod demo;
use demo::demo_items;

fn switches(values: &DemoValues) -> (bool, bool) {
    (
        values.str("bullets") == "true",
        values.str("line") == "dashed",
    )
}

/// Splices `items`, which `generate_code` never prints, into the generated block.
fn wrap_page(values: &DemoValues, source: &str) -> String {
    let (bullets, dashed) = switches(values);
    let items = format!("    items: demo_items({bullets}, {dashed}),\n");
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
                    prop("align", "TimelineAlign")
                        .default("start")
                        .doc("`start`, `end`, or `alternate` for content on both sides of a centred rail. Mirrored in a right-to-left layout. `alternate` fills its parent, so a narrower parent makes it narrower."),
                    prop("color", "ThemeAwareValue")
                        .default("primary")
                        .doc("The active accent. An event's own `.color(..)` overrides it."),
                    prop("bullet_size", "Size")
                        .default("md")
                        .doc("Bullet diameter."),
                    prop("radius", "Size")
                        .default("xl")
                        .doc("Bullet corner radius. `xl` is a dot."),
                    prop("gap", "Size")
                        .default("xl")
                        .doc("Space between events, which is also each connector's length."),
                    prop("parts", "Parts<TimelinePart>")
                        .doc("Styles for the inner parts in the Style API tab, under `sx`."),
                ])
                .parts("TimelinePart", vec![
                    (TimelinePart::Item, "One event's `<li>`."),
                    (TimelinePart::Bullet, "The dot, or the ring around an event's `bullet`."),
                    (TimelinePart::Body, "Title and content, beside the rail."),
                    (TimelinePart::Title, "The event's title."),
                ]),
                props("TimelineEvent", vec![
                    prop("new(title)", "impl Into<OptionLabel>")
                        .default("required")
                        .doc("The event's name, optionally with its own rendering, which a screen reader skips for the name."),
                    prop(".content(Element)", "Element")
                        .doc("The body below the title."),
                    prop(".bullet(Element)", "Element")
                        .doc("An icon or avatar inside the bullet in place of the dot. It inverts when active."),
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
                    "Say an error or other status in the title or content. The `.color(..)` accent alone does not carry it.",
                ])
                .example("An order's history in a `Timeline` with `active` on the \"Shipped\" event: a screen reader reads each event's place in the list and marks \"Shipped\" as the current step. A failed delivery says \"Failed\" in its title, not only in red.")
                .limits([
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
                    Control::toggle("align", ["start", "end", "alternate"])
                        .labels(["Start", "End", "Alternate"]),
                    // No `UNSET` swatch: unset is `primary`, the default. Hidden with no
                    // `active`, since nothing is drawn in the accent then.
                    Control::color("color").hidden_when(|values| values.str("active") == "none"),
                    Control::sizes("bullet_size")
                        .default("md"),
                    Control::sizes("radius")
                        .default("xl"),
                    Control::sizes("gap").default("xl"),
                    // Neither is a prop - both change what the `items` vec
                    // holds, which `Wrap` prints in full.
                    Control::switch("bullets").code(|_, _| vec![]),
                    Control::toggle("line", ["solid", "dashed"])
                        .labels(["Solid", "Dashed"])
                        .code(|_, _| vec![]),
                ],
                wrap: Wrap(wrap_page),
                file: DemoFile(include_str!("timeline/demo.rs")),
                render: move |values: DemoValues| {
                    let (bullets, dashed) = switches(&values);
                    let items = demo_items(bullets, dashed);
                    let active = values.str("active").parse::<usize>().ok();

                    rsx! {
                        Timeline {
                            items,
                            active,
                            align: values.str("align"),
                            color: values.str("color"),
                            bullet_size: values.str("bullet_size"),
                            radius: values.str("radius"),
                            gap: values.str("gap"),
                        }
                    }
                },
            }
        }
    }
}
