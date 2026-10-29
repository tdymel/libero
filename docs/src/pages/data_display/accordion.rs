use crate::components::{Control, Demo, DemoFile, DemoValues, DocPage, Wrap, a11y, prop, props};
use libero::components::Pictogram;
use pictogram_icons_lucide as lucide;

use dioxus::prelude::*;
use libero::{
    components::{
        Accordion, AccordionOpen, Box, Button, Code, Flex, Icon, OptionLabel, OptionList, Options,
        Text,
    },
    sx::sx,
    use_theme,
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

fn review_off(values: &DemoValues) -> bool {
    values.str("disabled_option") == "true"
}

/// One mode's panels. With `review_off` the step into Review is disabled: a host-opened
/// disabled section cannot be closed from its trigger.
fn one_steps(
    mut open: Signal<AccordionOpen<Step>>,
    review_off: bool,
) -> impl FnMut(Step) -> Element {
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
                Button { disabled: review_off, onclick: move |_| open.set(Some(Step::Review).into()), "Continue" }
            }
        },
        Step::Review => rsx! { Text { "Check the order, then place it." } },
    }
    // demo-code: one end
}

#[component]
pub fn AccordionPage() -> Element {
    // One signal per mode, so switching the mode control does not have to
    // convert one open set into the other.
    let one = use_signal(|| AccordionOpen::One(Some(Step::Shipping)));
    let several = use_signal(|| AccordionOpen::Many(vec![Step::Shipping]));
    let theme = use_theme();

    // Built once: a new callback per render would redraw every section.
    let many_panel = use_callback(
        // demo-code: many start
        |step: Step| match step {
            Step::Shipping => rsx! { Text { "Where should the parcel go?" } },
            Step::Payment => rsx! { Text { "Card, invoice or bank transfer." } },
            Step::Review => rsx! { Text { "Check the order, then place it." } },
        },
        // demo-code: many end
    );
    let one_panel = use_callback(one_steps(one, false));
    let one_panel_review_off = use_callback(one_steps(one, true));
    let rich_label = use_callback(rich);

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
                        .doc("The sections to show. A `Vec<T>` converts. An `OptionList<T>` can disable a section, which renders, cannot be toggled and stays a tab stop. A disabled section the host opens through `open` cannot be closed from its trigger, so open only enabled ones. Groups are drawn flat."),
                    prop("option_label", "Callback<T, OptionLabel>")
                        .default("T::label()")
                        .doc("Renames a section. `OptionLabel::rich` draws the trigger as rsx and still names it."),
                    prop("heading", "HtmlTag").default("h3").doc("The heading around each trigger, `h1` to `h6`."),
                    prop("size", "Size")
                        .default(theme.accordion.size.as_str())
                        .doc("Type and padding of the triggers and panels. Panel content takes the type size only where it sets none of its own; `Text` sets its own."),
                    prop("collapsible", "bool")
                        .default("true")
                        .doc("`false` keeps the open section open in `One` mode: its trigger turns `aria-disabled` and only opening another closes it. `Many` mode ignores it."),
                    prop("chevron_position", "AccordionChevronPosition")
                        .default("end")
                        .doc("`end` or `start`: the chevron after or before the label. Logical, so right-to-left flips it."),
                    prop("rotate_chevron", "bool")
                        .default("true")
                        .doc("`false` keeps the chevron pointing down when a section opens."),
                ]),
            ],
            accessibility: a11y()
                .key(["Tab"], "Moves between the triggers: each one is a tab stop.")
                .key(["Enter", "Space"], "Toggles the focused section.")
                .key(["Up", "Down"], "Moves to the previous or next enabled trigger, skipping disabled ones and wrapping at the ends, without toggling.")
                .key(["Home", "End"], "Jumps to the first or last enabled trigger, without toggling. On the first or last one it stays the page's.")
                .handles([
                    "Every open panel is a region named by its trigger.",
                    "Panel content wider than the panel wraps, or scrolls inside the panel, instead of being cut off.",
                    "With `collapsible: false` the open trigger is `aria-disabled` and stays a tab stop, and no cursor or hover invites a press.",
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
                            let disabled = match review_off(values) {
                                true => "disabled: true, ",
                                false => "",
                            };
                            let panel = FILE.section(mode).replace("disabled: review_off, ", disabled);
                            vec![format!("panel: {}", panel.trim_end_matches(','))]
                        }),
                    Control::sizes("size")
                        .default(theme.accordion.size.as_str()),
                    Control::toggle("heading", ["h2", "h3", "h4"])
                        .labels(["H2", "H3", "H4"])
                        .default("h3"),
                    Control::switch("collapsible")
                        .default("true")
                        .hidden_when(many),
                    Control::toggle("chevron_position", ["end", "start"])
                        .labels(["End", "Start"]),
                    Control::switch("rotate_chevron").default("true"),
                    // The flag lives inside `options`, so the switch stands
                    // for one named section rather than for a prop of its own.
                    Control::switch("disabled_option").code(|_, values| {
                        if review_off(values) {
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
                    let off = review_off(&values);
                    let panel = match (is_many, off) {
                        (true, _) => many_panel,
                        (false, true) => one_panel_review_off,
                        (false, false) => one_panel,
                    };
                    let options = if off {
                        // demo-code: disabled start
                        OptionList::from_options().disabling(|step| *step == Step::Review)
                        // demo-code: disabled end
                    } else {
                        OptionList::from_options()
                    };
                    // Keyed by mode and the disabled option, so a switch remounts it: a section
                    // redraws only when its value or open state changes, not with `panel`.
                    rsx! {
                        for (mode, off) in [(is_many, off)] {
                            Accordion {
                                key: "{mode}-{off}",
                                size: values.str("size"),
                                heading: values.str("heading"),
                                collapsible: values.str("collapsible") == "true",
                                chevron_position: values.str("chevron_position"),
                                rotate_chevron: values.str("rotate_chevron") == "true",
                                options: options.clone(),
                                option_label: (values.str("rich_label") == "true").then_some(rich_label),
                                open: open(),
                                onchange: move |next| open.set(next),
                                panel,
                            }
                        }
                    }
                },
            }
        }
    }
}
