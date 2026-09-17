//! `Tabs` activates automatically: ArrowRight moves focus and selection
//! together, and Tab leaves the strip from the selected tab.

use dioxus::prelude::*;
use libero::components::{OptionList, Options, Tabs, TabsActivation};
use native_tests::{Key, mount};

#[derive(Clone, Copy, PartialEq, Options)]
enum Pane {
    One,
    Two,
    Three,
}

fn app() -> Element {
    let mut pane = use_signal(|| Pane::One);
    rsx! {
        Tabs {
            value: pane(),
            onchange: move |v| pane.set(v),
            panel: move |v: Pane| rsx! {
                button { id: "inside", "{v.label()}" }
            },
        }
    }
}

const SELECTED: &str = "[role=tab][aria-selected=true]";

#[test]
fn arrow_right_moves_focus_and_selection_to_the_next_tab() {
    let mut page = mount(app);
    assert_eq!(page.text(SELECTED), "One");
    page.focus(SELECTED);

    page.press(Key::ArrowRight);
    assert_eq!(page.text(SELECTED), "Two", "{}", page.tree());
    assert!(
        page.is_focused(SELECTED),
        "focus is on {}",
        page.focus_owner()
    );
    assert_eq!(page.text("#inside"), "Two");
}

#[test]
fn arrow_left_wraps_and_home_and_end_jump() {
    let mut page = mount(app);
    page.focus(SELECTED);

    page.press(Key::ArrowLeft);
    assert_eq!(page.text(SELECTED), "Three", "{}", page.tree());
    assert!(
        page.is_focused(SELECTED),
        "focus is on {}",
        page.focus_owner()
    );
    page.press(Key::Home);
    assert_eq!(page.text(SELECTED), "One", "{}", page.tree());
    page.press(Key::End);
    assert_eq!(page.text(SELECTED), "Three", "{}", page.tree());
    assert!(
        page.is_focused(SELECTED),
        "focus is on {}",
        page.focus_owner()
    );
    page.press(Key::ArrowRight);
    assert_eq!(page.text(SELECTED), "One", "{}", page.tree());
    assert_eq!(page.text("#inside"), "One");
}

fn manual_app() -> Element {
    let mut pane = use_signal(|| Pane::Three);
    rsx! {
        Tabs {
            activation: TabsActivation::Manual,
            value: pane(),
            onchange: move |v| pane.set(v),
            panel: move |v: Pane| rsx! {
                button { id: "inside", "{v.label()}" }
            },
        }
    }
}

/// Manual mode: Tab leaves from the focused tab, not from the selected one
/// further along the strip.
#[test]
fn tab_leaves_the_strip_from_a_focused_unselected_tab() {
    let mut page = mount(manual_app);
    page.focus(SELECTED);
    page.press(Key::Home);
    assert_eq!(page.text(SELECTED), "Three", "{}", page.tree());

    page.tab();
    assert!(
        !page.is_focused("[role=tab]"),
        "Tab stayed in the strip on {}",
        page.focus_owner()
    );
    page.shift_tab();
    assert!(
        page.is_focused(SELECTED),
        "Shift+Tab landed on {}",
        page.focus_owner()
    );
}

fn disabled_app() -> Element {
    let mut pane = use_signal(|| Pane::One);
    rsx! {
        Tabs {
            value: pane(),
            onchange: move |v| pane.set(v),
            options: OptionList::from_options().disabling(|p| *p == Pane::Two),
            panel: move |v: Pane| rsx! {
                button { id: "inside", "{v.label()}" }
            },
        }
    }
}

/// A click focuses a disabled tab without selecting it; Shift+Tab then leaves
/// the strip rather than stopping on the selected tab before it.
#[test]
fn shift_tab_leaves_the_strip_from_a_clicked_disabled_tab() {
    let mut page = mount(disabled_app);
    page.click("[role=tab][aria-disabled=true]");
    assert_eq!(page.text(SELECTED), "One", "{}", page.tree());

    page.shift_tab();
    assert!(
        !page.is_focused("[role=tab]"),
        "Shift+Tab stayed in the strip on {}",
        page.focus_owner()
    );
}

#[test]
fn tab_leaves_the_strip_and_shift_tab_comes_back_to_the_selected_tab() {
    let mut page = mount(app);
    page.focus(SELECTED);
    page.press(Key::ArrowRight);

    page.tab();
    assert!(
        !page.is_focused("[role=tab]"),
        "Tab stayed in the strip on {}",
        page.focus_owner()
    );
    page.shift_tab();
    assert!(
        page.is_focused(SELECTED),
        "Shift+Tab landed on {}",
        page.focus_owner()
    );
}
