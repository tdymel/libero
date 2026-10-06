use crate::components::{
    Control, Demo, DemoFile, DemoValues, DocPage, DocSection, Wrap, a11y, prop, props,
};
use libero::components::Pictogram;
use pictogram_icons_lucide as lucide;

use dioxus::prelude::*;
use libero::{
    components::{
        Accordion, AccordionOpen, Box, Button, Code, Flex, Icon, OptionLabel, OptionList, Options,
        Text,
    },
    sx::sx,
};

/// The page's own source: the printed parts are cut from its live demo.
const FILE: DemoFile = DemoFile(include_str!("accordion.rs"));

// demo-code: step start
#[derive(Clone, PartialEq, Options)]
enum Step {
    Shipping,
    #[option(label = "Payment method")]
    Payment,
    Review,
}
// demo-code: step end

// demo-code: icon start
impl Step {
    fn icon(&self) -> Element {
        match self {
            Step::Shipping => rsx! { Pictogram { icon: lucide::truck::outlined } },
            Step::Payment => rsx! { Pictogram { icon: lucide::credit_card::outlined } },
            Step::Review => rsx! { Pictogram { icon: lucide::clipboard_check::outlined } },
        }
    }
}
// demo-code: icon end

// A trigger is a `button`, so its content stays phrasing: a `span` `Box`, not a `Flex` `div`.
// The inline-flex row centres the icon on the label.
fn rich(step: Step) -> OptionLabel {
    // demo-code: rich start
    OptionLabel::rich(
        step.label(),
        rsx! {
            Box {
                component: "span",
                sx: sx().display("inline-flex").align_items("center").gap("sm"),
                Icon { variant: "standard", size: "sm", {step.icon()} }
                "{step.label()}"
            }
        },
    )
    // demo-code: rich end
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
            source: "libero/src/components/data_display/accordion",
            markdown: "/md/accordion.md",
            properties: vec![
                props("Accordion", vec![
                    prop("open", "AccordionOpen<T>")
                        .default("One(None)")
                        .doc("Which sections are open. `One(Option<T>)` holds at most one, `Many(Vec<T>)` any number. Controlled, so pair it with `onchange`. An `Option<T>` or a `Vec<T>` converts into one."),
                    prop("onchange", "EventHandler<AccordionOpen<T>>")
                        .default("None")
                        .doc("Called with the whole new open set, in the same mode, ready to store."),
                    prop("panel", "Callback<T, Element>")
                        .default("None")
                        .doc("A section's body. A closed panel is not mounted, so it keeps no state. A closure that captures a plain value redraws only when its section's value or open state changes."),
                    prop("options", "OptionSource<T>")
                        .default("T::options()")
                        .doc("The sections to show. A `Vec<T>` converts. An `OptionList<T>` can disable a section, which renders, cannot be toggled and stays a tab stop. Groups are drawn flat."),
                    prop("option_label", "Callback<T, OptionLabel>")
                        .default("T::label()")
                        .doc("Renames a section. `OptionLabel::rich` draws the trigger as rsx and still names it."),
                    prop("heading", "HtmlTag").default("h3").doc("The heading around each trigger, `h1` to `h6`."),
                    prop("size", "Size").default("md").doc("Type and padding of the triggers and panels."),
                ]),
            ],
            accessibility: a11y()
                .key(["Tab"], "Moves between the triggers: each one is a tab stop.")
                .key(["Enter", "Space"], "Toggles the focused section.")
                .key(["Up", "Down"], "Moves to the previous or next trigger, without toggling.")
                .key(["Home", "End"], "Jumps to the first or last trigger, without toggling.")
                .handles([
                    "Every open panel is a region named by its trigger.",
                    "Panel content wider than the panel wraps, or scrolls inside the panel, instead of being cut off.",
                ])
                .must([
                    "Pick the heading level the page outline needs, then the size. `h3` assumes a section title above the accordion.",
                    "With `OptionLabel::rich`, make the name contain the visible text, since it replaces the drawn label (WCAG 2.5.3).",
                ])
                .example("A FAQ under the section title \"Shipping\", with the default `h3` heading: a screen reader lists each question as a level 3 heading, and Enter opens its answer as a region named by the question.")
                .limits([
                    "A `Many` accordion with a dozen open sections makes a long landmark list.",
                ]),
            lead: rsx! {
                Text {
                    "Sections over an enum, each a heading with a button that opens its "
                    "panel. "
                    Code { source: "panel" }
                    " is a match over the same enum, so a forgotten section is a compile "
                    "error. A closed panel is not mounted, so what it held is gone when it "
                    "closes."
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
                    let icon = match values.str("rich_label") == "true" {
                        true => format!("{}\n\n", FILE.section("icon")),
                        false => String::new(),
                    };
                    let step = FILE.section("step");
                    format!("{step}\n\n{icon}let mut open = use_signal(|| {initial});\n\n{source}")
                }),
                fixed: vec![
                    "open: open()".to_string(),
                    "onchange: move |next| open.set(next)".to_string(),
                ],
                controls: vec![
                    Control::toggle("mode", ["one", "many"])
                        .labels(["One", "Many"])
                        .code(|_, values| {
                            let mode = if many(values) { "many" } else { "one" };
                            vec![format!("panel: {}", FILE.section(mode).trim_end_matches(','))]
                        }),
                    Control::sizes("size")
                        .default("md"),
                    Control::toggle("heading", ["h2", "h3", "h4"])
                        .labels(["H2", "H3", "H4"])
                        .default("h3"),
                    // The flag lives inside `options`, so the switch stands
                    // for one named section rather than for a prop of its own.
                    Control::switch("disabled_option").code(|_, values| {
                        if values.str("disabled_option") == "true" {
                            vec![format!("options: {}", FILE.section("disabled"))]
                        } else {
                            vec![]
                        }
                    }),
                    Control::switch("rich_label").code(|_, values| {
                        if values.str("rich_label") == "true" {
                            vec![format!("option_label: |step: Step| {}", FILE.section("rich"))]
                        } else {
                            vec![]
                        }
                    }),
                ],
                render: move |values: DemoValues| {
                    let is_many = many(&values);
                    let mut open = if is_many { several } else { one };
                    let panel = if is_many {
                        Callback::new(
                            // demo-code: many start
                            |step: Step| match step {
                                Step::Shipping => rsx! { Text { "Where should the parcel go?" } },
                                Step::Payment => rsx! { Text { "Card, invoice or bank transfer." } },
                                Step::Review => rsx! { Text { "Check the order, then place it." } },
                            },
                            // demo-code: many end
                        )
                    } else {
                        Callback::new(
                            // demo-code: one start
                            move |step: Step| match step {
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
                            },
                            // demo-code: one end
                        )
                    };
                    let options = if values.str("disabled_option") == "true" {
                        // demo-code: disabled start
                        OptionList::from_options().disabling(|step| *step == Step::Review)
                        // demo-code: disabled end
                    } else {
                        OptionList::from_options()
                    };
                    // Keyed by mode, so a switch remounts it: `panel` captures `is_many` and
                    // the mode's signal, and a section redraws only when it changes.
                    rsx! {
                        for mode in [is_many] {
                            Accordion {
                                key: "{mode}",
                                size: values.str("size"),
                                heading: values.str("heading"),
                                options: options.clone(),
                                option_label: (values.str("rich_label") == "true").then(|| Callback::new(rich)),
                                open: open(),
                                onchange: move |next| open.set(next),
                                panel,
                            }
                        }
                    }
                },
            }

            DocSection {
                title: "Open sections",
                Text {
                    Code { source: "open" }
                    " is controlled, and its variant is the mode. "
                    Code { source: "AccordionOpen::One" }
                    " holds at most one section, so opening another closes the first. "
                    Code { source: "AccordionOpen::Many" }
                    " toggles each on its own. "
                    Code { source: "onchange" }
                    " hands back the whole new set."
                }
            }
        }
    }
}
