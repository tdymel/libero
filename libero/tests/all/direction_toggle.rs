//! `DirectionToggle`'s rendered contract: the name and the arrow say where a
//! press turns the text, from the provider's `direction` and the localization.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider, components::DirectionToggle, localization::Localization, theme::Direction,
};

/// The arrow of each glyph: left for "to right to left", right for back.
const TO_RTL: &str = pictogram_icons_lucide::pilcrow_left::outlined.body;
const TO_LTR: &str = pictogram_icons_lucide::pilcrow_right::outlined.body;

#[test]
fn left_to_right_it_offers_right_to_left() {
    fn app() -> Element {
        rsx! { LiberoProvider { DirectionToggle {} } }
    }
    let html = body(&render(app));
    let button = attributes_of(&html, "button");
    assert_eq!(button["aria-label"], "Switch to right-to-left text");
    assert!(
        !button.contains_key("aria-pressed"),
        "an action, not a toggle state"
    );
    assert!(html.contains(TO_RTL) && !html.contains(TO_LTR), "{html}");
}

#[test]
fn right_to_left_it_offers_the_way_back_in_the_localized_words() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { direction: Direction::Rtl, localization: &Localization::GERMAN,
                DirectionToggle {}
            }
        }
    }
    let html = body(&render(app));
    let button = attributes_of(&html, "button");
    assert_eq!(
        button["aria-label"],
        "Zu Text von links nach rechts wechseln"
    );
    assert!(html.contains(TO_LTR) && !html.contains(TO_RTL), "{html}");
}
