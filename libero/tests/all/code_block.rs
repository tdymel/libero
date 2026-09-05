use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::CodeBlock};

#[test]
fn code_block_renders_its_source_in_a_pre() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                CodeBlock { source: "let x = 1;" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("<pre"));
    assert!(body(&html).contains("let x = 1;"));
}

/// The copy confirmation is spoken through a status region that exists before
/// the copy, and a box nobody has measured as overflowing is no tab stop.
#[test]
fn code_block_mounts_an_empty_copy_status_and_no_tab_stop() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                CodeBlock { source: "let x = 1;" }
            }
        }
    }

    let html = render(app);
    let status = &html[html.find(r#"role="status""#).expect("a status region")..];
    let status = &status[..status.find("</span>").unwrap()];
    assert!(!status.contains("Copied"), "{status}");
    assert!(!html.contains("tabindex=\"0\""), "{html}");
    assert!(!html.contains(r#"role="region""#), "{html}");
}
