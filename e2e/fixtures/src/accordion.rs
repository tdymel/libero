//! `Accordion`, for its keys, its expanded state and its panels' labels.

use dioxus::prelude::*;
use libero::components::{
    Accordion, AccordionOpen, Button, Flex, OptionLabel, OptionList, Options, Text,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/accordion", || rsx! { AccordionPage {} }),
    ("/accordion-long", || rsx! { LongLabelPage {} }),
    ("/accordion-wide", || rsx! { WidePanelPage {} }),
    ("/accordion-lone", || rsx! { LoneTriggerPage {} }),
    ("/accordion-ends", || rsx! { EndsPage {} }),
    ("/accordion-pinned", || rsx! { PinnedPage {} }),
    ("/accordion-chevron", || rsx! { ChevronPage {} }),
    ("/accordion-growing", || rsx! { GrowingPage {} }),
];

/// One section always open (todo 4140).
#[component]
fn PinnedPage() -> Element {
    let mut open = use_signal(|| AccordionOpen::One(Some(Step::Shipping)));
    rsx! {
        Accordion {
            id: "pinned",
            open: open(),
            onchange: move |next| open.set(next),
            collapsible: false,
            panel: |_: Step| rsx! { Text { "Body." } },
        }
    }
}

/// The chevron before the label, and not turning when a section opens (todos 4141, 4142).
#[component]
fn ChevronPage() -> Element {
    let mut open = use_signal(|| AccordionOpen::One(Some(Step::Shipping)));
    rsx! {
        Accordion {
            id: "chevron",
            open: open(),
            onchange: move |next| open.set(next),
            chevron_position: "start",
            rotate_chevron: false,
            panel: |_: Step| rsx! { Text { "Body." } },
        }
    }
}

/// An open panel led by a button, then a word with no break opportunity and a child
/// wider than a phone: the ring and the content stay reachable (todos 2430, 2431).
#[component]
fn WidePanelPage() -> Element {
    let mut open = use_signal(|| AccordionOpen::One(Some(Step::Shipping)));
    rsx! {
        Accordion {
            id: "wide",
            open: open(),
            onchange: move |next| open.set(next),
            panel: |_: Step| rsx! {
                Flex { direction: "column", align: "flex-start", gap: "sm",
                    Button { id: "first", "Edit address" }
                    Text { id: "word", "Versandkostenberechnungsgrundlagenverordnungsentwurfsfassung" }
                    div { id: "wide-child", style: "width: 600px; height: 20px" }
                }
            },
        }
    }
}

/// One enabled trigger on a page taller than the viewport: the arrows have nowhere to
/// move focus, so they scroll the page (todo 2432).
#[component]
fn LoneTriggerPage() -> Element {
    let mut open = use_signal(|| AccordionOpen::One(None::<Step>));
    rsx! {
        div { style: "min-height: 300vh",
            Accordion {
                id: "lone",
                open: open(),
                onchange: move |next| open.set(next),
                options: OptionList::from_options().disabling(|s| *s != Step::Shipping),
                panel: |_: Step| rsx! { Text { "Body." } },
            }
        }
    }
}

/// Three enabled triggers on a tall page: Home on the first and End on the last have
/// nowhere to move focus, so they scroll the page (todo 2889).
#[component]
fn EndsPage() -> Element {
    let mut open = use_signal(|| AccordionOpen::One(None::<Step>));
    rsx! {
        div { style: "min-height: 300vh",
            Accordion {
                id: "ends",
                open: open(),
                onchange: move |next| open.set(next),
                panel: |_: Step| rsx! { Text { "Body." } },
            }
        }
    }
}

/// One label with no break opportunity, for reflow at a narrow width.
#[component]
fn LongLabelPage() -> Element {
    let mut open = use_signal(|| AccordionOpen::One(None::<Step>));
    rsx! {
        Accordion {
            id: "long",
            open: open(),
            onchange: move |next| open.set(next),
            option_label: |step: Step| match step {
                Step::Shipping => OptionLabel::from("Versandkostenberechnungsgrundlagenverordnung"),
                other => OptionLabel::from(other.label()),
            },
            panel: |_: Step| rsx! { Text { "Body." } },
        }
    }
}

/// A section inserted ahead of the open one keeps its panel and the focus in it (todo 4400).
#[component]
fn GrowingPage() -> Element {
    let mut open = use_signal(|| AccordionOpen::One(Some("Beta".to_string())));
    let mut options = use_signal(|| vec!["Beta".to_string(), "Gamma".to_string()]);
    rsx! {
        div {
            onkeydown: move |event: KeyboardEvent| {
                let known = options.read().iter().any(|value| value == "Alpha");
                if event.key() == Key::Named(NamedKey::F2) && !known {
                    options.write().insert(0, "Alpha".to_string());
                }
            },
            Accordion {
                id: "growing",
                open: open(),
                onchange: move |next| open.set(next),
                options: options(),
                panel: |value: String| rsx! { input { id: "field-{value}" } },
            }
        }
    }
}

#[derive(Clone, PartialEq, Options)]
enum Step {
    Shipping,
    Payment,
    Review,
}

/// Three sections, the middle disabled for the arrows to skip. Shipping's "Continue" opens
/// Review, closing Shipping around the focused button: the focus-return case.
#[component]
fn AccordionPage() -> Element {
    let mut open = use_signal(|| AccordionOpen::One(None::<Step>));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            Accordion {
                id: "checkout",
                open: open(),
                onchange: move |next| open.set(next),
                options: OptionList::from_options().disabling(|s| *s == Step::Payment),
                panel: move |step: Step| match step {
                    Step::Shipping => rsx! {
                        Flex { direction: "column", gap: "sm",
                            Text { "Shipping is calculated at checkout." }
                            Button {
                                id: "continue",
                                onclick: move |_| open.set(AccordionOpen::One(Some(Step::Review))),
                                "Continue"
                            }
                        }
                    },
                    Step::Payment => rsx! { Text { "Card details." } },
                    Step::Review => rsx! { Text { "Check your order." } },
                },
            }
        }
    }
}
