//! `Timeline`'s rendered contract: the list semantics, the progress split
//! between bullets and connectors, and the rail geometry `Stepper` inherits.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{OptionLabel, Timeline, TimelineEvent, TimelineLine},
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

/// Todo 2429: a rich title is drawn hidden from readers and named in plain text.
#[test]
fn a_rich_title_is_drawn_hidden_and_named_in_text() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Timeline {
                    items: vec![
                        TimelineEvent::new(OptionLabel::rich("Shipped", rsx! { b { "icon" } })),
                        TimelineEvent::new("Plain"),
                    ],
                }
            }
        }
    }

    let html = body(&render(app));

    assert!(
        html.contains("<span aria-hidden=\"true\"><b>icon</b></span>"),
        "{html}"
    );
    assert!(html.contains("Shipped"), "{html}");
    assert_eq!(
        html.matches("aria-hidden=\"true\"><b>").count(),
        1,
        "{html}"
    );
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

/// An item's `style` is a raw attribute the revert in `Box` never sees, so an
/// uncoloured item still declares its colour, as `revert-layer` (absent). Left
/// out, a colour the item had on an earlier render would stay.
#[test]
fn an_uncoloured_item_still_declares_its_colour() {
    let html = body(&render(plain_app));

    assert_eq!(
        html.matches("--lsx-timeline-color:revert-layer;").count(),
        4,
        "{html}"
    );
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

/// `Alternate` alternates at every width. It used to collapse to the
/// one-sided layout below 600px of the list's own width, behind a container
/// query - and a caller who asks for an alternating timeline in a narrow box
/// gets to have asked for it. The threshold also made the mode
/// undemonstrable: the docs preview caps the list at 498px, so there was no
/// window size at which anyone could see `Alternate` work.
///
/// Asserted by absence, because that is the only way a removed fallback can
/// be pinned: nothing queries a container, so nothing declares one either.
#[test]
fn the_alternate_layout_has_no_width_below_which_it_collapses() {
    let html = render(alternate_app);

    assert!(!html.contains("@container"), "{html}");
    assert!(!html.contains("container-type"), "{html}");
    assert!(!html.contains("container-name"), "{html}");
    // The centred rules are the only ones the align token carries now - the
    // one-sided fallback they used to override is gone, not merely outranked.
    assert!(
        !html.contains(&format!(
            r#"[data-state~="align-alternate"]{{padding-left:{}"#,
            "calc(var(--lsx-timeline-bullet) + var(--lsx-spacing-md))"
        )),
        "{html}"
    );
}

/// `Alternate` centres the rail *in the list*, so the list needs a width to
/// centre it in. Left to size itself it takes its content's width - 192px in
/// the docs preview, measured in Chromium on 2026-09-17 - and centring a rail
/// in that is not what anyone means by "alternate". `Left` and `Right` keep
/// shrink-to-fitting, which is why this is asserted per align token and not
/// on the bare class.
///
/// It also repaired the original defect, by a different route than it reads:
/// the arm used to carry `container-type: inline-size`, which is inline-axis
/// size containment, so the list could not take its width from its contents
/// and resolved to **zero** as a shrink-to-fit flex item. The container is
/// gone now, but the width it needed is still the right answer.
#[test]
fn the_alternate_arm_gives_the_list_a_width() {
    let html = render(alternate_app);

    assert!(html.contains("width:100%"), "{html}");
    // Under the align token, not on the bare class - a `width` that drifted
    // onto the class would widen every timeline, `Left` and `Right` included.
    let block = html
        .split('}')
        .find(|block| block.contains("width:100%"))
        .unwrap_or_default();
    assert!(
        block.contains(r#"[data-state~="align-alternate"]"#),
        "{block}"
    );
}
