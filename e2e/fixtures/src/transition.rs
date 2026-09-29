//! `Transition`, for the enter/exit and reduced-motion checks.

use dioxus::prelude::*;
use libero::components::{Button, Flex, Text, Transition, TransitionKind};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/transition", || rsx! { ExitPage {} }),
    ("/transition-mount", || rsx! { MountPage {} }),
    ("/transition-corner", || rsx! { CornerPage {} }),
];

/// An origin-based kind: grows from scale(0) out of its bottom right corner.
#[component]
fn CornerPage() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button {
                id: "toggle",
                variant: "outlined",
                onclick: move |_| open.toggle(),
                "Toggle"
            }
            Transition { id: "panel", kind: TransitionKind::PopBottomRight, open: open(),
                Text { id: "panel-text", "Hello" }
            }
        }
    }
}

/// A passed `open`: enters and exits, and the closed content is unmounted.
#[component]
fn ExitPage() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button {
                id: "toggle",
                variant: "outlined",
                onclick: move |_| open.toggle(),
                "Toggle"
            }
            Transition { id: "panel", kind: TransitionKind::FadeUp, open: open(),
                Text { id: "panel-text", "Hello" }
            }
        }
    }
}

/// No `open`: animates in once on mount.
#[component]
fn MountPage() -> Element {
    rsx! {
        Transition { id: "panel", kind: TransitionKind::Scale,
            Text { id: "panel-text", "Hello" }
        }
    }
}
