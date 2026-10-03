use crate::common::{attributes_of, body, classes_of, has_rule_for, render, tags_with};

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
    // one `when` shallower - so the attribute selectors are load-bearing. One
    // logical rule for both directions: Blitz matches no `:dir()` (todo 781).
    assert!(html.contains(&format!(
        ".{root_class}[data-state~=\"collapsed\"][data-state~=\"horizontal\"] > label:not(:first-of-type){{"
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

/// Every segment may shrink below its label, which ends in an ellipsis with
/// the whole name as `title` (todo 481); `full_width` shares the row out.
/// Only a browser sees the layout, so this reads the markup and rules.
#[test]
fn a_segment_can_shrink_below_its_label() {
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
    assert!(body.contains(r#"title="Bold""#), "{body}");
    assert!(body.contains("><span>Bold</span></label>"), "{body}");

    let root_class = classes_of(body, "div");
    let root_class = root_class.first().expect("a framework class");
    let root = format!(".{root_class}");
    let row = format!("{root}[data-state~=\"full-width\"][data-state~=\"horizontal\"]");
    assert!(
        html.contains("max-width:100%;min-width:0;overflow:hidden;}"),
        "{html}"
    );
    assert!(
        html.contains(&format!(
            "{root} > label > span{{min-width:0;overflow:hidden;text-overflow:ellipsis;}}"
        )),
        "{html}"
    );
    assert!(
        html.contains(&format!("{row} > label{{flex:1 1 0;}}")),
        "{html}"
    );
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

/// Events dispatched the way a renderer does, through [`crate::dispatch`].
mod dispatched {

    use crate::dispatch::*;

    use dioxus::prelude::*;
    use libero::{
        LiberoProvider,
        components::{Form, OptionList, SegmentedControl},
    };

    /// The control is strictly controlled and the radio is cancelled on click, so
    /// what is checked after a click comes from Rust alone - never from the flip
    /// the browser did during activation.
    #[test]
    fn clicking_a_segment_moves_the_selection() {
        fn app() -> Element {
            let mut value = use_signal(|| Emphasis::Bold);

            rsx! {
                LiberoProvider {
                    SegmentedControl {
                        value: value(),
                        onchange: move |next| value.set(next),
                    }
                }
            }
        }

        let Page { mut dom, rec: find } = Page::build(app);
        let italic = find.element("click", "aria-label", "Italic");

        assert_eq!(checked_states(&dioxus_ssr::render(&dom)), [true, false]);

        dom.runtime()
            .handle_event("click", Event::new(click_event(), true), italic);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        assert_eq!(checked_states(&dioxus_ssr::render(&dom)), [false, true]);

        // Exactly one is always selected, so clicking it again is a no-op rather
        // than a deselect - that is the whole difference from a row of toggles.
        dom.runtime()
            .handle_event("click", Event::new(click_event(), true), italic);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        assert_eq!(checked_states(&dioxus_ssr::render(&dom)), [false, true]);
    }

    fn readonly_segments() -> Element {
        let mut value = use_signal(|| Emphasis::Bold);
        rsx! {
            LiberoProvider {
                SegmentedControl {
                    label: "Emphasis",
                    value: value(),
                    readonly: READ_ONLY.get(),
                    onchange: move |next| {
                        heard(next == Emphasis::Italic);
                        value.set(next);
                    },
                }
            }
        }
    }

    /// Todo 306: the arrows move *and* select, so a read-only strip refuses them
    /// outright, as `RadioGroup` does - and the click and Space with them.
    #[test]
    fn a_read_only_segmented_control_picks_nothing() {
        let (_, html) = send(readonly_segments, false, "keydown", first_keydown, || {
            key_event(Key::ArrowRight)
        });
        assert_eq!(
            checked_states(&html),
            [false, true],
            "the arrow is the control"
        );
        let (heard, html) = send(readonly_segments, true, "keydown", first_keydown, || {
            key_event(Key::ArrowRight)
        });
        assert_eq!(heard, Vec::<String>::new());
        assert_eq!(checked_states(&html), [true, false]);
        assert!(html.contains("role=\"radiogroup\""), "{html}");
        assert!(html.contains("aria-readonly=\"true\""), "{html}");

        let (_, html) = send(readonly_segments, false, "click", last_click, click_event);
        assert_eq!(
            checked_states(&html),
            [false, true],
            "the click is the control"
        );
        let (heard, html) = send(readonly_segments, true, "click", last_click, click_event);
        assert_eq!(heard, Vec::<String>::new());
        assert_eq!(checked_states(&html), [true, false]);

        let space = || key_event(Key::Character(" ".into()));
        let (picked, _) = send(readonly_segments, false, "keydown", last_keydown, space);
        assert_eq!(picked, ["true"], "the key is the control");
        let (heard, _) = send(readonly_segments, true, "keydown", last_keydown, space);
        assert_eq!(heard, Vec::<String>::new());
    }

    fn segments_in_a_form() -> Element {
        let mut value = use_signal(|| Emphasis::Bold);
        rsx! {
            LiberoProvider {
                Form::<()> {
                    SegmentedControl {
                        label: "Emphasis",
                        value: value(),
                        onchange: move |next| {
                            heard(next == Emphasis::Italic);
                            value.set(next);
                        },
                    }
                }
            }
        }
    }

    /// Todo 648: outside a `Form`, Enter picks the segment, as Space does. Inside
    /// one it is left to the browser, which submits as for a native radio (508).
    #[test]
    fn enter_picks_a_segment_outside_a_form_only() {
        let enter = || key_event(Key::Enter);
        let (heard, html) = send(readonly_segments, false, "keydown", last_keydown, enter);
        assert_eq!(heard, ["true"], "outside a form");
        assert_eq!(checked_states(&html), [false, true]);
        let (heard, html) = send(segments_in_a_form, false, "keydown", last_keydown, enter);
        assert_eq!(heard, Vec::<String>::new(), "inside a form");
        assert_eq!(checked_states(&html), [true, false]);
    }

    fn tier_segments() -> Element {
        rsx! {
            LiberoProvider {
                SegmentedControl {
                    aria_label: "Tier",
                    value: TIER.get(),
                    options: OptionList::from_options()
                        .disabling(|tier| TIERS_OFF.with_borrow(|off| off.contains(tier))),
                    onchange: move |next: Tier| heard(next),
                }
            }
        }
    }

    /// Mounts `tier_segments` with `Pro` picked and disabled, and returns what one
    /// `key` on segment `at` picked.
    fn segment_press_around_a_disabled_pick(at: usize, key: Key) -> Vec<String> {
        TIER.set(Some(Tier::Pro));
        TIERS_OFF.set(vec![Tier::Pro]);
        HEARD.with_borrow_mut(Vec::clear);
        let Page { mut dom, rec: find } = Page::build(tier_segments);
        assert_eq!(find.keydown.len(), 3, "one keydown per segment");
        dom.runtime().handle_event(
            "keydown",
            Event::new(key_event(key), true),
            find.keydown[at],
        );
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        HEARD.with_borrow(Clone::clone)
    }

    /// Unlike Tabs (todo 403), a disabled pick does not send both arrows the same
    /// way: each neighbour steps past it in its own direction.
    #[test]
    fn segment_arrows_step_past_a_disabled_pick_both_ways() {
        assert_eq!(
            segment_press_around_a_disabled_pick(0, Key::ArrowRight),
            ["Team"]
        );
        assert_eq!(
            segment_press_around_a_disabled_pick(2, Key::ArrowLeft),
            ["Free"]
        );
        assert_eq!(
            segment_press_around_a_disabled_pick(0, Key::ArrowLeft),
            ["Team"]
        );
        assert_eq!(
            segment_press_around_a_disabled_pick(2, Key::ArrowRight),
            ["Free"]
        );
        // The disabled radio takes no focus in a browser, and answers no key here.
        assert!(segment_press_around_a_disabled_pick(1, Key::ArrowRight).is_empty());
    }
}

/// The date-time flow drops its segments from the tab order: no radio is a
/// tab stop, where the default keeps the checked one.
#[test]
fn a_non_focusable_segmented_control_leaves_the_tab_order() {
    #[component]
    fn Segments(focusable: bool) -> Element {
        rsx! {
            LiberoProvider {
                SegmentedControl::<String> {
                    value: "a".to_string(),
                    options: vec!["a".to_string(), "b".to_string()],
                    onchange: move |_| {},
                    focusable,
                }
            }
        }
    }
    let stops = |focusable| {
        let mut dom = VirtualDom::new_with_props(Segments, SegmentsProps { focusable });
        dom.rebuild_in_place();
        let html = body(&dioxus_ssr::render(&dom));
        let radios = tags_with(&html, r#"type="radio""#);
        assert_eq!(radios.len(), 2, "{html}");
        radios
            .iter()
            .filter(|radio| radio.get("tabindex").is_none_or(|index| index != "-1"))
            .count()
    };

    assert_eq!(stops(false), 0);
    assert_eq!(stops(true), 1);
}
