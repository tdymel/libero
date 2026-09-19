//! Inline `Code` natively. Blitz (parley) trims the whitespace an inline
//! element starts or ends with, so a space inside a token span vanished ("letwidth").

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::{Code, Text};

fn app() -> Element {
    rsx! {
        Text { id: "line",
            "Bind it with "
            Code { id: "code", source: "let width: u32 = 320;", language: "rust" }
            " before the first draw."
        }
        Text { id: "plain",
            Code { id: "bare", source: "let width: u32 = 320;" }
        }
    }
}

#[test]
fn a_highlighted_snippet_keeps_its_spaces() {
    let mut page = mount(app);
    page.settle();
    assert!(page.exists("#line code span"), "highlighted");
    assert_eq!(
        page.laid_out_text("#line"),
        "Bind it with let width: u32 = 320; before the first draw."
    );
}

#[test]
fn a_highlighted_snippet_paints_its_background() {
    let mut page = mount(app);
    page.settle();
    let background = |selector: &str| {
        let (left, top, _, height) = page.rect(selector);
        page.painted_pixel((left + 2.0) as u32, (top + height / 2.0) as u32)
    };
    assert_eq!(background("#code"), background("#bare"));
    assert_ne!(background("#code"), "rgb(255, 255, 255)");
}
