//! `Form` natively. Blitz fires no `submit` and implements no reset, so a
//! submit button, Enter in a field, `FormHandle::submit` and `reset` all run
//! libero's own path.

use dioxus::prelude::*;
use libero::components::{Button, Fields, Form, Rule, TextField, not_empty, use_form};
use native_tests::{Key, Page, mount};

#[derive(Clone, PartialEq, Default, Fields)]
struct Signup {
    email: String,
}

thread_local! {
    /// What the last submit carried. Each test runs on its own thread.
    static POSTED: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

fn app() -> Element {
    let value = use_store(Signup::default);
    let form = use_form();
    let mut submits = use_signal(|| 0u32);
    rsx! {
        Form { value, form,
            onsubmit: move |event: FormEvent| {
                submits += 1;
                let mut values: Vec<_> = event
                    .values()
                    .into_iter()
                    .map(|(name, value)| format!("{name}={value:?}"))
                    .collect();
                values.sort();
                POSTED.set(values.join(" "));
            },
            TextField { id: "email", label: "Email", name: Signup::FIELDS.email(),
                validate: not_empty.error("Email needed") }
            input { id: "loose", name: "loose" }
            input { id: "tick", r#type: "checkbox", name: "tick", checked: true }
            span { id: "plain", "Not a control" }
            button { id: "other", r#type: "button", "Other" }
            button { id: "cancels", onclick: |event: MouseEvent| event.prevent_default(), "Cancels" }
            button { id: "native-submit", r#type: "submit", "Send" }
        }
        span { id: "submits", "{submits}" }
        span { id: "email-value", "{value.read().email}" }
        Button { id: "submit", onclick: move |_| { let _ = form.submit(); }, "Submit" }
        Button { id: "reset", onclick: move |_| form.reset(), "Reset" }
    }
}

/// Two text fields and no submit button: Enter submits nothing on the web.
fn two_fields() -> Element {
    let mut submits = use_signal(|| 0u32);
    rsx! {
        Form::<()> { onsubmit: move |_| submits += 1,
            input { id: "first", name: "first" }
            input { id: "second", name: "second" }
        }
        span { id: "submits", "{submits}" }
    }
}

fn submits(page: &Page) -> String {
    page.text("#submits")
}

fn type_text(page: &mut Page, selector: &str, text: &str) {
    page.focus(selector);
    for character in text.chars() {
        page.press(Key::Character(character.to_string()));
    }
}

fn filled() -> Page {
    let mut page = mount(app);
    type_text(&mut page, "#email", "a@b");
    assert_eq!(page.text("#email-value"), "a@b", "typing reached no store");
    page
}

#[test]
fn a_submit_button_click_submits_once() {
    let mut page = filled();
    page.click("#native-submit");
    assert_eq!(submits(&page), "1", "{}", page.tree());
}

#[test]
fn enter_on_a_submit_button_submits() {
    let mut page = filled();
    page.focus("#native-submit");
    page.press(Key::Enter);
    assert_eq!(submits(&page), "1");
}

#[test]
fn enter_in_a_text_field_submits() {
    let mut page = filled();
    page.press(Key::Enter);
    assert_eq!(submits(&page), "1");
}

#[test]
fn enter_submits_nothing_with_two_fields_and_no_button() {
    let mut page = mount(two_fields);
    page.focus("#first");
    page.press(Key::Enter);
    assert_eq!(submits(&page), "0");
}

#[test]
fn other_clicks_in_the_form_submit_nothing() {
    let mut page = filled();
    page.click("#other");
    page.click("#cancels");
    // A focused submit button is not what a press elsewhere activates.
    page.focus("#native-submit");
    page.click("#plain");
    assert_eq!(submits(&page), "0");
}

#[test]
fn the_submit_carries_the_named_values() {
    let mut page = filled();
    type_text(&mut page, "#loose", "xy");
    page.click("#native-submit");
    assert_eq!(
        POSTED.with_borrow(Clone::clone),
        r#"email=Text("a@b") loose=Text("xy") tick=Text("on")"#
    );
}

#[test]
fn the_handle_submits_a_valid_form() {
    let mut page = filled();
    page.click("#submit");
    assert_eq!(submits(&page), "1");
}

#[test]
fn an_error_blocks_the_submit_and_shows_the_summary() {
    let mut page = mount(app);
    page.click("#native-submit");
    page.click("#submit");
    assert_eq!(submits(&page), "0");
    assert!(page.exists("[data-slot=summary]"), "{}", page.tree());
}

#[test]
fn reset_empties_bound_and_unbound_fields() {
    let mut page = filled();
    type_text(&mut page, "#loose", "xyz");
    page.click("#reset");
    assert_eq!(page.text("#email-value"), "");
    let doc = page.doc.inner.borrow();
    let typed = |selector: &str| {
        doc.get_node(page.node(selector))
            .and_then(|node| node.element_data())
            .and_then(|data| data.text_input_data())
            .map(|input| input.editor.raw_text().to_string())
    };
    assert_eq!(typed("#email").as_deref(), Some(""));
    assert_eq!(typed("#loose").as_deref(), Some(""));
}
