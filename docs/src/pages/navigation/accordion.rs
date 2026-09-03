use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, prop, props};
use crate::icons::FileIcon;
use dioxus::prelude::*;
use libero::components::{
    Accordion, AccordionOpen, Button, Code, Flex, Icon, OptionLabel, Options, Text,
};

/// The sections are the enum, so the snippet is a lie without it.
const STEP_ENUM: &str = r#"#[derive(Clone, PartialEq, Options)]
enum Step {
    Shipping,
    #[option(label = "Payment method")]
    Payment,
    Review,
}

"#;

/// In `One` mode each step's button opens the next, which closes this one
/// with focus inside it - the case the component hands focus back for.
const ONE_PANEL: &str = r#"panel: move |step: Step| match step {
    Step::Shipping => rsx! {
        Flex { gap: "sm", align: "flex-start",
            Text { "Where should the parcel go?" }
            Button { onclick: move |_| open.set(Some(Step::Payment).into()), "Continue" }
        }
    },
    Step::Payment => rsx! {
        Flex { gap: "sm", align: "flex-start",
            Text { "Card, invoice or bank transfer." }
            Button { onclick: move |_| open.set(Some(Step::Review).into()), "Continue" }
        }
    },
    Step::Review => rsx! { Text { "Check the order, then place it." } },
}"#;

const MANY_PANEL: &str = r#"panel: |step: Step| match step {
    Step::Shipping => rsx! { Text { "Where should the parcel go?" } },
    Step::Payment => rsx! { Text { "Card, invoice or bank transfer." } },
    Step::Review => rsx! { Text { "Check the order, then place it." } },
}"#;

// A trigger is a `button`, so its content has to stay phrasing content: `Icon`
// is an inline-flex `span`.
const RICH: &str = r#"label: |step: Step| OptionLabel::rich(
    step.label(),
    rsx! {
        Icon { variant: "transparent", size: "sm", FileIcon {} }
        "{step.label()}"
    },
)"#;

#[derive(Clone, PartialEq, Options)]
enum Step {
    Shipping,
    #[option(label = "Payment method")]
    Payment,
    Review,
}

fn rich(step: Step) -> OptionLabel {
    OptionLabel::rich(
        step.label(),
        rsx! {
            Icon { variant: "transparent", size: "sm", FileIcon {} }
            "{step.label()}"
        },
    )
}

fn many(values: &DemoValues) -> bool {
    values.str("mode") == "many"
}

#[component]
pub fn AccordionPage() -> Element {
    // One signal per mode, so switching the mode control does not have to
    // convert one open set into the other.
    let one = use_signal(|| AccordionOpen::One(Some(Step::Shipping)));
    let several = use_signal(|| AccordionOpen::Many(vec![Step::Shipping]));

    rsx! {
        DocPage {
            title: "Accordion",
            source: "libero/src/components/navigation/accordion",
            markdown: "/md/accordion.md",
            properties: vec![
                props("Accordion", vec![
                    prop("open", "AccordionOpen<T>")
                        .default("One(None)")
                        .doc("Which sections are expanded. The variant is the mode: `One(Option<T>)` holds at most one, `Many(Vec<T>)` any number. Strictly controlled - pair it with `onchange`."),
                    prop("onchange", "EventHandler<AccordionOpen<T>>").doc("Called with the whole new open set, in the same mode, ready to store."),
                    prop("panel", "Callback<T, Element>").doc("A section's body. A closed panel's content is never mounted, so it keeps no state."),
                    prop("sections", "Vec<T>").default("T::options()").doc("The sections to show."),
                    prop("label", "Callback<T, OptionLabel>")
                        .default("T::label()")
                        .doc("Overrides what the derive named a section. `OptionLabel::rich` draws the trigger as rsx and still names it."),
                    prop("disabled", "Vec<T>").doc("Sections that render but cannot be toggled. They stay tab stops."),
                    prop("heading", "HtmlTag").default("h3").doc("The heading around each trigger, `h1`..`h6`."),
                    prop("size", "Size").default("md").doc("Type and padding of the triggers and panels."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A list of sections over an enum, each a heading with a button that expands "
                    "its panel. The sections are the enum's variants and "
                    Code { source: "panel" }
                    " is a match over the same type, so a forgotten section is a compile error. "
                    "A closed panel's content is not mounted: what it held is gone when it closes."
                }
                Text {
                    Code { source: "open" }
                    " is strictly controlled, and its variant is the mode: "
                    Code { source: "AccordionOpen::One" }
                    " holds at most one section, so opening another closes the first by "
                    "construction; "
                    Code { source: "AccordionOpen::Many" }
                    " toggles each on its own. "
                    Code { source: "onchange" }
                    " hands back the whole new set in the same mode."
                }
                Text {
                    "Each trigger sits in a heading, "
                    Code { source: "h3" }
                    " unless "
                    Code { source: "heading" }
                    " says otherwise - pick the level the page outline needs first; "
                    Code { source: "size" }
                    " only changes the look. Every trigger is a tab stop; Up and Down move "
                    "between triggers and Home and End jump to the ends, without toggling. Every "
                    "panel is a "
                    Code { source: "role=\"region\"" }
                    " named by its trigger, so a screen reader lists each open one as a landmark "
                    "- with a dozen sections open at once in "
                    Code { source: "Many" }
                    " mode that list gets long."
                }
            },
            Demo {
                component: "Accordion",
                children_text: "",
                wrap: Wrap(|values: &DemoValues, source: &str| {
                    let initial = if many(values) {
                        "AccordionOpen::Many(vec![Step::Shipping])"
                    } else {
                        "AccordionOpen::One(Some(Step::Shipping))"
                    };
                    format!("{STEP_ENUM}let mut open = use_signal(|| {initial});\n\n{source}")
                }),
                fixed: vec![
                    "open: open()".to_string(),
                    "onchange: move |next| open.set(next)".to_string(),
                ],
                controls: vec![
                    Control::toggle("mode", ["one", "many"])
                        .labels(["One", "Many"])
                        .code(|_, values| {
                            vec![if many(values) { MANY_PANEL } else { ONE_PANEL }.to_string()]
                        }),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::toggle("heading", ["h2", "h3", "h4"]).default("h3"),
                    Control::switch("disabled").code(|_, values| {
                        if values.str("disabled") == "true" {
                            vec!["disabled: vec![Step::Review]".to_string()]
                        } else {
                            vec![]
                        }
                    }),
                    Control::switch("rich_label").code(|_, values| {
                        if values.str("rich_label") == "true" {
                            vec![RICH.to_string()]
                        } else {
                            vec![]
                        }
                    }),
                ],
                render: move |values: DemoValues| {
                    let is_many = many(&values);
                    let mut open = if is_many { several } else { one };
                    let panel = move |step: Step| match step {
                        Step::Shipping if !is_many => rsx! {
                            Flex { gap: "sm", align: "flex-start",
                                Text { "Where should the parcel go?" }
                                Button {
                                    onclick: move |_| open.set(Some(Step::Payment).into()),
                                    "Continue"
                                }
                            }
                        },
                        Step::Payment if !is_many => rsx! {
                            Flex { gap: "sm", align: "flex-start",
                                Text { "Card, invoice or bank transfer." }
                                Button {
                                    onclick: move |_| open.set(Some(Step::Review).into()),
                                    "Continue"
                                }
                            }
                        },
                        Step::Shipping => rsx! { Text { "Where should the parcel go?" } },
                        Step::Payment => rsx! { Text { "Card, invoice or bank transfer." } },
                        Step::Review => rsx! { Text { "Check the order, then place it." } },
                    };
                    rsx! {
                        Accordion {
                            size: values.str("size"),
                            heading: values.str("heading"),
                            disabled: if values.str("disabled") == "true" {
                                vec![Step::Review]
                            } else {
                                vec![]
                            },
                            label: (values.str("rich_label") == "true").then(|| Callback::new(rich)),
                            open: open(),
                            onchange: move |next| open.set(next),
                            panel,
                        }
                    }
                },
            }
        }
    }
}
