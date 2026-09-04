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
