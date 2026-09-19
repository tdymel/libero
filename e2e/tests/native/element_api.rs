//! `ElementApi` on Blitz: the attribute read (todo 615), the value write (todo
//! 590), the previous-focusable lookup (todo 589) and the caret read (todo 666).

use dioxus::prelude::*;
use e2e::native::{Key, mount};
use libero::{hooks::use_element, platform::ElementApi};

fn app() -> Element {
    let field = use_element();
    let mut out = use_signal(String::new);
    let id = |element: &dyn ElementApi| {
        element
            .attribute("id")
            .ok()
            .flatten()
            .unwrap_or_else(|| "?".into())
    };
    rsx! {
        button { id: "before", "Before" }
        div { id: "wrap", tabindex: "0",
            input { id: "field", r#type: "text", "data-kind": "pin", onmounted: field.mount() }
        }
        button {
            id: "run",
            onclick: move |_| {
                let kind = field.attribute("data-kind");
                let absent = field.attribute("data-none");
                let button = field.previous_focusable("button");
                let any = field.previous_focusable("button, [tabindex]");
                let none = field.previous_focusable("select");
                let _ = field.set_value("7");
                out.set(format!(
                    "{kind:?} {absent:?} {} {} {}",
                    button.ok().flatten().map_or("none".into(), |e| id(e.as_ref())),
                    any.ok().flatten().map_or("none".into(), |e| id(e.as_ref())),
                    none.map(|none| none.is_none()).unwrap_or(false),
                ));
            },
            "Run"
        }
        span { id: "out", "{out}" }
    }
}

/// The ancestor `#wrap` comes before the input in document order, so the
/// wider selector finds it; `#run` comes after and is never an answer.
#[test]
fn reads_an_attribute_finds_the_previous_match_and_writes_the_value() {
    let mut page = mount(app);
    page.click("#run");
    assert_eq!(
        page.text("#out"),
        r#"Ok(Some("pin")) Ok(None) before wrap true"#,
        "{}",
        page.tree()
    );
    assert_eq!(page.attr("#field", "value").as_deref(), Some("7"));
}

fn caret_app() -> Element {
    let field = use_element();
    let mut out = use_signal(String::new);
    rsx! {
        input {
            id: "caret",
            r#type: "text",
            onmounted: field.mount(),
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Enter {
                    out.set(format!("{:?}", field.selection_start()));
                }
            },
        }
        span { id: "out", "{out}" }
    }
}

/// Todo 666: the caret in UTF-16 units, as the DOM counts, not the editor's
/// bytes.
#[test]
fn reads_the_caret_of_a_text_input() {
    let mut page = mount(caret_app);
    page.focus("#caret");
    page.press(Key::Character("é".into()));
    page.press(Key::Character("b".into()));
    page.press(Key::Enter);
    assert_eq!(page.text("#out"), "Some(2)", "{}", page.tree());
    page.press(Key::ArrowLeft);
    page.press(Key::ArrowLeft);
    page.press(Key::Enter);
    assert_eq!(page.text("#out"), "Some(0)");
}
