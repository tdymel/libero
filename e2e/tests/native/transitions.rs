//! CSS transitions run natively, but Blitz sends no `transitionend`: an exit
//! ends on `use_presence`'s own timer (todo 468 N6).

use std::time::Duration;

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::Collapse;

fn collapse_app() -> Element {
    let mut open = use_signal(|| true);
    rsx! {
        button { id: "toggle", onclick: move |_| open.toggle(), "Toggle" }
        Collapse { open: open(), keep_mounted: false, div { id: "content", "Details" } }
    }
}

#[test]
fn a_closed_collapse_unmounts_its_content_on_the_exit_timer() {
    let mut page = mount(collapse_app);
    assert!(page.exists("#content"));

    page.click("#toggle");
    page.advance(2.0);
    page.wait(Duration::from_secs(1));
    assert!(
        !page.exists("#content"),
        "the closed content is still mounted:\n{}",
        page.tree()
    );
}

static ENDED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn fading_app() -> Element {
    let mut faded = use_signal(|| false);
    rsx! {
        button { id: "toggle", onclick: move |_| faded.toggle(), "Toggle" }
        div {
            id: "box",
            style: if faded() { "opacity: 0; transition: opacity 0.1s" } else { "opacity: 1; transition: opacity 0.1s" },
            ontransitionend: |_| {
                ENDED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            },
            "Box"
        }
    }
}

#[test]
#[ignore = "needs Blitz: it runs the transition but dispatches no transitionend"]
fn a_finished_transition_fires_transitionend() {
    let mut page = mount(fading_app);
    page.click("#toggle");
    for _ in 0..5 {
        page.advance(0.05);
    }
    assert_eq!(page.computed("#box", "opacity"), "0");
    assert_eq!(ENDED.load(std::sync::atomic::Ordering::SeqCst), 1);
}
