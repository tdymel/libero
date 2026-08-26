//! Reads Blitz's computed styles headlessly - no window, no clicking - so a
//! "does Blitz do X?" question can be measured instead of argued about. Built
//! to catch the stale painted transform (see
//! `writeups/blitz-stale-painted-transform.md`); left in as the harness to
//! extend for the next one.
//!
//! cargo run -p docs --example native_style_probe --features native-vello

use blitz_traits::shell::{ColorScheme, Viewport};
use dioxus::prelude::*;
use dioxus_native::{DioxusDocument, DocumentConfig};
use libero::{LiberoProvider, components::Switch};

/// Raw transforms, no libero involved - which forms does Blitz keep?
#[component]
fn Raw() -> Element {
    rsx! {
        div { id: "literal", style: "transform: translate(10px, -50%)" }
        div { id: "calc", style: "transform: translate(calc(1 * 10px), -50%)" }
        div { id: "var", style: "--x: 1; transform: translate(calc(var(--x) * 10px), -50%)" }
        div { id: "var-nested", style: "--x: 1; --w: 40px; transform: translate(calc(var(--x) * ((var(--w) - 10px - 2 * 2px))), -50%)" }
    }
}

#[component]
fn On() -> Element {
    rsx! { LiberoProvider { Switch { checked: true, onchange: move |_| {} } } }
}

#[component]
fn Off() -> Element {
    rsx! { LiberoProvider { Switch { checked: false, onchange: move |_| {} } } }
}

fn dump(label: &str, app: fn() -> Element) {
    let mut doc = DioxusDocument::new(
        VirtualDom::new(app),
        DocumentConfig {
            viewport: Some(Viewport::new(800, 600, 1.0, ColorScheme::Light)),
            // `dioxus_native::launch` sets this; without it every
            // `dangerous_inner_html` is silently dropped and libero's
            // stylesheets arrive empty.
            html_parser_provider: Some(std::sync::Arc::new(blitz_html::HtmlProvider)),
            ..Default::default()
        },
    );
    doc.initial_build();
    for _ in 0..3 {
        blitz_dom::Document::poll(&mut doc, None);
        doc.inner.borrow_mut().resolve(0.0);
    }

    println!("\n=== {label} ===");
    doc.inner.borrow().visit(|_id, node| {
        let Some(element) = node.element_data() else {
            return;
        };
        let Some(styles) = node.primary_styles() else {
            return;
        };
        let transform = &styles.get_box().transform;
        let id = element
            .attrs
            .iter()
            .find(|a| *a.name.local == *"id")
            .map(|a| a.value.clone())
            .unwrap_or_default();
        let name = element.name.local.to_string();
        if name == "style" {
            let text = node.text_content();
            println!("<style> {} bytes: {}", text.len(), &text.chars().take(160).collect::<String>());
            return;
        }
        if name == "head" || name == "html" {
            return;
        }
        println!(
            "<{name}{}> position: {:?} transform: {transform:?}",
            if id.is_empty() { String::new() } else { format!(" #{id}") },
            styles.get_box().position,
        );
    });
}

static TOGGLE: GlobalSignal<bool> = Global::new(|| false);

#[component]
fn Live() -> Element {
    rsx! {
        LiberoProvider {
            Switch { checked: TOGGLE(), onchange: move |v| *TOGGLE.write() = v }
        }
    }
}

/// The question the static dumps can't answer: when `checked` flips at
/// runtime, does the thumb - a descendant inheriting `--lsx-switch-on` from
/// the root's inline style - get restyled?
fn live() {
    let mut doc = DioxusDocument::new(
        VirtualDom::new(Live),
        DocumentConfig {
            viewport: Some(Viewport::new(800, 600, 1.0, ColorScheme::Light)),
            html_parser_provider: Some(std::sync::Arc::new(blitz_html::HtmlProvider)),
            ..Default::default()
        },
    );
    doc.initial_build();
    blitz_dom::Document::poll(&mut doc, None);
    doc.inner.borrow_mut().resolve(0.0);
    println!("\n=== live toggle ===");
    println!("before:        {}", thumb(&doc));

    doc.vdom.in_runtime(|| *TOGGLE.write() = true);
    blitz_dom::Document::poll(&mut doc, None);
    doc.inner.borrow_mut().resolve(0.0);
    println!("after (t=0):   {}", thumb(&doc));

    // If a `transition` is mid-flight, advancing the animation clock finishes it.
    for time in [0.05, 0.1, 0.2, 1.0] {
        doc.inner.borrow_mut().resolve(time);
        println!("after (t={time}): {}", thumb(&doc));
    }
}

/// The innermost `<span>` - the thumb.
fn thumb(doc: &DioxusDocument) -> String {
    let mut last = String::from("<not found>");
    doc.inner.borrow().visit(|_id, node| {
        let Some(element) = node.element_data() else {
            return;
        };
        if element.name.local.to_string() != "span" {
            return;
        }
        if let Some(styles) = node.primary_styles() {
            // Computed style vs the cached Affine the painter actually uses.
            last = format!(
                "computed {:?} | painted {:?}",
                styles.get_box().transform,
                node.transform()
            );
        }
    });
    last
}

fn main() {
    dump("raw transforms", Raw);
    dump("checked", On);
    dump("unchecked", Off);
    live();
}
