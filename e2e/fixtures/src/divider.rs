//! `Divider`: a plain rule, a labelled one in each orientation, and one the
//! caller marks decorative.

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
            Flex { align: "center", height: "64px",
                Text { "Left" }
                Divider { id: "vertical", orientation: "vertical", "Or" }
                Text { "Right" }
            }
        }
    }
}
