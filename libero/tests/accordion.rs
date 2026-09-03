//! `Accordion`'s rendered contract: the APG wiring between each trigger and
//! its region, which panels are in the DOM, the heading level, and the
//! open-set arithmetic behind `onchange`.

mod common;

use common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Accordion, AccordionOpen, OptionLabel, Options},
};

#[derive(Clone, Debug, PartialEq, Options)]
enum Step {
    Shipping,
    #[option(label = "Payment method")]
    Payment,
    Review,
}

fn panel(step: Step) -> Element {
    match step {
        Step::Shipping => rsx! { "shipping body" },
        Step::Payment => rsx! { "payment body" },
        Step::Review => rsx! { "review body" },
    }
}

fn one_app() -> Element {
    rsx! {
        LiberoProvider {
            Accordion::<Step> {
                id: "checkout",
                open: AccordionOpen::One(Some(Step::Shipping)),
                onchange: |_| {},
                panel: panel,
            }
        }
    }
}

fn many_app() -> Element {
    rsx! {
        LiberoProvider {
            Accordion::<Step> {
                open: AccordionOpen::Many(vec![Step::Shipping, Step::Review]),
                onchange: |_| {},
                panel: panel,
            }
        }
    }
}

/// Every `<button …>` open tag, in order.
fn buttons(html: &str) -> Vec<std::collections::BTreeMap<String, String>> {
    let body = body(html);
    body.match_indices("<button")
        .map(|(at, _)| attributes_of(&body[at..], "button"))
        .collect()
}

#[test]
fn each_trigger_points_at_its_region_and_the_region_back() {
    let html = render(one_app);
    let body = body(&html);
    let triggers = buttons(&html);

    assert_eq!(triggers.len(), 3, "{html}");
    for (index, trigger) in triggers.iter().enumerate() {
        assert_eq!(trigger["id"], format!("checkout-trigger-{index}"));
        assert_eq!(trigger["aria-controls"], format!("checkout-region-{index}"));
        assert_eq!(trigger["type"], "button");
        // Every header is a tab stop: no roving tabindex, unlike `Tabs`.
        assert!(!trigger.contains_key("tabindex"), "{html}");

        let region_at = body
            .find(&format!("id=\"checkout-region-{index}\""))
            .unwrap_or_else(|| panic!("region {index} missing:\n{html}"));
        let region_tag = &body[body[..region_at].rfind('<').unwrap()..];
        let region = attributes_of(region_tag, "div");
        assert_eq!(region["role"], "region");
        assert_eq!(
            region["aria-labelledby"],
            format!("checkout-trigger-{index}")
        );
    }
}

#[test]
fn aria_expanded_follows_the_open_set_as_strings() {
    let triggers = buttons(&render(one_app));

    assert_eq!(triggers[0]["aria-expanded"], "true");
    assert_eq!(triggers[1]["aria-expanded"], "false");
    assert_eq!(triggers[2]["aria-expanded"], "false");
}

/// `keep_mounted: false`: a closed panel's content is not in the DOM at all,
/// while its region still is, so `aria-controls` always resolves.
#[test]
fn only_open_panels_are_mounted() {
    let one = body(&render(one_app));
    assert!(one.contains("shipping body"), "{one}");
    assert!(!one.contains("payment body"), "{one}");
    assert!(!one.contains("review body"), "{one}");

    let many = body(&render(many_app));
    assert!(many.contains("shipping body"), "{many}");
    assert!(!many.contains("payment body"), "{many}");
    assert!(many.contains("review body"), "{many}");
}

#[test]
fn the_derived_label_names_the_trigger() {
    let html = render(one_app);

    assert_eq!(buttons(&html)[1]["aria-label"], "Payment method");
    assert!(body(&html).contains("Payment method"), "{html}");
}

fn rich(step: Step) -> OptionLabel {
    OptionLabel::rich(format!("Step {}", step.label()), rsx! { em { "drawn" } })
}

fn rich_app() -> Element {
    rsx! {
        LiberoProvider {
            Accordion::<Step> {
                onchange: |_| {},
                panel: panel,
                label: rich,
            }
        }
    }
}

#[test]
fn a_rich_label_draws_the_rsx_and_names_the_trigger_with_the_name() {
    let html = render(rich_app);

    assert_eq!(buttons(&html)[0]["aria-label"], "Step Shipping");
    assert!(body(&html).contains("<em>drawn</em>"), "{html}");
}

fn disabled_app() -> Element {
    rsx! {
        LiberoProvider {
            Accordion::<Step> {
                onchange: |_| {},
                panel: panel,
                disabled: vec![Step::Payment],
            }
        }
    }
}

/// `aria-disabled`, never `disabled`: the section stays a tab stop.
#[test]
fn a_disabled_section_stays_reachable() {
    let triggers = buttons(&render(disabled_app));

    assert_eq!(triggers[1]["aria-disabled"], "true");
    assert!(!triggers[1].contains_key("disabled"));
    assert_eq!(triggers[0]["aria-disabled"], "false");
}

fn heading_h2_app() -> Element {
    rsx! {
        LiberoProvider {
            Accordion::<Step> { onchange: |_| {}, panel: panel, heading: "h2" }
        }
    }
}

fn heading_invalid_app() -> Element {
    rsx! {
        LiberoProvider {
            Accordion::<Step> { onchange: |_| {}, panel: panel, heading: "div" }
        }
    }
}

#[test]
fn the_heading_defaults_to_h3_and_takes_another_level() {
    let default = body(&render(one_app));
    assert_eq!(default.matches("<h3").count(), 3, "{default}");

    let h2 = body(&render(heading_h2_app));
    assert_eq!(h2.matches("<h2").count(), 3, "{h2}");
    assert_eq!(h2.matches("<h3").count(), 0, "{h2}");
}

/// Anything but `h1`..`h6` would drop the triggers out of the outline.
#[test]
fn a_non_heading_tag_falls_back_to_h3() {
    let html = body(&render(heading_invalid_app));

    assert_eq!(html.matches("<h3").count(), 3, "{html}");
}

/// The chevron's turn is declared in a selector block, so its guard has to be
/// nested inside that same block to win - a flat one would lose on specificity.
#[test]
fn the_chevron_turn_has_a_reduced_motion_guard_beside_it() {
    let html = render(one_app);
    let class = attributes_of(&body(&html), "div")["class"]
        .split_whitespace()
        .next()
        .unwrap()
        .to_string();

    assert!(
        html.contains(&format!(
            "@media (prefers-reduced-motion: reduce){{.{class} > [data-accordion-item] > [data-accordion-heading] [data-accordion-chevron]{{transition:none;}}}}"
        )),
        "{html}"
    );
}

#[test]
fn one_mode_holds_at_most_one_section() {
    let open = AccordionOpen::One(Some(Step::Shipping));

    assert_eq!(
        open.toggled(Step::Payment),
        AccordionOpen::One(Some(Step::Payment))
    );
    assert_eq!(open.toggled(Step::Shipping), AccordionOpen::One(None));
    assert_eq!(
        AccordionOpen::<Step>::One(None).toggled(Step::Review),
        AccordionOpen::One(Some(Step::Review))
    );
}

#[test]
fn many_mode_toggles_each_section_on_its_own() {
    let open = AccordionOpen::Many(vec![Step::Shipping]);

    assert_eq!(
        open.toggled(Step::Review),
        AccordionOpen::Many(vec![Step::Shipping, Step::Review])
    );
    assert_eq!(open.toggled(Step::Shipping), AccordionOpen::Many(vec![]));
}

#[test]
fn the_accessors_read_either_mode() {
    let one = AccordionOpen::from(Some(Step::Payment));
    assert_eq!(one.one(), Some(&Step::Payment));
    assert_eq!(one.values(), &[Step::Payment]);

    let many = AccordionOpen::from(vec![Step::Shipping, Step::Review]);
    assert_eq!(many.one(), None);
    assert_eq!(many.values(), &[Step::Shipping, Step::Review]);
}
