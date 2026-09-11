use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Dialog};

/// `Dialog` is a `Paper`, so the surface has to reach it - and the dialog's
/// own chrome has to survive the composition.
#[test]
fn a_dialog_renders_the_paper_surface_under_its_own_chrome() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Dialog { aria_label: "Inline", "content" }
            }
        }
    }

    let html = render(app);
    let dialog = attributes_of(&body(&html), "div");

    assert_eq!(dialog["role"], "dialog");
    assert!(
        html.contains("background:var(--lsx-paper-background);"),
        "{html}"
    );
    assert!(
        html.contains("border-radius:var(--lsx-dialog-radius, var(--lsx-paper-radius));"),
        "{html}"
    );
    assert!(html.contains("box-shadow:var(--lsx-shadow-xl);"), "{html}");
}

thread_local! {
    static TITLE: std::cell::Cell<&'static str> = const { std::cell::Cell::new("First") };
}

/// The header memoizes, so a new title has to reach it through its props.
#[test]
fn a_changed_title_redraws_the_memoized_header() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Dialog { title: TITLE.get(), close_button: true, "content" }
            }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    assert!(dioxus_ssr::render(&dom).contains("First"));

    TITLE.set("Second");
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate(&mut dioxus::dioxus_core::NoOpMutations);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("Second"), "{html}");
    assert!(!html.contains("First"), "{html}");
}
