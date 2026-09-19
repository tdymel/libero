//! `FocusReturn` behind a click: Blitz runs the click handler before it moves
//! focus, so libero remembers the pointerdown target instead (todo 262(a)).
//! The click and keyboard returns are e2e's shared scenarios (`focus_return::`);
//! this pins the Blitz-only gap.

use dioxus::prelude::*;
use libero::{
    components::{Button, Dialog},
    hooks::{ModalScope, use_modal},
};
use native_tests::{Key, mount};

const TRIGGER: &str = "#open-modal";
const DIALOG: &str = "[role=dialog]";

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
