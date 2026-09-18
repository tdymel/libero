//! Blitz draws no `placeholder`, so a framed field draws its own: shown while
//! the control holds no text, hidden once it does, back after clearing.

use dioxus::prelude::*;
use libero::components::{NumberField, TagsField, TextField, Textarea};
use native_tests::{Key, Page, mount};

const SHOWN: &str = "[data-lsx-placeholder-shown]";

fn typing(page: &mut Page, text: &str) {
    for c in text.chars() {
        page.press(Key::Character(c.to_string()));
    }
}

/// Pixels along the control's first line that differ from its background.
fn inked(page: &Page, control: &str) -> usize {
    let (x, y, width, _) = page.rect(control);
    let row = (y + 10.0) as u32;
    let points: Vec<(u32, u32)> = (0..60).map(|i| ((x + 2.0) as u32 + i, row)).collect();
    let background = page.painted_pixels(&[((x + width - 4.0) as u32, row)])[0];
    page.painted_pixels(&points)
        .into_iter()
        .filter(|pixel| *pixel != background)
        .count()
}

fn round_trip(mut page: Page, control: &str) {
    assert!(page.exists(SHOWN), "no placeholder shown: {}", page.tree());
    assert!(inked(&page, control) > 0, "the placeholder painted nothing");
    page.click(control);
    typing(&mut page, "a");
    assert!(
        !page.exists(SHOWN),
        "still shown after typing: {}",
        page.tree()
    );
    page.press(Key::Backspace);
    assert!(
        page.exists(SHOWN),
        "not back after clearing: {}",
        page.tree()
    );
}

#[test]
fn a_text_field_draws_its_placeholder_while_empty() {
    fn app() -> Element {
        let mut value = use_signal(String::new);
        rsx! {
            TextField {
                label: "Name",
                placeholder: "Ada Lovelace",
                value: value(),
                oninput: move |next| value.set(next),
            }
        }
    }
    let page = mount(app);
    let (span, input) = (page.rect("[data-lsx-placeholder]"), page.rect("input"));
    let middle = |(_, y, _, height): (f64, f64, f64, f64)| y + height / 2.0;
    assert!(
        (span.0 - input.0).abs() < 1.0 && (middle(span) - middle(input)).abs() <= 1.0,
        "placeholder at {span:?}, input at {input:?}"
    );
    round_trip(page, "input");
}

#[test]
fn an_uncontrolled_text_field_draws_its_placeholder_while_empty() {
    fn app() -> Element {
        rsx! {
            TextField { label: "Name", placeholder: "Ada Lovelace" }
        }
    }
    round_trip(mount(app), "input");
}

#[test]
fn a_placeholder_leaves_the_input_in_place() {
    fn with() -> Element {
        rsx! {
            TextField { label: "Name", placeholder: "Ada Lovelace" }
        }
    }
    fn without() -> Element {
        rsx! {
            TextField { label: "Name" }
        }
    }
    let (with, without) = (mount(with).rect("input"), mount(without).rect("input"));
    assert_eq!(with, without, "the placeholder moved the input");
}

#[test]
fn a_number_field_draws_its_placeholder_while_empty() {
    fn app() -> Element {
        rsx! {
            NumberField::<i32> { label: "Count", placeholder: "0" }
        }
    }
    let mut page = mount(app);
    assert!(page.exists(SHOWN), "no placeholder shown: {}", page.tree());
    page.click("input");
    typing(&mut page, "4");
    assert!(
        !page.exists(SHOWN),
        "still shown after typing: {}",
        page.tree()
    );
}

#[test]
fn a_textarea_draws_its_placeholder_at_the_top() {
    fn app() -> Element {
        let mut value = use_signal(String::new);
        rsx! {
            Textarea {
                label: "Notes",
                placeholder: "Start typing",
                value: value(),
                oninput: move |next| value.set(next),
            }
        }
    }
    let page = mount(app);
    let (span, area) = (page.rect("[data-lsx-placeholder]"), page.rect("textarea"));
    assert!(
        (span.1 - area.1).abs() < 1.0,
        "placeholder at {span:?}, textarea at {area:?}"
    );
    round_trip(page, "textarea");
}

#[test]
fn a_tags_field_draws_its_placeholder_until_a_tag_is_held() {
    fn app() -> Element {
        let mut tags = use_signal(Vec::<String>::new);
        rsx! {
            TagsField {
                label: "Topics",
                placeholder: "Add a topic",
                value: tags(),
                onchange: move |next| tags.set(next),
            }
        }
    }
    let mut page = mount(app);
    assert!(page.exists(SHOWN), "no placeholder shown: {}", page.tree());
    page.click("input");
    typing(&mut page, "rust");
    assert!(!page.exists(SHOWN), "still shown after typing");
    page.press(Key::Enter);
    assert!(
        !page.exists("[data-lsx-placeholder]"),
        "drawn beside a held tag: {}",
        page.tree()
    );
}
