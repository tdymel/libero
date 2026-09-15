//! `Mark`, and the focus ring of a link on its tint.

use dioxus::prelude::*;
use libero::components::{Anchor, Flex, Mark, Text};

use crate::Routes;

pub const ROUTES: Routes = &[("/mark", || rsx! { MarkPage {} })];

/// Todo 53, part two: a link inside a `Mark` sits on the tint, so its ring has
/// to be drawn from the tint's contrast twin, not the primary fallback.
///
/// One `Mark` per palette colour, because the fallback's contrast against a
/// tint varies with the hue: it cleared 3:1 on some and not on others.
#[component]
fn MarkPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Text {
                Mark {
                    Anchor { id: "mark-default", to: "#", "a link inside a mark" }
                }
            }
            for (id, color) in MARK_COLORS {
                Text {
                    Mark { color: *color,
                        Anchor { id: *id, to: "#", "a link inside a mark" }
                    }
                }
            }
        }
    }
}

/// The ids `tests/all/mark.rs` tabs to, with the `Mark` colour each sits on.
/// `mark-default` sits on the theme's default and is rendered separately.
const MARK_COLORS: &[(&str, &str)] = &[
    ("mark-primary", "primary"),
    ("mark-secondary", "secondary"),
    ("mark-error", "error"),
    ("mark-info", "info"),
    ("mark-success", "success"),
    // A dark shade: its white twin needs the fill as its halo (todo 630).
    ("mark-info-6", "info.6"),
];
