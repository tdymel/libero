//! `Timeline`'s rendered contract: the list semantics, the progress split
//! between bullets and connectors, and the rail geometry `Stepper` inherits.

mod common;

use common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Timeline, TimelineEvent, TimelineLine},
};

fn events() -> Vec<TimelineEvent> {
    vec![
        TimelineEvent::new("Pushed"),
        TimelineEvent::new("Reviewed"),
        TimelineEvent::new("Deployed"),
        TimelineEvent::new("Rolled back").line(TimelineLine::Dashed),
    ]
}

fn plain_app() -> Element {
    rsx! {
        LiberoProvider { Timeline { items: events() } }
    }
}

fn active_app() -> Element {
    rsx! {
        LiberoProvider { Timeline { active: 2, items: events() } }
    }
}

fn overflowing_app() -> Element {
    rsx! {
        LiberoProvider { Timeline { active: 99, items: events() } }
    }
}

fn alternate_app() -> Element {
    rsx! {
        LiberoProvider { Timeline { align: "alternate", items: events() } }
    }
}

/// The rail draws position and count; the list is how a screen-reader user
/// gets the same two facts. `role` is explicit because Safari with VoiceOver
/// drops list semantics from a `list-style: none` list.
#[test]
fn it_renders_an_ordered_list_with_an_explicit_role() {
    let html = body(&render(plain_app));

    assert!(html.contains("<ol"), "{html}");
    assert_eq!(attributes_of(&html, "ol")["role"], "list");
    assert_eq!(html.matches("<li").count(), 4, "{html}");
}

/// A caller's own `role` has to win - `attr_default`, not `attr`.
#[test]
fn a_callers_role_beats_the_default() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Timeline { role: "presentation", items: events() }
            }
        }
    }

    assert_eq!(attributes_of(&render(app), "ol")["role"], "presentation");
}

/// The two independent active states: bullets `0..=active`, connectors
/// `0..active`. That split is what makes the rail read as progress rather
/// than as a highlight, so it is worth pinning per item.
#[test]
fn active_colours_the_bullets_through_and_the_connectors_between() {
    let html = body(&render(active_app));

    // Item 2 is current: active bullet, and no connector below it.
    assert!(html.contains(r#"aria-current="step""#), "{html}");
    assert_eq!(html.matches(r#"aria-current="step""#).count(), 1, "{html}");
    // Three active bullets (0, 1, 2) plus their three bullet spans.
    assert_eq!(html.matches("line-active").count(), 2, "{html}");
}

/// `active` names an event, so past the end means "all done", not "none".
#[test]
fn an_out_of_range_active_clamps_to_the_last_event() {
    let html = body(&render(overflowing_app));

    assert_eq!(html.matches(r#"aria-current="step""#).count(), 1, "{html}");
    // Every connector but the last event's is active.
    assert_eq!(html.matches("line-active").count(), 3, "{html}");
}

/// No `active` at all means no current event - and no `aria-current` anywhere.
#[test]
fn no_active_marks_no_current_event() {
    let html = body(&render(plain_app));

    assert!(!html.contains("aria-current"), "{html}");
    assert!(!html.contains("line-active"), "{html}");
}

/// The bullet is the rail's drawing; the title is the text.
#[test]
fn bullets_are_hidden_from_the_accessibility_tree() {
    let html = body(&render(plain_app));

    assert_eq!(html.matches(r#"aria-hidden="true""#).count(), 4, "{html}");
    assert!(html.contains("Pushed"), "{html}");
}

/// Per-item line style rides a variable, so one `::before` rule serves every
/// style instead of one rule per combination.
#[test]
fn a_per_item_line_style_is_a_variable_not_a_rule() {
    let html = body(&render(plain_app));

    assert!(html.contains("--lsx-timeline-line-style:solid"), "{html}");
    assert!(html.contains("--lsx-timeline-line-style:dashed"), "{html}");
}

/// The geometry `Stepper` inherits. Asserted on the emitted text rather than
/// on the formula, because the plan quotes a formula whose origin could not be
/// verified - see `components/common/rail.rs`.
#[test]
fn the_rail_geometry_is_derived_from_one_centreline() {
    let html = render(plain_app);

    // The connector is centred on the rail: half a marker, less half a line.
    assert!(
        html.contains(
            "left:calc(var(--lsx-timeline-bullet) / 2 - var(--lsx-timeline-line-width) / 2)"
        ),
        "{html}"
    );
    // And it reaches a whole gap past its own item, which is what makes one
    // continuous rail out of separate items.
    assert!(
        html.contains("bottom:calc(var(--lsx-timeline-space) * -1)"),
        "{html}"
    );
    // Never below the last event - a rail past the final marker points at
    // nothing.
    assert!(html.contains(":not(:last-of-type)::before"), "{html}");
    // The one-sided content inset clears the whole marker, so the gap between
    // bullet and text is the space term and nothing else.
    assert!(
        html.contains("padding-left:calc(var(--lsx-timeline-bullet) + var(--lsx-spacing-md))"),
        "{html}"
    );
}

/// The centred arm's inset is the one that was wrong: the marker straddles the
/// midline, so content has to start a half-marker past 50%. Inlining
/// `calc(50% + space)` left a clearance of `space - marker/2` - 2px at the
/// default bullet size, and negative at the two largest. Pinned here because
/// neither SSR nor a diff read can see it, and C3's centred arm inherits the
/// same expression.
#[test]
fn the_centred_arm_clears_the_whole_half_marker() {
    let html = render(alternate_app);
    let inset = "calc(50% + var(--lsx-timeline-bullet) / 2 + var(--lsx-spacing-md))";

    assert!(html.contains(&format!("padding-left:{inset}")), "{html}");
    // The mirrored side has to match, or even and odd events sit at different
    // distances from the same rail.
    assert!(html.contains(&format!("padding-right:{inset}")), "{html}");
    // And the half-marker term is exactly what a hand-rolled version drops.
    assert!(
        !html.contains("padding-left:calc(50% + var(--lsx-spacing-md))"),
        "{html}"
    );
    // The pairing, not just the inset: the marker starts a half-marker before
    // the midline, so its trailing edge is exactly where the inset begins
    // measuring its space from. Asserting one without the other would let the
    // two drift apart while each still looked right.
    assert!(
        html.contains("left:calc(50% - var(--lsx-timeline-bullet) / 2)"),
        "{html}"
    );
}

/// The specificity trap this library keeps meeting: a container query adds no
/// specificity, so the collapse has to carry the same `[data-state~=..]` the
/// rule it overrides does. Written beside the condition it would be 0-1-0
/// against 0-2-0, and a phone-width timeline would stay alternating.
#[test]
fn the_alternate_collapse_is_nested_inside_the_align_condition() {
    let html = render(alternate_app);

    assert!(
        html.contains("@container timeline (min-width: 600px)"),
        "{html}"
    );
    // The guarded rule keeps the align token, rather than sitting on the bare
    // class the way a hoisted query would.
    assert!(html.contains(r#"(min-width: 600px){.lsx-"#), "{html}");
    assert!(
        html.matches(r#"[data-state~="align-alternate"]"#).count() > 1,
        "{html}"
    );
}
