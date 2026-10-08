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
    assert!(page.wait_for(|page| page.exists(DIALOG)), "{}", page.tree());
    // Escape goes to whatever holds focus: under load it is still on the trigger.
    assert!(
        page.wait_for(|page| page.exists("[role=dialog] :focus")),
        "focus is on {}",
        page.focus_owner()
    );
    page.press(Key::Escape);
    assert!(
        page.wait_for(|page| !page.exists(DIALOG)),
        "{}",
        page.tree()
    );
    // The restore is a spawned task: under load it lands after the close.
    assert!(
        page.wait_for(|page| page.is_focused("#elsewhere")),
        "focus is on {}",
        page.focus_owner()
    );
}
