use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Code};

#[test]
fn code_renders_its_source() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Code { source: "let x = 1;" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("<code"));
    assert!(body(&html).contains("let x = 1;"));
}

/// Todo 732: a short span never breaks mid-token; a long one keeps
/// `overflow-wrap: anywhere` for a 320px column.
#[test]
fn only_a_short_code_span_is_kept_on_one_line() {
    fn short() -> Element {
        rsx! {
            LiberoProvider { Code { source: "#[derive(Options)]" } }
        }
    }
    fn long() -> Element {
        rsx! {
            LiberoProvider { Code { source: "an_identifier_far_too_long_for_one_line" } }
        }
    }

    let html = render(short);
    assert!(body(&html).contains(r#"data-state="short""#), "{html}");
    assert!(
        html.contains(r#"[data-state~="short"]{white-space:nowrap;"#),
        "{html}"
    );
    assert!(!body(&render(long)).contains("short"));
}
