//! The `use_id` docs demo's Disclosure: natively `hidden: !open()` wrote
//! `hidden="false"`, so the panel never showed (todo 943).

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::{
    components::{Box, Button, Flex, Text},
    hooks::use_id,
};

// As on `docs/src/pages/hooks/use_id.rs`, plus test ids.
#[component]
fn Disclosure(title: String, children: Element) -> Element {
    let panel = use_id();
    let mut open = use_signal(|| false);

    rsx! {
        Button {
            variant: "standard",
            aria_expanded: open(),
            aria_controls: panel(),
            onclick: move |_| open.toggle(),
            "{title}"
        }
        Box { id: panel(), hidden: !open(), {children} }
    }
}

fn app() -> Element {
    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "xs",
            Disclosure { title: "Shipping", Text { id: "shipping", "Two to four working days." } }
            Disclosure { title: "Returns", Text { id: "returns", "Free within 30 days." } }
        }
    }
}

/// A child of a `display: none` box keeps its old layout: ask the box itself.
fn shown(page: &Page, selector: &str) -> bool {
    page.computed(selector, "display") != "none"
}

fn panel(page: &Page, button: &str) -> String {
    format!("#{}", page.attr(button, "aria-controls").unwrap())
}

#[test]
fn a_disclosure_shows_and_hides_its_panel() {
    const FIRST: &str = "button[aria-controls]";
    const SECOND: &str = "button[aria-controls] ~ button[aria-controls]";
    let mut page = mount(app);
    let (first, second) = (panel(&page, FIRST), panel(&page, SECOND));
    assert!(!shown(&page, &first), "{}", page.tree());

    page.click(FIRST);
    assert!(shown(&page, &first), "{}", page.tree());
    assert!(page.rect("#shipping").3 > 0.0);
    assert!(!shown(&page, &second), "the other panel opened too");
    assert_eq!(page.attr(&first, "hidden"), None);

    page.click(FIRST);
    assert!(!shown(&page, &first), "{}", page.tree());
    page.click(FIRST);
    assert!(shown(&page, &first), "it did not open a second time");
}

fn raw() -> Element {
    let mut off = use_signal(|| true);
    rsx! {
        button { id: "toggle", onclick: move |_| off.toggle(), "Toggle" }
        input { id: "field", disabled: !off() }
        div { id: "raw", hidden: !off(), "Raw" }
    }
}

/// Any bool attribute, on a raw element too, as the web's interpreter does.
#[test]
fn a_raw_false_flag_is_dropped() {
    let mut page = mount(raw);
    assert!(shown(&page, "#raw"));
    assert_eq!(page.attr("#field", "disabled"), None);

    page.click("#toggle");
    assert!(!shown(&page, "#raw"));
    assert!(page.attr("#field", "disabled").is_some());

    page.click("#toggle");
    assert!(shown(&page, "#raw"), "{}", page.tree());
    assert_eq!(page.attr("#field", "disabled"), None);
}
