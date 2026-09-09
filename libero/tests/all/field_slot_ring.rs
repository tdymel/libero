//! A button in a field's slot rings itself, not the whole frame (todo 410).

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{PasswordField, PhoneField},
};

/// The frame's own overlay is the only one: a slot or the phone picker adding
/// one would ring the frame again when its button takes focus.
fn assert_only_the_frame_overlay(html: &str) {
    let body = body(html);
    assert_eq!(body.matches("data-ring").count(), 1, "{body}");
}

/// The button carries a class whose rule draws the ring on its own
/// `:focus-visible`.
fn assert_rings_itself(html: &str, button: &str) {
    let classes = attributes_of(&body(html), button)["class"].clone();
    assert!(
        classes
            .split_whitespace()
            .any(|class| html.contains(&format!(".{class}:focus-visible{{outline:"))),
        "{button} has no own ring among {classes}: {html}"
    );
}

#[test]
fn the_reveal_button_rings_itself() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                PasswordField { label: "Password" }
            }
        }
    }

    let html = render(app);
    assert_only_the_frame_overlay(&html);
    assert_rings_itself(&html, "button");
}

#[test]
fn the_phone_picker_rings_itself() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                PhoneField { label: "Phone" }
            }
        }
    }

    let html = render(app);
    assert_only_the_frame_overlay(&html);
    assert_rings_itself(&html, "button");
}
