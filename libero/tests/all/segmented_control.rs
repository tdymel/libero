use crate::common::{attributes_of, body, classes_of, has_rule_for, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{OptionItem, OptionList, Options, SegmentedControl},
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
    // Todo 20: each radio posts `Options::value`, not its position.
    let values: Vec<String> = body
        .match_indices("<input")
        .map(|(at, _)| attributes_of(&body[at..], "input")["value"].clone())
        .collect();
    assert_eq!(values, ["Bold", "Italic"]);

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

static TONAL: libero::theme::Theme = libero::theme::Theme {
    segmented_control: libero::theme::SegmentedControlDefaults {
        variant: libero::theme::Variant::Tonal,
    },
    ..libero::theme::Theme::DEFAULT
};

/// An unset `variant` takes `theme.segmented_control.variant`, its own field
/// and not `Button`'s.
#[test]
fn an_unset_variant_follows_its_own_theme_field() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { themes: &TONAL,
                SegmentedControl { value: Emphasis::Bold, onchange: move |_| {} }
            }
        }
    }

    let html = body(&render(app));
    // The first `div` is the field's wrapper; the group is the one inside it.
    let group = &html[html[1..].find("<div").expect("the group") + 1..];
    assert_eq!(
        attributes_of(group, "div")["data-state"],
        "horizontal tonal collapsed"
    );
}

/// Todo 371: the per-segment flag rides on the option, and `options` left
/// unset is not an empty list - it still means every `Options::options()`.
#[test]
fn a_segmented_control_disables_the_segment_its_option_flagged() {
    fn unset() -> Element {
        rsx! {
            LiberoProvider {
                SegmentedControl {
                    "aria-label": "Emphasis",
                    value: Emphasis::Bold,
                    onchange: move |_| {},
                }
            }
        }
    }

    fn flagged() -> Element {
        rsx! {
            LiberoProvider {
                SegmentedControl {
                    "aria-label": "Emphasis",
                    value: Emphasis::Bold,
                    onchange: move |_| {},
                    options: OptionList::new([
                        Emphasis::Bold.into(),
                        OptionItem::new(Emphasis::Italic).disabled(true),
                    ]),
                }
            }
        }
    }

    // Unset falls back to the enum's own options, and none of them is off.
    let unset = body(&render(unset));
    assert_eq!(unset.matches("<input").count(), 2);
    assert_eq!(unset.matches("disabled").count(), 0);

    // The flagged segment is the only one the control refuses, and the flag
    // is matched by position in the list rather than by value.
    let flagged = body(&render(flagged));
    assert_eq!(flagged.matches("<input").count(), 2);
    assert_eq!(flagged.matches("disabled").count(), 2); // the input, and `data-state`
    let italic = &flagged[flagged.find("Italic").expect("the second segment")..];
    assert!(italic.contains("disabled"));
}
