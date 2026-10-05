//! `Mark`, and the focus ring of a link on its tint.

use dioxus::prelude::*;
use libero::components::{Anchor, Flex, Mark, Text};

use crate::Routes;

pub const ROUTES: Routes = &[("/mark", || rsx! { MarkPage {} })];

/// Todo 53: a link's ring inside a `Mark` must use the tint's contrast twin, not the primary.
/// One per palette colour: the fallback cleared 3:1 on some hues only.
#[component]
fn MarkPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Text {
                Mark {
                    Anchor { id: "mark-default", to: "#", "a link inside a mark" }
                }
            }
            // Todo 1666: the at-rest underline leaves an `underline: never` Anchor alone.
            Text {
                Mark {
                    Anchor { id: "mark-never", to: "#", underline: "never", "a bare link" }
                }
            }
            // Todo 2346: a gradient keeps the system pair in forced colours, and its label 4.5:1.
            Text {
                Mark { id: "mark-gradient", gradient: ("secondary", 45), "a gradient highlight" }
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
