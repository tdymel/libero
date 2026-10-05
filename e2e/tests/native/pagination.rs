//! `Pagination`'s focus debt under Blitz's silent Tab (1639). The browser's
//! twin is `all/pagination.rs`'s `a_rejected_press_owes_no_focus_once_focus_leaves`.

use dioxus::prelude::*;
use e2e::native::{Key, mount};
use libero::components::Pagination;

const LAST: &str = "[aria-label=\"Go to last page\"]";

// The caller rejects every press; only `#reset` moves the page.
fn rejecting() -> Element {
    let mut page = use_signal(|| 5u32);
    rsx! {
        Pagination {
            aria_label: "Results",
            total: 10,
            with_edges: true,
            page: page(),
            onchange: move |_| {},
        }
        button { id: "reset", onclick: move |_| page.set(2), "Reset" }
    }
}

#[test]
fn a_rejected_press_owes_no_focus_once_tab_leaves() {
    let mut page = mount(rejecting);
    page.focus(LAST);
    page.press(Key::Enter);
    page.tab();
    assert!(
        page.is_focused("#reset"),
        "after Tab: {}",
        page.focus_owner()
    );
    page.press(Key::Enter);
    let moved = page.wait_for(|page| page.text("[aria-current=page]") == "2");
    assert!(moved, "the reset never reached page 2");
    assert!(
        page.is_focused("#reset"),
        "the old debt pulled focus: {}",
        page.focus_owner()
    );
}
