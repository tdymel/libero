//! Does Blitz match `:has()`, and does it re-match one when a state it
//! depends on changes? `SegmentedControl`'s focus ring is
//! `label:has(> input:focus-visible)` and never appears natively; this says
//! whether the selector never matches, or matches but is never invalidated.
//!
//! cargo run -p docs --example has_selector_probe --features native-vello

use blitz_traits::shell::{ColorScheme, Viewport};
use dioxus::prelude::*;
use dioxus_native::{DioxusDocument, DocumentConfig};

const CSS: &str = "
    .parent { color: rgb(0, 0, 255); }
    .parent:has(> .child) { color: rgb(0, 255, 0); }
    .focus-parent { color: rgb(0, 0, 255); }
    .focus-parent:has(> .child:focus) { color: rgb(255, 0, 0); }
";

#[component]
fn App() -> Element {
    rsx! {
        style { dangerous_inner_html: CSS }
        div { id: "static-parent", class: "parent", span { class: "child" } }
        div { id: "focus-parent", class: "focus-parent",
            input { id: "target", class: "child", r#type: "radio", tabindex: "0" }
        }
    }
}

fn color_of(doc: &DioxusDocument, wanted: &str) -> String {
    let mut found = String::from("<not found>");
    doc.inner.borrow().visit(|_id, node| {
        let Some(element) = node.element_data() else {
            return;
        };
        let is_target = element
            .attrs
            .iter()
            .any(|a| *a.name.local == *"id" && a.value == wanted);
        if !is_target {
            return;
        }
        if let Some(styles) = node.primary_styles() {
            found = format!("{:?}", styles.get_inherited_text().color);
        }
    });
    found
}

fn node_id_of(doc: &DioxusDocument, wanted: &str) -> Option<blitz_dom::NodeId> {
    let mut found = None;
    doc.inner.borrow().visit(|id, node| {
        let Some(element) = node.element_data() else {
            return;
        };
        if element
            .attrs
            .iter()
            .any(|a| *a.name.local == *"id" && a.value == wanted)
        {
            found = Some(id);
        }
    });
    found
}

fn settle(doc: &mut DioxusDocument) {
    for _ in 0..3 {
        blitz_dom::Document::poll(doc, None);
        doc.inner.borrow_mut().resolve(0.0);
    }
}

fn main() {
    let mut doc = DioxusDocument::new(
        VirtualDom::new(App),
        DocumentConfig {
            viewport: Some(Viewport::new(800, 600, 1.0, ColorScheme::Light)),
            html_parser_provider: Some(std::sync::Arc::new(blitz_html::HtmlProvider)),
            ..Default::default()
        },
    );
    doc.initial_build();
    settle(&mut doc);

    println!("blue = rule did not apply, green/red = it did");
    println!(
        "static  `:has(> .child)`      -> {}",
        color_of(&doc, "static-parent")
    );
    println!(
        "unfocused `:has(> :focus)`    -> {}",
        color_of(&doc, "focus-parent")
    );

    let Some(target) = node_id_of(&doc, "target") else {
        println!("no #target node");
        return;
    };
    doc.inner.borrow_mut().set_focus_to(target);
    settle(&mut doc);
    println!(
        "focused   `:has(> :focus)`    -> {}",
        color_of(&doc, "focus-parent")
    );
}
