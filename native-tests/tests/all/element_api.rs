//! `ElementApi` on Blitz: the attribute read (todo 615), the value write (todo
//! 590) and the previous-focusable lookup (todo 589).

use dioxus::prelude::*;
use libero::{hooks::use_element, platform::ElementApi};
use native_tests::mount;

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
