//! `Divider` in a 320px column: a plain rule, labelled ones (each position, a long
//! label, caller names), and decorative ones in each orientation.

use dioxus::prelude::*;
use libero::components::{Divider, Flex, Text};

use crate::Routes;

pub const ROUTES: Routes = &[("/divider", || rsx! { DividerPage {} })];

#[component]
fn DividerPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Text { "Above" }
            Divider { id: "plain" }
            Divider { id: "labelled", "Or continue with" }
            Divider { id: "decorative", role: "none" }
            Divider { id: "same-role", role: "separator", "Advanced" }
            Divider { id: "named", aria_label: "Billing", "Or" }
            Text { id: "divider-heading", "Account" }
            Divider { id: "named-by", aria_labelledby: "divider-heading", "Or" }
            Divider { id: "start", label_position: "start", "Start" }
            Divider { id: "end", label_position: "end", "End" }
            Divider { id: "long", "Or continue with your work email address instead" }
            Flex { align: "center", height: "64px",
                Text { "Left" }
                Divider { id: "vertical", orientation: "vertical", "Or" }
                Text { "Right" }
                Divider { id: "decorative-vertical", orientation: "vertical", role: "none" }
                Text { "End" }
            }
        }
    }
}
