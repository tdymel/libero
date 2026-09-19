//! `CopyButton`: the copied/failed status resets when focus leaves by Tab,
//! which Blitz moves without a `blur` (todo 926).

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::{Button, CopyButton};

const COPY: &str = "button[aria-label=Copy]";
const STATUS: &str = "[role=status]";

fn app() -> Element {
    rsx! {
        CopyButton { value: "cargo add libero" }
        Button { id: "after", "After" }
    }
}

/// Copied or failed, whichever the harness's clipboard answers.
fn answered(page: &mut Page) {
    let told = page.wait_for(|page| !page.text(STATUS).trim().is_empty());
    assert!(told, "no status after the press:\n{}", page.tree());
}

#[test]
fn tab_out_resets_the_status() {
    let mut page = mount(app);
    page.click(COPY);
    answered(&mut page);
    assert!(
        page.is_focused(COPY),
        "the press left focus on {}",
        page.focus_owner()
    );
    page.tab();
    assert!(page.is_focused("#after"));
    let reset = page.wait_for(|page| page.text(STATUS).trim().is_empty());
    assert!(reset, "Tab out kept {:?}", page.text(STATUS));
}

#[test]
fn a_click_elsewhere_resets_the_status() {
    let mut page = mount(app);
    page.click(COPY);
    answered(&mut page);
    page.click("#after");
    let reset = page.wait_for(|page| page.text(STATUS).trim().is_empty());
    assert!(reset, "a click away kept {:?}", page.text(STATUS));
}
