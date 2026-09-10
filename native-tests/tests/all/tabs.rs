//! `Tabs` activates automatically: ArrowRight moves focus and selection
//! together, and Tab leaves the strip from the selected tab.

use dioxus::prelude::*;
use libero::components::{Options, Tabs};
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
