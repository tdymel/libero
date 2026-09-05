use crate::common::{attributes_of, body, classes_of, has_rule_for, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Options, SegmentedControl},
};

#[derive(Clone, PartialEq, Options)]
enum Emphasis {
    Bold,
    Italic,
}

#[test]
fn a_segmented_control_checks_only_the_selected_radio() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                SegmentedControl {
                    value: Emphasis::Bold,
                    onchange: move |_| {},
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    // The first `div` is the field's wrapper; the group is the one inside it.
    let body = &body[body[1..].find("<div").expect("the group") + 1..];
    let root = attributes_of(body, "div");

    // A radio group, not a toolbar: exactly one segment is ever selected, so
    // the semantics are the browser's rather than `aria-pressed`.
    assert_eq!(root["role"], "radiogroup");
    assert_eq!(root["data-state"], "horizontal filled collapsed");

    let checked: Vec<bool> = body
        .match_indices("<input")
        .map(|(at, _)| {
            let tag = &body[at..at + body[at..].find('>').expect("an unterminated tag")];
            tag.contains("checked")
        })
        .collect();
    assert_eq!(checked, [true, false]);

    // A rich label is an icon beside text, so the segment separates its own
    // children - no caller `sx` should be needed for that.
    assert!(html.contains(&format!(
        ".{} > label{{",
        classes_of(body, "div").first().expect("a framework class")
    )));
    assert!(html.contains("gap:var(--lsx-spacing-xs)"));

    // The derive names the segments, and the radio carries that name because
    // a rich label's text is not it.
    assert!(body.contains("aria-label=\"Bold\""));
    assert!(body.contains("aria-label=\"Italic\""));

    // The selected look is a `data-state`, so one class serves both segments -
    // `classes_of` only ever reads the first tag, hence the split.
    let first_at = body.find("<label").expect("a label");
    let (first, second) =
        body.split_at(first_at + body[first_at + 1..].find("<label").expect("two labels") + 1);
    assert_eq!(classes_of(first, "label"), classes_of(second, "label"));
    assert!(attributes_of(first, "label")["data-state"].contains("checked"));
    assert!(!attributes_of(second, "label")["data-state"].contains("checked"));

    let root_class = classes_of(body, "div");
    let root_class = root_class.first().expect("a framework class");
    assert!(has_rule_for(&html, root_class));
    // The selected block ties the variant's own `:hover` on specificity, so it
    // has to be emitted after it - a swap would silently lose the selected
    // background under the pointer.
    let selected =
        format!(".{root_class}[data-state~=\"outlined\"] > label[data-state~=\"checked\"]");
    let hover = format!(".{root_class}[data-state~=\"outlined\"] > label:hover");
    assert!(
        html.find(&selected).expect("no selected rule") > html.find(&hover).expect("no hover rule")
    );

    // The inner corners have to beat the segment's own radius rule, which is
    // one `when` shallower - so the attribute selectors are load-bearing.
    assert!(html.contains(&format!(
        ".{root_class}[data-state~=\"collapsed\"][data-state~=\"horizontal\"] > label:not(:first-of-type)"
    )));
}

#[test]
fn a_gapped_segmented_control_keeps_every_segment_s_own_corners() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                SegmentedControl {
                    gap: "xs",
                    value: Emphasis::Bold,
                    onchange: move |_| {},
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let body = &body[body[1..].find("<div").expect("the group") + 1..];
    let root = attributes_of(body, "div");

    // No `collapsed`, so the corner-squashing rules below cannot match.
    assert_eq!(root["data-state"], "horizontal filled size-xs");

    let root_class = classes_of(body, "div");
    let root_class = root_class.first().expect("a framework class");
    assert!(html.contains(&format!(
        ".{root_class}[data-state~=\"size-xs\"]{{gap:var(--lsx-spacing-xs)"
    )));
}

/// A `full_width` row must fit the box it claims to fill: its segments may
/// shrink below their labels, and a plain label sits in a span so it can end
/// in an ellipsis. Before, a narrow column pushed the last segment out of the
/// box. Only a browser sees the layout, so this reads the markup and rules.
#[test]
fn a_full_width_segment_can_shrink_below_its_label() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                SegmentedControl {
                    full_width: true,
                    value: Emphasis::Bold,
                    onchange: move |_| {},
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let body = &body[body[1..].find("<div").expect("the group") + 1..];
    assert!(body.contains("><span>Bold</span></label>"), "{body}");

    let root_class = classes_of(body, "div");
    let root_class = root_class.first().expect("a framework class");
    let row = format!(".{root_class}[data-state~=\"full-width\"][data-state~=\"horizontal\"]");
    assert!(
        html.contains(&format!(
            "{row} > label{{flex:1 1 0;min-width:0;overflow:hidden;}}"
        )),
        "{html}"
    );
    assert!(html.contains(&format!(
        "{row} > label > span{{min-width:0;overflow:hidden;text-overflow:ellipsis;}}"
    )));
}
