use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, indent, prop, props};
use crate::icons::{CheckmarkIcon, CodeIcon, FileIcon, GitHubIcon};
use dioxus::prelude::*;
use libero::components::{Code, Text, Timeline, TimelineEvent, TimelineLine};

/// The four fixed events, as the code block prints them. `Wrap` substitutes
/// the bullet lines and the line style, so what is printed is what the
/// controls produced.
fn items_code(values: &DemoValues) -> String {
    let bullets = values.str("bullets") == "true";
    let dashed = values.str("line") == "dashed";

    let rows = [
        ("Pushed to main", "3 commits", "CodeIcon"),
        ("Review requested", "@tom", "GitHubIcon"),
        ("Deployed", "v2.4.0 to production", "CheckmarkIcon"),
        ("Rolled back", "reverted in 4 minutes", "FileIcon"),
    ];

    let mut out = String::from("items: vec![\n");
    for (index, (title, content, icon)) in rows.iter().enumerate() {
        out.push_str(&format!(
            "    TimelineEvent::new(\"{title}\")\n        \
             .content(rsx! {{ Text {{ \"{content}\" }} }})"
        ));
        if bullets {
            out.push_str(&format!("\n        .bullet(rsx! {{ {icon} {{}} }})"));
        }
        // Only the last event's line would be visible if it had one, so the
        // style goes on the second-to-last - the one whose connector reaches
        // the final bullet.
        if dashed && index == rows.len() - 2 {
            out.push_str("\n        .line(TimelineLine::Dashed)");
        }
        out.push_str(",\n");
    }
    out.push(']');
    out
}

/// `items` is not a control, so `generate_code` never prints it - but it is
/// most of what the reader needs. The generated block is reopened and the
/// whole `vec![..]` literal spliced in before the closing brace, so the
/// snippet is the one that produces the preview.
fn wrap_page(values: &DemoValues, source: &str) -> String {
    let items = indent(&items_code(values));
    match source.strip_suffix('}') {
        // The props form, `Timeline {\n    ..\n}`.
        Some(open) if source.contains('\n') => format!("{open}{items}}}"),
        // Every control at its default, so `generate_code` emitted
        // `Timeline {}` on one line.
        _ => format!("Timeline {{\n{items}}}"),
    }
}

fn demo_items(values: &DemoValues) -> Vec<TimelineEvent> {
    let bullets = values.str("bullets") == "true";
    let dashed = values.str("line") == "dashed";

    let rows: [(&str, &str); 4] = [
        ("Pushed to main", "3 commits"),
        ("Review requested", "@tom"),
        ("Deployed", "v2.4.0 to production"),
        ("Rolled back", "reverted in 4 minutes"),
    ];

    rows.iter()
        .enumerate()
        .map(|(index, (title, content))| {
            let content = content.to_string();
            let mut event = TimelineEvent::new(*title).content(rsx! { Text { "{content}" } });
            if bullets {
                // Matched by index rather than looked up: each icon is its own
                // component, and the printed code names it directly.
                event = event.bullet(match index {
                    0 => rsx! { CodeIcon {} },
                    1 => rsx! { GitHubIcon {} },
                    2 => rsx! { CheckmarkIcon {} },
                    _ => rsx! { FileIcon {} },
                });
            }
            if dashed && index == rows.len() - 2 {
                event = event.line(TimelineLine::Dashed);
            }
            event
        })
        .collect()
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
                    prop("active", "usize")
                        .doc("The current event, strictly controlled: bind it to whatever already knows how far along the process is. Bullets `0..=active` and the connectors `0..active` draw active; out of range clamps to the last event, so \"step 7 of 4\" means finished."),
                    prop("align", "TimelineAlign")
                        .default("theme.timeline.align")
                        .doc("`\"left\"`, `\"right\"`, or `\"alternate\"` - content either side of a centred rail. `\"alternate\"` alternates at every width and fills its parent, so a narrower parent is how you make it narrower."),
                    prop("color", "ThemeAwareValue")
                        .default("theme.timeline.color")
                        .doc("The active accent. A per-event `.color(..)` overrides it."),
                    prop("bullet_size", "Size")
                        .default("theme.timeline.bullet_size")
                        .doc("Bullet diameter."),
                    prop("radius", "Size")
                        .default("theme.timeline.radius")
                        .doc("Bullet corner radius; `xl` is the dot."),
                    prop("gap", "Size")
                        .default("theme.timeline.gap")
                        .doc("Space between events, which is also each connector's length."),
                ]),
                props("TimelineEvent", vec![
                    prop("new(title)", "impl Into<OptionLabel>")
                        .default("required")
                        .doc("The event's name, optionally with its own rendering."),
                    prop(".content(Element)", "Element")
                        .doc("The body below the title."),
                    prop(".bullet(Element)", "Element")
                        .doc("An icon or avatar inside the bullet instead of the dot. A bullet with a child inverts when active."),
                    prop(".color(value)", "impl Into<ThemeAwareValue>")
                        .doc("This event's own accent - an error step in an otherwise unremarkable run."),
                    prop(".line(TimelineLine)", "TimelineLine")
                        .default("Solid")
                        .doc("The connector below this event: `Solid`, `Dashed` or `Dotted`."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "An ordered list of events drawn against a rail. It renders an "
                    Code { source: "<ol role=\"list\">" }
                    " - the rail shows position and count visually, and the list is how a "
                    "screen-reader user gets the same two facts. "
                    Code { source: "active" }
                    " names the current event: bullets up to and including it fill with the "
                    "accent, and the connectors between them draw in it, so the rail reads as "
                    "progress rather than as a highlight, and not by colour alone. Bullets are decorative and hidden from the "
                    "accessibility tree; the title is the text. A custom "
                    Code { source: ".bullet(..)" }
                    " is hidden too, so never put anything focusable in one - "
                    Code { source: "aria-hidden" }
                    " does not remove an element from the tab order, and a button in a bullet "
                    "would stay tabbable while announcing as nothing. Interactive content "
                    "belongs in "
                    Code { source: ".content(..)" }
                    "."
                }
            },
            Demo {
                component: "Timeline",
                children_text: "",
                // `Alternate` centres a rail in the `<ol>`'s whole width, and
                // the side-by-side preview never exceeds 498px at any viewport
                // - which is how the mode shipped undemonstrable. Wide, the
                // list has room for the alternation to read.
                wide_preview: true,
                controls: vec![
                    Control::slider("active", ["none", "0", "1", "2", "3"])
                        .default("2")
                        .code(|_, values| match values.str("active").as_str() {
                            "none" => vec![],
                            value => vec![format!("active: {value}")],
                        }),
                    Control::toggle("align", ["left", "right", "alternate"])
                        .labels(["Left", "Right", "Alternate"]),
                    // No `UNSET` swatch: unset resolves to `primary`, and a
                    // white swatch beside five colours reads as a sixth
                    // colour choice rather than as "no prop". `primary` is
                    // the first option, so it is the default, and
                    // `generate_code` omits a control at its default - the
                    // block prints no `color:` line there, exactly as the
                    // unset state did.
                    Control::color("color"),
                    Control::slider("bullet_size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("xl"),
                    Control::slider("gap", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("xl"),
                    // Neither is a prop - both change what the `items` vec
                    // holds, which `Wrap` prints in full.
                    Control::switch("bullets").code(|_, _| vec![]),
                    Control::toggle("line", ["solid", "dashed"])
                        .labels(["Solid", "Dashed"])
                        .code(|_, _| vec![]),
                ],
                wrap: Wrap(wrap_page),
                render: move |values: DemoValues| {
                    let items = demo_items(&values);
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
