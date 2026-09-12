//! `Accordion`, for its keys, its expanded state and its panels' labels.

use dioxus::prelude::*;
use libero::components::{
    Accordion, AccordionOpen, Button, Flex, OptionLabel, OptionList, Options, Text,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/accordion", || rsx! { AccordionPage {} }),
    ("/accordion-long", || rsx! { LongLabelPage {} }),
];

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

#[derive(Clone, PartialEq, Options)]
enum Step {
    Shipping,
    Payment,
    Review,
}

/// Three sections, the middle one disabled so the arrows have one to skip.
/// Shipping's "Continue" opens Review, which closes Shipping around the
/// focused button: the focus-return case.
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
