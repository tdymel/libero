use crate::common::{attributes_of, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Highlight, Mark},
    theme::Color,
};

#[test]
fn mark_highlights_with_its_background_variable() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Mark { color: Color::Warning, "highlighted" }
            }
        }
    }

    let html = render(app);

    assert!(attributes_of(&html, "mark")["style"].contains("--lsx-mark-background"));
}

/// Todo 3983: each match is its own `mark` in the term's tint; the text between stays bare.
#[test]
fn highlight_marks_each_match_of_its_terms() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Highlight { text: "Rust or rust", terms: vec!["rust".to_string()], color: "info" }
            }
        }
    }

    let html = render(app);

    assert_eq!(html.matches("<mark").count(), 2, "{html}");
    assert!(
        html.contains(">Rust</mark>") && html.contains(">rust</mark>"),
        "{html}"
    );
    assert!(html.contains(" or "), "{html}");
}
