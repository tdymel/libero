//! Checkbox, Switch, Radio and RadioGroup, driven through dispatched events.

use crate::common::tags_with;
use crate::dispatch::*;
use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Button, Checkbox, Form, OptionList, Radio, RadioGroup, Rule, Switch, use_form},
};

/// Toggling a mixed checkbox gives `true`, as its prop doc promises: a mixed
/// "select all" fills its group rather than clearing it (todo 412).
#[test]
fn clicking_an_indeterminate_checkbox_reports_true() {
    fn app() -> Element {
        let reported = use_signal(Vec::<bool>::new);
        rsx! {
            LiberoProvider {
                Checkbox {
                    label: "Select all",
                    checked: false,
                    indeterminate: true,
                    onchange: move |on: bool| {
                        let mut reported = reported;
                        reported.write().push(on);
                    },
                }
                "reported: {reported():?}"
            }
        }
    }

    let Page { mut dom, rec: find } = Page::build(app);
    let checkbox = find.element("click", "text", "Select all");

    dom.runtime()
        .handle_event("click", Event::new(click_event(), true), checkbox);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    let html = dioxus_ssr::render(&dom);
    let reported = html
        .split("reported: ")
        .nth(1)
        .and_then(|rest| rest.split('<').next());
    assert_eq!(
        reported,
        Some("[true]"),
        "a mixed checkbox toggles to `true`"
    );
}

/// A field with rules, no value of its own and no place in a form's value used
/// to validate `T::default()` for ever, so inside a `Form` it cancelled every
/// submit however the user answered it (todo 185). The checkbox is the case
/// where the browser keeps no state either: what the user ticked lives only in
/// the field, so it is both what renders and what the rules judge.
#[test]
fn ticking_an_uncontrolled_checkbox_satisfies_its_own_rules() {
    fn app() -> Element {
        let handle = use_form();
        let valid = use_signal(|| true);

        rsx! {
            LiberoProvider {
                Button {
                    id: "submit",
                    onclick: move |_| {
                        let mut valid = valid;
                        valid.set(handle.validate());
                    },
                    "Submit"
                }
                Form::<()> { form: handle,
                    Checkbox { label: "Agree", validate: (|on: &bool| *on).error("Tick it") }
                }
                "valid: {valid}"
            }
        }
    }

    let Page { mut dom, rec: find } = Page::build(app);
    let submit = find.element("click", "id", "submit");
    let checkbox = find.element("click", "text", "Agree");

    let click = |dom: &mut VirtualDom, id| {
        dom.runtime()
            .handle_event("click", Event::new(click_event(), true), id);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    };

    click(&mut dom, submit);
    assert!(
        dioxus_ssr::render(&dom).contains("valid: false"),
        "an unticked required checkbox must block the submit"
    );

    click(&mut dom, checkbox);
    assert_eq!(checked_states(&dioxus_ssr::render(&dom)), [true]);

    click(&mut dom, submit);
    assert!(
        dioxus_ssr::render(&dom).contains("valid: true"),
        "the rules must judge what was ticked, not the default"
    );
}

/// A read-only checkbox keeps its tab stop and its place in the post, so the
/// one thing that must not happen is the toggle - and the activation is ours,
/// not the browser's, so only Rust can refuse it (todo 209).
#[test]
fn a_read_only_checkbox_refuses_its_own_activation() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Checkbox { label: "Agree", readonly: true }
            }
        }
    }

    let Page { mut dom, rec: find } = Page::build(app);
    let label = find.click.expect("registered no click listener");

    dom.runtime()
        .handle_event("click", Event::new(click_event(), true), label);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    assert_eq!(checked_states(&dioxus_ssr::render(&dom)), [false]);
}

fn readonly_checkbox() -> Element {
    rsx! {
        LiberoProvider {
            Checkbox {
                label: "Agree",
                checked: false,
                readonly: READ_ONLY.get(),
                onchange: move |on: bool| heard(on),
            }
        }
    }
}

fn readonly_switch() -> Element {
    rsx! {
        LiberoProvider {
            Switch {
                label: "Wi-Fi",
                checked: false,
                readonly: READ_ONLY.get(),
                onchange: move |on: bool| heard(on),
            }
        }
    }
}

fn readonly_radio() -> Element {
    rsx! {
        LiberoProvider {
            Radio {
                label: "Tea",
                checked: false,
                readonly: READ_ONLY.get(),
                onselect: move |_| heard("tea"),
            }
        }
    }
}

/// Todo 489: a read-only checkable refuses the click, and a switch its Space
/// too; each checked against the same control editable, where it must change.
#[test]
fn a_read_only_checkable_refuses_every_activation() {
    let space = || key_event(Key::Character(" ".into()));
    for (app, name, data) in [
        (
            readonly_checkbox as fn() -> Element,
            "click",
            click_event as fn() -> _,
        ),
        (readonly_switch, "click", click_event),
        (readonly_switch, "keydown", space),
        (readonly_radio, "click", click_event),
    ] {
        let pick = if name == "click" {
            last_click
        } else {
            last_keydown
        };
        let (changed, _) = send(app, false, name, pick, data);
        assert_eq!(changed.len(), 1, "{name} changed nothing while editable");
        let (heard, _) = send(app, true, name, pick, data);
        assert_eq!(heard, Vec::<String>::new(), "{name}");
    }
}

fn switch_in_a_form() -> Element {
    rsx! {
        LiberoProvider {
            Form::<()> {
                Switch { label: "Wi-Fi", checked: false, onchange: move |on: bool| heard(on) }
            }
        }
    }
}

/// Todo 648: outside a `Form`, Enter toggles a switch (APG). Inside one it is
/// left to the browser, which submits as for a native checkbox (508).
#[test]
fn enter_toggles_a_switch_outside_a_form_only() {
    let enter = || key_event(Key::Enter);
    let (heard, _) = send(readonly_switch, false, "keydown", last_keydown, enter);
    assert_eq!(heard, ["true"], "outside a form");
    let (heard, _) = send(switch_in_a_form, false, "keydown", last_keydown, enter);
    assert_eq!(heard, Vec::<String>::new(), "inside a form");
}

fn readonly_radio_group() -> Element {
    rsx! {
        LiberoProvider {
            RadioGroup {
                label: "Emphasis",
                value: Some(Emphasis::Bold),
                readonly: READ_ONLY.get(),
                onchange: move |next: Emphasis| heard(next == Emphasis::Italic),
            }
        }
    }
}

/// Todo 320: a read-only group refused the arrows by returning early, and so
/// left the browser's own arrow in place - which moves focus to the next radio
/// and checks it natively. Measured in Chromium: the selection held, but focus
/// walked onto options whose `tabindex` is -1. The arrow has to be cancelled
/// either way; only whether it also picks depends on `readonly`.
#[test]
fn a_read_only_radio_group_cancels_the_native_arrow() {
    for readonly in [false, true] {
        READ_ONLY.set(readonly);
        HEARD.with_borrow_mut(Vec::clear);
        let Page { mut dom, rec: find } = Page::build(readonly_radio_group);

        let arrow = Event::new(key_event(Key::ArrowDown), true);
        dom.runtime()
            .handle_event("keydown", arrow.clone(), last_keydown(&find));
        dom.render_immediate(&mut dioxus::core::NoOpMutations);

        assert!(
            !arrow.default_action_enabled(),
            "readonly {readonly}: the browser's own arrow was left to run"
        );
        let expected: &[String] = if readonly { &[] } else { &["true".to_string()] };
        assert_eq!(
            HEARD.with_borrow(Clone::clone),
            expected,
            "readonly {readonly}"
        );
    }
}

fn tier_group() -> Element {
    rsx! {
        LiberoProvider {
            RadioGroup {
                label: "Tier",
                value: TIER.get(),
                options: OptionList::from_options()
                    .disabling(|tier| TIERS_OFF.with_borrow(|off| off.contains(tier))),
                onchange: move |next: Tier| heard(next),
            }
        }
    }
}

/// `(disabled, tabindex)` per radio input, in order.
fn radio_inputs(html: &str) -> Vec<(bool, String)> {
    tags_with(html, "<input")
        .into_iter()
        .map(|input| {
            let tabindex = input.get("tabindex").cloned().unwrap_or_default();
            (input.contains_key("disabled"), tabindex)
        })
        .collect()
}

/// Mounts `tier_group`, and returns each input's `(disabled, tabindex)` and
/// what one `key` press picked.
fn tier_press(value: Option<Tier>, off: &[Tier], key: Key) -> (Vec<(bool, String)>, Vec<String>) {
    TIER.set(value);
    TIERS_OFF.set(off.to_vec());
    HEARD.with_borrow_mut(Vec::clear);
    let Page { mut dom, rec: find } = Page::build(tier_group);
    let inputs = radio_inputs(&dioxus_ssr::render(&dom));
    dom.runtime().handle_event(
        "keydown",
        Event::new(key_event(key), true),
        last_keydown(&find),
    );
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    (inputs, HEARD.with_borrow(Clone::clone))
}

/// Todo 221: a disabled `OptionItem` greys out a single option, and the arrows
/// step over them, wrapping.
#[test]
fn a_radio_group_steps_over_the_options_its_list_disabled() {
    let (inputs, picked) = tier_press(Some(Tier::Free), &[Tier::Pro], Key::ArrowDown);
    let off: Vec<bool> = inputs.iter().map(|(off, _)| *off).collect();
    assert_eq!(off, [false, true, false], "{inputs:?}");
    assert_eq!(picked, ["Team"]);

    let (_, picked) = tier_press(Some(Tier::Team), &[Tier::Pro], Key::ArrowUp);
    assert_eq!(picked, ["Free"]);
    // Positive control: without the option disabled, the arrow lands on it.
    let (_, picked) = tier_press(Some(Tier::Free), &[], Key::ArrowDown);
    assert_eq!(picked, ["Pro"]);
}

/// A disabled input takes no focus, so a tab stop on one would drop the group
/// out of the Tab order. The stop moves to the first option that can be picked.
#[test]
fn a_radio_group_never_puts_its_tab_stop_on_a_disabled_option() {
    let stop = |inputs: &[(bool, String)]| {
        (0..inputs.len())
            .filter(|&i| inputs[i].1 == "0")
            .collect::<Vec<_>>()
    };
    let (inputs, _) = tier_press(None, &[Tier::Free], Key::Tab);
    assert_eq!(stop(&inputs), [1], "{inputs:?}");
    let (inputs, _) = tier_press(Some(Tier::Pro), &[Tier::Pro], Key::Tab);
    assert_eq!(stop(&inputs), [0], "{inputs:?}");
    let (inputs, _) = tier_press(Some(Tier::Pro), &[], Key::Tab);
    assert_eq!(stop(&inputs), [1], "{inputs:?}");
}
