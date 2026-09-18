use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, prop, props};
use crate::icons::{ClipboardCheckIcon, CreditCardIcon, TruckIcon};
use dioxus::prelude::*;
use libero::{
    components::{
        Accordion, AccordionOpen, Box, Button, Code, Flex, Icon, Kbd, List, ListItem, OptionLabel,
        OptionList, Options, Text,
    },
    sx::sx,
};

/// The flag sits on the option, inside the one `options` prop. A section
/// this list refuses, rather than a step the type refuses everywhere.
// snippet: after STEP_ENUM
// snippet: let mut open = use_signal(|| AccordionOpen::One(Some(Step::Shipping)));
// snippet: in Accordion { open: open(), onchange: move |next| open.set(next), panel: |_: Step| rsx! {}, .. }
const DISABLED_OPTION: &str =
    r#"options: OptionList::from_options().disabling(|step| *step == Step::Review)"#;

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
// snippet: after STEP_ENUM
// snippet: let mut open = use_signal(|| AccordionOpen::One(Some(Step::Shipping)));
// snippet: in Accordion { open: open(), onchange: move |next| open.set(next), .. }
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

// snippet: after STEP_ENUM
// snippet: let mut open = use_signal(|| AccordionOpen::Many(vec![Step::Shipping]));
// snippet: in Accordion { open: open(), onchange: move |next| open.set(next), .. }
const MANY_PANEL: &str = r#"panel: |step: Step| match step {
    Step::Shipping => rsx! { Text { "Where should the parcel go?" } },
    Step::Payment => rsx! { Text { "Card, invoice or bank transfer." } },
    Step::Review => rsx! { Text { "Check the order, then place it." } },
}"#;

// A trigger is a `button`, so its content has to stay phrasing content: a
// `Box` as a `span`, not a `Flex`, which is a `div`. The inline-flex row is
// what centres the icon on the label; beside bare text it sits on the
// baseline.
// snippet: after STEP_ENUM
// snippet: item impl Step { fn icon(&self) -> Element { rsx! {} } }
// snippet: let mut open = use_signal(|| AccordionOpen::One(Some(Step::Shipping)));
// snippet: in Accordion { open: open(), onchange: move |next| open.set(next), panel: |_: Step| rsx! {}, .. }
const RICH: &str = r#"option_label: |step: Step| OptionLabel::rich(
    step.label(),
    rsx! {
        Box {
            component: "span",
            sx: sx().display("inline-flex").align_items("center").gap("sm"),
            Icon { variant: "standard", size: "sm", {step.icon()} }
            "{step.label()}"
        }
    },
)"#;

/// Printed with the rich label, which calls it.
// snippet: after STEP_ENUM
// snippet: item #[component] fn TruckIcon() -> Element { rsx! {} }
// snippet: item #[component] fn CreditCardIcon() -> Element { rsx! {} }
// snippet: item #[component] fn ClipboardCheckIcon() -> Element { rsx! {} }
const STEP_ICON: &str = r#"impl Step {
    fn icon(&self) -> Element {
        match self {
            Step::Shipping => rsx! { TruckIcon {} },
            Step::Payment => rsx! { CreditCardIcon {} },
            Step::Review => rsx! { ClipboardCheckIcon {} },
        }
    }
}

"#;

#[derive(Clone, PartialEq, Options)]
enum Step {
    Shipping,
    #[option(label = "Payment method")]
    Payment,
    Review,
}

impl Step {
    fn icon(&self) -> Element {
        match self {
            Step::Shipping => rsx! { TruckIcon {} },
            Step::Payment => rsx! { CreditCardIcon {} },
            Step::Review => rsx! { ClipboardCheckIcon {} },
        }
    }
}

fn rich(step: Step) -> OptionLabel {
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
                        .doc("A section's body. A closed panel is not mounted, so it keeps no state."),
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
            lead: rsx! {
                Text {
                    "Sections over an enum, each a heading with a button that opens its "
                    "panel. "
                    Code { source: "panel" }
                    " is a match over the same enum, so a forgotten section is a compile "
                    "error. A closed panel is not mounted, so what it held is gone when it "
                    "closes."
                }
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
                        true => STEP_ICON,
                        false => "",
                    };
                    format!("{STEP_ENUM}{icon}let mut open = use_signal(|| {initial});\n\n{source}")
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
                    Control::toggle("heading", ["h2", "h3", "h4"])
                        .labels(["H2", "H3", "H4"])
                        .default("h3"),
                    // The flag lives inside `options`, so the switch stands
                    // for one named section rather than for a prop of its own.
                    Control::switch("disabled_option").code(|_, values| {
                        if values.str("disabled_option") == "true" {
                            vec![DISABLED_OPTION.to_string()]
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
                            options: {
                                let off = values.str("disabled_option") == "true";
                                OptionList::from_options()
                                    .disabling(move |step| off && *step == Step::Review)
                            },
                            option_label: (values.str("rich_label") == "true").then(|| Callback::new(rich)),
                            open: open(),
                            onchange: move |next| open.set(next),
                            panel,
                        }
                    }
                },
            }
            DocSection { title: "Accessibility",
                List {
                    ListItem {
                        "Every trigger is a tab stop. "
                        Kbd { "Enter" } " and " Kbd { "Space" } " toggle. "
                        Kbd { "↑" } " and " Kbd { "↓" }
                        " move between triggers, and "
                        Kbd { "Home" } " and " Kbd { "End" }
                        " jump to the ends, without toggling."
                    }
                    ListItem {
                        "Pick the heading level the page outline needs, then the size. "
                        Code { source: "h3" }
                        " assumes a section title above the accordion."
                    }
                    ListItem {
                        "Every open panel is a region named by its trigger. A "
                        Code { source: "Many" }
                        " accordion with a dozen open sections makes a long landmark list."
                    }
                    ListItem {
                        "With "
                        Code { source: "OptionLabel::rich" }
                        ", the name replaces the drawn label, so it must contain the visible "
                        "text (WCAG 2.5.3)."
                    }
                }
            }
        }
    }
}
