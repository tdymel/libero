//! `FocusReturn` behind a click: Blitz runs the click handler before it moves
//! focus, so libero remembers the pointerdown target instead (todo 262(a)).

use dioxus::prelude::*;
use libero::{
    components::{Button, Dialog},
    hooks::{ModalScope, use_modal},
};
use native_tests::{Key, mount};

const TRIGGER: &str = "#open-modal";
const DIALOG: &str = "[role=dialog]";

fn app() -> Element {
    let prompt = use_modal(|s: ModalScope<()>| {
        rsx! {
            Dialog {
                title: "Unsaved changes",
                Button { onclick: move |_| s.close(), "Keep editing" }
            }
        }
    });
    rsx! {
        button { id: "elsewhere", "Elsewhere" }
        Button {
            id: "open-modal",
            onclick: move |_| {
                prompt.open();
            },
            "Close editor"
        }
    }
}

#[test]
fn a_modal_opened_by_click_returns_focus_to_its_trigger() {
    let mut page = mount(app);
    // Focus elsewhere first, so a stale snapshot has somewhere wrong to go.
    page.click("#elsewhere");
    assert!(page.is_focused("#elsewhere"));

    page.click(TRIGGER);
    assert!(
        page.exists(DIALOG),
        "the click did not open it:\n{}",
        page.tree()
    );

    page.press(Key::Escape);
    assert!(!page.exists(DIALOG), "Escape did not close it");
    assert!(
        page.is_focused(TRIGGER),
        "focus is on {}, not the trigger",
        page.focus_owner()
    );
}

fn stopping_app() -> Element {
    let prompt = use_modal(|s: ModalScope<()>| {
        rsx! {
            Dialog {
                title: "Unsaved changes",
                Button { onclick: move |_| s.close(), "Keep editing" }
            }
        }
    });
    rsx! {
        button { id: "elsewhere", "Elsewhere" }
        button {
            id: "open-modal",
            onpointerdown: |event| event.stop_propagation(),
            onclick: move |_| {
                prompt.open();
            },
            "Close editor"
        }
    }
}

/// Pins a gap: libero's listener sits above the app in bubble phase (this
/// dioxus has no capture listeners), so a trigger that stops its pointerdown
/// hides the press and focus returns to the element focused before.
#[test]
fn a_trigger_that_stops_its_pointerdown_gets_the_old_answer() {
    let mut page = mount(stopping_app);
    page.click("#elsewhere");

    page.click(TRIGGER);
    assert!(page.exists(DIALOG));
    page.press(Key::Escape);
    assert!(!page.exists(DIALOG));
    assert!(
        page.is_focused("#elsewhere"),
        "focus is on {}",
        page.focus_owner()
    );
}

#[test]
fn a_modal_opened_by_keyboard_still_returns_focus_to_its_trigger() {
    let mut page = mount(app);
    page.click("#elsewhere");
    page.focus(TRIGGER);

    page.press(Key::Enter);
    assert!(
        page.exists(DIALOG),
        "Enter did not open it:\n{}",
        page.tree()
    );

    page.press(Key::Escape);
    assert!(!page.exists(DIALOG), "Escape did not close it");
    assert!(
        page.is_focused(TRIGGER),
        "focus is on {}, not the trigger",
        page.focus_owner()
    );
}
