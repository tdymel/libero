//! `FocusTrap`: a native radio group is one Tab stop, its checked radio or,
//! with none checked, its first (todo 615); a trap mounted with the document
//! focuses its first stop (todo 664).

use dioxus::prelude::*;
use libero::components::FocusTrap;
use native_tests::mount;

// Opened by a click, as a dialog's trap is.
fn app() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        button { id: "open", onclick: move |_| open.set(true), "Open" }
        if open() {
            FocusTrap {
                button { id: "first", "First" }
                input { id: "r1", r#type: "radio", name: "plan", value: "a" }
                input { id: "r2", r#type: "radio", name: "plan", value: "b", checked: true }
                input { id: "s1", r#type: "radio", name: "size", value: "s" }
                input { id: "s2", r#type: "radio", name: "size", value: "m" }
                button { id: "last", "Last" }
            }
        }
    }
}

// Mounted with the document, its `focus()` and a `when_laid_out` timer both
// re-keyed `Outlet` in one poll, and dioxus-native panicked 3 runs in 15 (664).
fn mounted_open() -> Element {
    rsx! {
        FocusTrap {
            button { id: "first", "First" }
            button { id: "last", "Last" }
        }
    }
}

#[test]
fn a_trap_mounted_with_the_document_focuses_its_first() {
    let page = mount(mounted_open);
    assert!(page.is_focused("#first"), "{}", page.focus_owner());
}

#[test]
fn a_radio_group_is_one_tab_stop() {
    let mut page = mount(app);
    page.click("#open");
    page.focus("#first");
    let mut walked = Vec::new();
    for _ in 0..4 {
        page.tab();
        walked.push(
            page.attr_of(page.focused().unwrap(), "id")
                .unwrap_or_default(),
        );
    }
    assert_eq!(walked, ["r2", "s1", "last", "first"], "{}", page.tree());
}
