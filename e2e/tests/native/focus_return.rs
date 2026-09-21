//! `FocusReturn` behind a click: Blitz runs the click handler before moving focus, so libero
//! remembers the pointerdown target (todo 262(a)). The rest is shared (`focus_return::`).

use dioxus::prelude::*;
use e2e::native::{Key, mount};
use libero::{
    components::{Button, Dialog},
    hooks::{ModalScope, use_modal},
};

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

/// Pins a gap: libero listens in bubble phase (no capture in this dioxus), so a trigger
/// stopping its pointerdown hides the press and focus returns to the earlier element.
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
