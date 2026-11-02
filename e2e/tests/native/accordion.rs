//! `Accordion`'s plain heading in Blitz's layout (todo 5368); keys and focus are shared scenarios.

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::{Accordion, AccordionOpen, Options, Text};

#[derive(Clone, Copy, PartialEq, Options)]
enum Step {
    Shipping,
    Payment,
    Review,
}

fn plain() -> Element {
    rsx! {
        Accordion {
            id: "plain",
            open: AccordionOpen::One(None::<Step>),
            has_panel: |step: Step| step != Step::Payment,
            panel: |_: Step| rsx! { Text { "Body." } },
        }
    }
}

/// Todo 5392 (5368): a section without a panel is text in the trigger's box, not a button.
#[test]
fn a_section_without_a_panel_is_a_plain_heading() {
    let page = mount(plain);
    assert_eq!(page.query_all("#plain button").len(), 2, "{}", page.tree());
    assert_eq!(page.text("#plain h3 > [data-accordion-plain]"), "Payment");
    assert!(
        page.attr("#plain h3 > [data-accordion-plain]", "aria-expanded")
            .is_none()
    );
    assert_eq!(page.query_all("#plain [aria-expanded]").len(), 2);

    let (_, trigger_y, _, trigger_h) = page.rect("#plain-trigger-0");
    let (_, plain_y, plain_w, plain_h) = page.rect("#plain h3 > [data-accordion-plain]");
    let (_, _, root_w, _) = page.rect("#plain");
    assert!(
        (plain_h - trigger_h).abs() < 0.5,
        "plain {plain_h}px, trigger {trigger_h}px"
    );
    assert!(
        plain_y > trigger_y,
        "the heading sits below the first trigger"
    );
    assert!(
        (plain_w - root_w).abs() < 1.0,
        "plain {plain_w}px of {root_w}px"
    );
    assert_eq!(
        page.computed("#plain h3 > [data-accordion-plain]", "color"),
        page.computed("#plain-trigger-0", "color")
    );
    assert_eq!(
        page.computed("#plain h3 > [data-accordion-plain]", "padding-left"),
        page.computed("#plain-trigger-0", "padding-left")
    );
}
