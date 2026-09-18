//! `Switch` is a visually hidden checkbox with `role="switch"`: the track takes
//! the pointer, the input the keyboard.

use dioxus::prelude::*;
use libero::components::{Form, Switch};
use native_tests::{Key, Page, mount};

const INPUT: &str = "[role=switch]";
const TRACK: &str = "[role=switch] + [aria-hidden=true]";

fn app() -> Element {
    let mut on = use_signal(|| false);
    rsx! {
        Switch { checked: on(), onchange: move |v| on.set(v), label: "Wifi" }
    }
}

fn checked(page: &Page) -> bool {
    page.attr(INPUT, "checked").is_some_and(|v| v == "true")
}

#[test]
fn a_click_on_the_track_toggles_it() {
    let mut page = mount(app);
    assert!(!checked(&page));
    page.click(TRACK);
    assert!(checked(&page), "{}", page.tree());
    page.click(TRACK);
    assert!(!checked(&page));
}

#[test]
fn a_click_on_the_label_toggles_it() {
    let mut page = mount(app);
    page.click("label");
    assert!(checked(&page), "{}", page.tree());
}

#[test]
fn the_thumb_moves_once_its_transition_ends() {
    const THUMB: &str = "[role=switch] + [aria-hidden=true] > span";
    let mut page = mount(app);
    let off = page.computed(THUMB, "transform");
    page.click(TRACK);
    page.advance(1.0);
    let on = page.computed(THUMB, "transform");
    assert_ne!(off, on, "the thumb stayed at {off}");
}

/// What Blitz paints, not only what stylo computed: the thumb's transform
/// comes from a custom property its track declares.
#[test]
fn the_painted_thumb_moves_too() {
    const THUMB: &str = "[role=switch] + [aria-hidden=true] > span";
    let mut page = mount(app);
    let off = page.painted_transform(THUMB);
    page.click(TRACK);
    page.advance(1.0);
    let on = page.painted_transform(THUMB);
    assert_ne!(off, on, "the painted thumb stayed at {off:?}");
}

/// Todo 781: an off thumb starts at the right under an `[dir=rtl]` ancestor,
/// which stylo matches where it does not match `:dir(rtl)`.
#[test]
fn an_rtl_thumb_starts_at_the_right() {
    const THUMB: &str = "[role=switch] + [aria-hidden=true] > span";
    fn rtl() -> Element {
        rsx! {
            div { dir: "rtl",
                Switch { checked: false, onchange: move |_| {}, label: "Wifi" }
            }
        }
    }
    let page = mount(rtl);
    let (track_x, _, track_w, _) = page.rect(TRACK);
    let (thumb_x, _, thumb_w, _) = page.rect(THUMB);
    assert!(
        thumb_x + thumb_w / 2.0 > track_x + track_w / 2.0,
        "the thumb sits at {thumb_x} in a track at {track_x}+{track_w}"
    );
}

/// Outside a `Form`, Enter toggles as Space does (todo 648).
#[test]
fn space_and_enter_toggle_the_focused_switch() {
    let mut page = mount(app);
    page.focus(INPUT);
    page.press(Key::Character(" ".into()));
    assert!(checked(&page), "Space");
    page.press(Key::Enter);
    assert!(!checked(&page), "Enter");
    assert!(page.is_focused(INPUT));
}

/// Todo 757: Blitz ignores `clip`, so the hidden input painted a speck beside
/// the track, and its focus outline a blue square; only the ring may show.
#[test]
fn the_hidden_input_paints_nothing() {
    fn padded() -> Element {
        let mut on = use_signal(|| false);
        rsx! {
            div { padding: "10px",
                Switch { checked: on(), onchange: move |v| on.set(v), label: "Wifi" }
            }
        }
    }
    let mut page = mount(padded);
    let (x, y, ..) = page.rect(INPUT);
    // Around the input, inside the ring, which starts 2px left of it.
    let (x, y) = (x as u32, y as u32);
    let window: Vec<_> = (x - 1..x + 3)
        .flat_map(|px| (y - 4..y + 5).map(move |py| (px, py)))
        .collect();
    let background = page.painted_pixel(1, 1);
    assert_eq!(page.painted_pixel(x, y), background, "a speck at the input");
    let idle = page.painted_pixels(&window);
    page.tab();
    assert!(page.is_focused(INPUT), "Tab reached {}", page.focus_owner());
    assert_eq!(
        page.painted_pixels(&window),
        idle,
        "the focused input painted"
    );
}

fn form_app() -> Element {
    let mut on = use_signal(|| false);
    let mut submits = use_signal(|| 0u32);
    rsx! {
        Form::<()> { onsubmit: move |_| submits += 1,
            Switch { checked: on(), onchange: move |v| on.set(v), label: "Wifi" }
            button { r#type: "submit", "Save" }
        }
        span { id: "submits", "{submits}" }
    }
}

/// Inside a `Form`, Enter is the form's: it submits, as for a native checkbox,
/// and leaves the switch alone (todo 508).
#[test]
fn enter_in_a_form_submits_and_toggles_nothing() {
    let mut page = mount(form_app);
    page.focus(INPUT);
    page.press(Key::Enter);
    assert!(!checked(&page), "Enter toggled it");
    assert_eq!(page.text("#submits"), "1", "{}", page.tree());
}
