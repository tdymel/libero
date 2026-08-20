//! Portal registration across mount and unmount.
//!
//! `use_portal` keeps a per-portal index hint into the shared entry list, so
//! it doesn't scan every entry on every render. An unmount shifts those
//! entries, so this renders three more passes after one and checks every
//! survivor still owns its own entry. The content changes every pass: a portal
//! writing through a stale hint lands in a neighbour's entry, which identical
//! content would hide.

use std::cell::Cell;

use dioxus::dioxus_core::{NoOpMutations, ScopeId, VirtualDom};
use dioxus::prelude::*;
use libero::{LiberoProvider, components::Drawer};

thread_local! {
    static SHOW_MIDDLE: Cell<bool> = const { Cell::new(true) };
    /// Bumped per render, so every pass writes content no other pass wrote.
    static PASS: Cell<u32> = const { Cell::new(0) };
}

fn app() -> Element {
    let phase = PASS.with(|pass| {
        pass.set(pass.get() + 1);
        pass.get()
    });

    rsx! {
        LiberoProvider {
            Drawer { "first-{phase}" }
            if SHOW_MIDDLE.with(Cell::get) {
                Drawer { "middle-{phase}" }
            }
            // Two portals after the removable one: with only one, every stale
            // hint lands past the end of the list and the fallback scan hides
            // the bug this test is for.
            Drawer { "third-{phase}" }
            Drawer { "last-{phase}" }
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
    // The render *after* the unmount is where a stale hint bites: the
    // survivors re-register against a list that has already shrunk.
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
