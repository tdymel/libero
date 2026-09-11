//! `ColorPicker` and `ColorField`. `#1c7ed6` is saturation 87%, brightness
//! 84%: room to step both ways.

use dioxus::prelude::*;
use libero::components::{Button, ColorCode, ColorField, ColorPicker, SliderChangeEvent};
use native_tests::{Key, Modifiers, Page, mount};

const PAD: &str = "[role=slider][aria-label=Saturation]";
const HUE: &str = "[role=slider][aria-label=Hue]";

fn start() -> ColorCode {
    "#1c7ed6".parse().unwrap()
}

fn picker() -> Element {
    let mut color = use_signal(start);
    rsx! {
        Button { id: "before", "Before" }
        ColorPicker {
            value: color(),
            saturation_label: "Saturation",
            hue_label: "Hue",
            swatches: vec!["#fa5252", "#40c057"],
            oninput: move |event: SliderChangeEvent<ColorCode>| color.set(event.value()),
        }
        span { id: "echo", "{color().to_hex()}" }
    }
}

/// `saturation,brightness` from the pad thumb's ARIA.
fn reading(page: &Page) -> String {
    let now = page.attr(PAD, "aria-valuenow").unwrap_or_default();
    let text = page.attr(PAD, "aria-valuetext").unwrap_or_default();
    let brightness = text
        .split("brightness ")
        .nth(1)
        .and_then(|rest| rest.split('%').next())
        .unwrap_or_default();
    format!("{now},{brightness}")
}

#[test]
fn the_arrows_move_the_pad_on_both_axes() {
    let mut page = mount(picker);
    page.focus(PAD);
    assert_eq!(reading(&page), "87,84", "{}", page.tree());
    page.press(Key::ArrowRight);
    assert_eq!(reading(&page), "88,84");
    page.press(Key::ArrowUp);
    assert_eq!(reading(&page), "88,85");
    page.press(Key::ArrowLeft);
    page.press(Key::ArrowDown);
    assert_eq!(reading(&page), "87,84");
    page.press_with(Key::ArrowDown, Modifiers::SHIFT);
    assert_eq!(reading(&page), "87,74");
}

#[test]
fn a_drag_on_the_pad_moves_both_axes_and_focuses_the_thumb() {
    let mut page = mount(picker);
    page.click("#before");
    let before = reading(&page);
    // Left and down: less saturated, darker.
    page.drag(PAD, -40.0, 40.0);
    let after = reading(&page);
    let parse = |r: &str| -> Vec<i32> { r.split(',').filter_map(|v| v.parse().ok()).collect() };
    let (b, a) = (parse(&before), parse(&after));
    assert!(a[0] < b[0] && a[1] < b[1], "{before} -> {after}");
    assert!(page.is_focused(PAD), "{}", page.focus_owner());
}

#[test]
fn the_arrows_step_the_hue() {
    let mut page = mount(picker);
    let before = page.attr(HUE, "aria-valuenow");
    page.focus(HUE);
    page.press_with(Key::ArrowRight, Modifiers::SHIFT);
    assert_ne!(page.attr(HUE, "aria-valuenow"), before, "{}", page.tree());
    assert_ne!(page.text("#echo"), "#1c7ed6");
}

#[test]
fn a_click_on_a_swatch_picks_its_colour() {
    let mut page = mount(picker);
    page.click("[aria-label='#40c057'], [data-color='#40c057']");
    assert_eq!(page.text("#echo"), "#40c057", "{}", page.tree());
}

fn field() -> Element {
    let mut color = use_signal(start);
    rsx! {
        ColorField {
            label: "Accent",
            value: color(),
            swatches: vec!["#fa5252", "#40c057"],
            oninput: move |event: SliderChangeEvent<ColorCode>| {
                if let SliderChangeEvent::Change(next) = event {
                    color.set(next);
                }
            },
        }
        span { id: "echo", "{color().to_hex()}" }
    }
}

const INPUT: &str = "input[data-controlled]";

#[test]
fn a_typed_colour_commits_on_enter() {
    let mut page = mount(field);
    page.click(INPUT);
    page.press(Key::End);
    for _ in 0..10 {
        page.press(Key::Backspace);
    }
    for c in "#228be6".chars() {
        page.press(Key::Character(c.to_string()));
    }
    page.press(Key::Enter);
    assert_eq!(page.text("#echo"), "#228be6", "{}", page.tree());
}

#[test]
fn arrow_down_enters_the_colour_dialog_and_escape_returns() {
    let mut page = mount(field);
    page.click(INPUT);
    assert!(page.exists("[role=dialog]"), "{}", page.tree());
    page.press(Key::ArrowDown);
    let inside = page
        .focused()
        .is_some_and(|id| page.query_all("[role=dialog] *").contains(&id));
    assert!(inside, "focus is on {}", page.focus_owner());
    page.press(Key::Escape);
    assert_eq!(page.attr(INPUT, "aria-expanded").as_deref(), Some("false"));
    assert!(page.is_focused(INPUT), "{}", page.focus_owner());
}
