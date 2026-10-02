//! `Accordion`'s rendered contract: the APG wiring between each trigger and
//! its region, which panels are in the DOM, the heading level, and the
//! open-set arithmetic behind `onchange`.

use std::time::Duration;

use crate::common::{attributes_of, body, drive_until, render};

use dioxus::{core::ScopeId, prelude::*};
use libero::{
    LiberoProvider,
    components::{Accordion, AccordionOpen, OptionLabel, OptionList, Options},
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
        // A closed panel leaves the landmarks through `Collapse`'s visibility.
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
                option_label: rich,
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
                options: OptionList::from_options().disabling(|step| *step == Step::Payment),
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

/// Focus moves find a trigger by `{id}-trigger-{n}`, and a caller's id need
/// not be a CSS identifier. The lookup selects by attribute (todo 248), which
/// only works while the triggers carry exactly that id.
#[test]
fn a_caller_id_that_is_no_css_identifier_still_names_the_triggers() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Accordion::<Step> {
                    id: "1-faq",
                    open: AccordionOpen::One(Some(Step::Shipping)),
                    onchange: |_| {},
                    panel: panel,
                }
            }
        }
    }

    let body = body(&render(app));

    for n in 0..3 {
        assert!(
            body.contains(&format!("id=\"1-faq-trigger-{n}\"")),
            "{body}"
        );
    }
    assert!(body.contains("id=\"1-faq-region-0\""), "{body}");
}

/// Todo 371: an `options` prop that was never set is not an empty list - it
/// still means every `Options::options()`, with nothing disabled.
#[test]
fn an_accordion_left_without_options_still_lists_the_enums_own_sections() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Accordion::<Step> { onchange: |_| {}, panel: panel }
            }
        }
    }

    let triggers = buttons(&render(app));

    assert_eq!(triggers.len(), 3);
    assert!(triggers.iter().all(|t| t["aria-disabled"] == "false"));
}

thread_local! {
    static PANELS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[derive(Clone, Copy)]
struct Held {
    open: Signal<AccordionOpen<Step>>,
    note: Signal<&'static str>,
    parent: Signal<u32>,
}

/// The panel bodies read `note`, a signal, and count their builds.
fn held_app() -> Element {
    let held = use_context_provider(|| Held {
        open: Signal::new(AccordionOpen::One(Some(Step::Shipping))),
        note: Signal::new("first"),
        parent: Signal::new(0),
    });
    let note = held.note;
    rsx! {
        LiberoProvider {
            Accordion::<Step> {
                class: "render-{held.parent}",
                open: (held.open)(),
                onchange: |_| {},
                panel: move |step: Step| {
                    PANELS.with(|panels| panels.set(panels.get() + 1));
                    rsx! { "{step.label()} {note}" }
                },
            }
        }
    }
}

fn mount_held() -> (VirtualDom, Held) {
    let mut dom = VirtualDom::new(held_app);
    dom.rebuild_in_place();
    let held = dom.in_scope(ScopeId::APP, consume_context::<Held>);
    (dom, held)
}

/// The body once it shows `shown` after `change`, and the panels built meanwhile.
fn step(dom: &mut VirtualDom, change: impl FnOnce(), shown: &str) -> (String, usize) {
    PANELS.with(|panels| panels.set(0));
    dom.in_runtime(change);
    // An opening `Collapse` mounts its body from an effect.
    let html = drive_until(dom, Duration::from_secs(2), |html| html.contains(shown));
    (html, PANELS.with(std::cell::Cell::get))
}

/// A render-count budget (todo 2024): a switch builds the opening panel alone, and an
/// `Accordion` render that changes no section builds none.
#[test]
fn a_switch_builds_only_the_opening_panel() {
    let (mut dom, mut held) = mount_held();

    let (html, built) = step(&mut dom, || held.parent.set(1), "render-1");
    assert!(html.contains("render-1"), "{html}");
    assert_eq!(built, 0, "{html}");
    let open = || held.open.set(AccordionOpen::One(Some(Step::Payment)));
    let (html, built) = step(&mut dom, open, "Payment method first");
    assert!(html.contains("Payment method first"), "{html}");
    assert_eq!(built, 1, "{html}");
}

/// A section scope still follows a signal its open panel reads (todo 2024).
#[test]
fn a_signal_an_open_panel_reads_redraws_it() {
    let (mut dom, mut held) = mount_held();

    let (html, _) = step(&mut dom, || held.note.set("second"), "Shipping second");
    assert!(html.contains("Shipping second"), "{html}");
    assert!(!html.contains("first"), "{html}");
}

/// A panel closed while its signal changed opens with the current value.
#[test]
fn a_closed_panel_opens_with_the_signal_value_of_now() {
    let (mut dom, mut held) = mount_held();

    step(&mut dom, || held.note.set("second"), "Shipping second");
    let open = || {
        held.open
            .set(AccordionOpen::Many(vec![Step::Shipping, Step::Review]))
    };
    let (html, _) = step(&mut dom, open, "Review second");
    assert!(html.contains("Review second"), "{html}");
}
