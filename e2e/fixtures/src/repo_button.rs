//! `RepoButton`, with `fetch` stubbed: no call leaves the page.

use dioxus::prelude::*;
use libero::components::{Button, Flex, RepoButton, RepoHost};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/repo-button", || rsx! { MountPage {} }),
    ("/repo-button/stubbed", || rsx! { StubbedPage {} }),
];

/// The buttons mount on a press, so a test can stub `fetch` before they ask.
#[component]
fn MountPage() -> Element {
    let mut mounted = use_signal(|| false);
    rsx! {
        Button { id: "mount", onclick: move |_| mounted.toggle(), "Mount" }
        if mounted() {
            Flex { direction: "row", gap: "md",
                RepoButton { id: "github", repo: "example/repo" }
                RepoButton { id: "gitlab", repo: "group/sub/repo", host: RepoHost::GitLab }
            }
        }
    }
}

/// A count seeded before mount so the baseline sees the pill on first render; `fetch` stubbed.
/// One per count colour: dimmed text on the page, the fill's contrast colour.
#[component]
fn StubbedPage() -> Element {
    let seeded = use_resource(|| async {
        document::eval(
            "sessionStorage.setItem('libero-repo-stars:https://api.github.com/repos/example/repo', '1234'); \
             window.fetch = () => Promise.reject(new TypeError('blocked')); return true;",
        )
        .await
        .is_ok()
    });
    rsx! {
        if seeded() == Some(true) {
            Flex { direction: "row", gap: "md",
                RepoButton { id: "github", repo: "example/repo" }
                RepoButton { id: "tonal", repo: "example/repo", variant: "tonal" }
                RepoButton { id: "filled", repo: "example/repo", variant: "filled" }
            }
            // Todo 852: a light custom colour, palette and literal, on every variant.
            for color in ["warning", "#ffe066"] {
                Flex { direction: "row", gap: "md",
                    for variant in ["outlined", "tonal", "elevated", "standard", "filled"] {
                        RepoButton { key: "{color}-{variant}", repo: "example/repo", variant, color }
                    }
                }
            }
        }
    }
}
