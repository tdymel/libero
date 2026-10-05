//! `Repository`, with `fetch` stubbed: no call leaves the page.

use dioxus::prelude::*;
use libero::components::{Button, ButtonGroup, Flex, RepoHost, Repository};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/repository", || rsx! { MountPage {} }),
    ("/repository/stubbed", || rsx! { StubbedPage {} }),
    ("/repository/named", || rsx! { NamedPage {} }),
    ("/repository/grouped", || rsx! { GroupedPage {} }),
];

/// Seeds `example/repo`'s count for the session and blocks `fetch`; `Some(true)` once done.
fn use_seeded_count() -> Resource<bool> {
    use_resource(|| async {
        document::eval(
            "sessionStorage.setItem('libero-repo-stars:https://api.github.com/repos/example/repo', '1234'); \
             window.fetch = () => Promise.reject(new TypeError('blocked')); return true;",
        )
        .await
        .is_ok()
    })
}

/// A caller's `aria_label` beside a seeded count, and a raw `aria-label` attribute.
#[component]
fn NamedPage() -> Element {
    let seeded = use_seeded_count();
    rsx! {
        if seeded() == Some(true) {
            Repository { id: "named", repo: "example/repo", aria_label: "Example source" }
            Repository { id: "raw", repo: "example/repo", "aria-label": "Example source, opens a new tab" }
        }
    }
}

/// The buttons mount on a press, so a test can stub `fetch` before they ask.
#[component]
fn MountPage() -> Element {
    let mut mounted = use_signal(|| false);
    rsx! {
        Button { id: "mount", onclick: move |_| mounted.toggle(), "Mount" }
        if mounted() {
            Flex { direction: "row", gap: "md",
                Repository { id: "github", repo: "example/repo" }
                Repository { id: "gitlab", repo: "group/sub/repo", host: RepoHost::GitLab }
            }
        }
    }
}

/// A count seeded before mount so the baseline sees the pill on first render; `fetch` stubbed.
/// One per count colour: dimmed text on the page, the fill's contrast colour.
#[component]
fn StubbedPage() -> Element {
    let seeded = use_seeded_count();
    rsx! {
        if seeded() == Some(true) {
            Flex { direction: "row", gap: "md",
                Repository { id: "github", repo: "example/repo" }
                Repository { id: "tonal", repo: "example/repo", variant: "tonal" }
                Repository { id: "filled", repo: "example/repo", variant: "filled" }
                // Unset: the theme's gradient, as `muted`'s stops miss 4.5:1 for every label (todo 1650).
                Repository { id: "gradient", repo: "example/repo", variant: "gradient" }
            }
            // Todo 852: a light custom colour, palette and literal, on every variant.
            for color in ["warning", "#ffe066"] {
                Flex { direction: "row", gap: "md",
                    for variant in ["outlined", "tonal", "elevated", "standard", "filled"] {
                        Repository { key: "{color}-{variant}", repo: "example/repo", variant, color }
                    }
                }
            }
        }
    }
}

/// What `Repository` takes from a `ButtonGroup`: its variant, `disabled`, the column's width.
#[component]
fn GroupedPage() -> Element {
    let seeded = use_seeded_count();
    rsx! {
        if seeded() == Some(true) {
            Flex { direction: "column", gap: "md", align: "start",
                ButtonGroup { "aria-label": "Gradient", variant: "gradient",
                    Button { "Docs" }
                    Repository { id: "gradient", repo: "example/repo" }
                }
                ButtonGroup { "aria-label": "Disabled", disabled: true,
                    Button { "Docs" }
                    Repository { id: "disabled", repo: "example/repo" }
                }
                ButtonGroup { "aria-label": "Vertical", orientation: "vertical",
                    Button { id: "wide", "A much wider label" }
                    Repository { id: "vertical", repo: "example/repo" }
                }
            }
        }
    }
}
