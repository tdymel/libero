//! The docs shell's star pill and heading focus, from the docs' own files.

use dioxus::prelude::*;
use libero::{components::Button, hooks::use_element};

#[path = "../../../docs/src/github_stars.rs"]
mod github_stars;
#[path = "../../../docs/src/heading_focus.rs"]
mod heading_focus;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/docs-shell/stars", || rsx! { StarsPage {} }),
    ("/docs-shell/heading", || rsx! { HeadingPage {} }),
];

/// The link mounts on a press, so a test can mock `fetch` before it asks.
#[component]
fn StarsPage() -> Element {
    let mut mounted = use_signal(|| false);
    rsx! {
        Button { id: "mount", onclick: move |_| mounted.toggle(), "Mount" }
        if mounted() {
            github_stars::GitHubLink { to: "https://github.com/example/repo",
                span { display: "inline-flex", width: "18px", height: "18px", "G" }
            }
        }
    }
}

/// Two "pages" behind one signal: the hook only needs a value that changes.
#[component]
fn HeadingPage() -> Element {
    let mut page = use_signal(|| "A");
    let content = use_element();
    heading_focus::use_heading_focus(page(), content);
    rsx! {
        nav { aria_label: "Pages",
            Button { id: "to-a", onclick: move |_| page.set("A"), "Page A" }
            Button { id: "to-b", onclick: move |_| page.set("B"), "Page B" }
        }
        div { onmounted: content.mount(),
            main {
                h1 { tabindex: "-1", "Page {page}" }
            }
        }
    }
}
