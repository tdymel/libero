//! Portal registration across an unmount, which shifts `use_portal`'s index hints.
//! Content changes every pass, so a write through a stale hint shows.

use std::cell::Cell;

use crate::{LiberoProvider, hooks::use_portal};
use dioxus::dioxus_core::{NoOpMutations, ScopeId, VirtualDom};
use dioxus::prelude::*;

thread_local! {
    static SHOW_MIDDLE: Cell<bool> = const { Cell::new(true) };
    /// Bumped per render, so every pass writes content no other pass wrote.
    static PASS: Cell<u32> = const { Cell::new(0) };
}

/// One bare portal entry: `use_portal` itself is under test.
#[component]
fn Portalled(content: String) -> Element {
    use_portal(Some(rsx! { "{content}" }));

    rsx! {}
}

fn app() -> Element {
    let phase = PASS.with(|pass| {
        pass.set(pass.get() + 1);
        pass.get()
    });

    rsx! {
        LiberoProvider {
            Portalled { content: "first-{phase}" }
            if SHOW_MIDDLE.with(Cell::get) {
                Portalled { content: "middle-{phase}" }
            }
            // Two after the removable one: with one, a stale hint lands past the
            // end and the fallback scan hides the bug.
            Portalled { content: "third-{phase}" }
            Portalled { content: "last-{phase}" }
        }
    }
}

#[test]
fn a_portal_still_owns_its_entry_after_an_earlier_one_unmounts() {
    SHOW_MIDDLE.with(|show| show.set(true));
    PASS.with(|pass| pass.set(0));
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();

    let html = dioxus_ssr::render(&dom);
    for content in ["first-1", "middle-1", "third-1", "last-1"] {
        assert!(html.contains(content), "{content} is missing:\n{html}");
    }

    SHOW_MIDDLE.with(|show| show.set(false));
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate(&mut NoOpMutations);
    // The render after the unmount is where a stale hint bites.
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate(&mut NoOpMutations);

    let html = dioxus_ssr::render(&dom);
    assert!(
        !html.contains("middle"),
        "the unmounted portal still renders"
    );
    for content in ["first-3", "third-3", "last-3"] {
        assert!(html.contains(content), "{content} is missing:\n{html}");
    }
    for stale in [
        "first-1", "first-2", "third-1", "third-2", "last-1", "last-2",
    ] {
        assert!(
            !html.contains(stale),
            "a portal wrote through a stale index hint, leaving {stale}:\n{html}"
        );
    }
}
