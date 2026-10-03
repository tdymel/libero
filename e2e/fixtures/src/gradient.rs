//! Gradient fills (todo 937): every component that takes one, on the theme's
//! gradient and on an override, plus glass over a gradient and gradient text.

use dioxus::prelude::*;
use libero::components::{ActionIcon, Badge, Button, Flex, Header, Icon, Mark, Paper, Text, Title};
use libero::theme::Gradient;

use crate::Routes;

pub const ROUTES: Routes = &[("/gradient", || rsx! { GradientPage {} })];

#[component]
fn GradientPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "480px",
            Flex { gap: "sm", wrap: "wrap", align: "center",
                Button { id: "button", variant: "gradient", "Theme" }
                Button { id: "override", variant: "gradient", color: "success",
                    gradient: ("info", 90),
                    "Override"
                }
                // `color` alone: the first stop, into the theme's second (1016).
                Button { id: "color-only", variant: "gradient", color: "error", "Color" }
                Button { id: "selected", variant: "gradient", selected: true, "Selected" }
                Button { id: "ignored", gradient: Gradient::default().to("info"), "Filled" }
                Button { id: "plain-standard", variant: "standard", "Plain" }
                ActionIcon { id: "action-icon", variant: "gradient", aria_label: "Star", "★" }
                Badge { id: "badge", variant: "gradient", "New" }
                Icon { id: "icon", variant: "gradient", aria_label: "Verified", "✓" }
            }
            Paper { id: "paper", gradient: Gradient::default(), sx: libero::sx::sx().padding("md"),
                Text { "On the theme's gradient." }
                // 1663: an uncoloured standard Button takes the gradient's label; a plain
                // Paper inside gives its own back.
                Button { id: "in-paper", variant: "standard", "More" }
                // 1833: so does an outlined one, its border too (1.00:1 before).
                Button { id: "in-paper-outlined", variant: "outlined", "Outlined" }
                Paper { id: "nested-plain", sx: libero::sx::sx().padding("xs"),
                    Button { id: "in-nested", variant: "standard", "Nested" }
                }
            }
            Paper { id: "glass", glass: true, gradient: Gradient::default().to("error"),
                sx: libero::sx::sx().padding("md"),
                Text { "Glass over a gradient." }
            }
            Text { id: "text", gradient: Gradient::default(), size: "xl", "Gradient text" }
            Text { id: "text-color", color: "error", "Plain coloured text" }
            // Text's gradient on Title (2132); a plain one keeps its look.
            Title { id: "title", size: "lg", gradient: Gradient::default(), "Gradient title" }
            Title { id: "title-plain", size: "lg", "Plain title" }
            // Mark's gradient is the highlight; its text takes the label (2132).
            Text { "A " Mark { id: "mark", gradient: Gradient::default(), "gradient highlight" } " and a "
                Mark { id: "mark-color", color: "error", gradient: ("info", 90), "coloured one" }
            }
            // `color` is the first stop, over glass too (1016).
            Header { id: "header", position: "static", glass: true, color: "error",
                gradient: Gradient::default(),
                Button { id: "in-header", variant: "standard", "Menu" }
                Button { id: "in-header-outlined", variant: "outlined", "Sign in" }
            }
            // Paper `color` (1017): a palette fill with its label, a literal as given, glass.
            Paper { id: "paper-color", color: "info", sx: libero::sx::sx().padding("md"),
                Text { "On an info fill." }
                button { id: "in-paper-color", style: "color: inherit; background: none; border: 0",
                    "More"
                }
            }
            // A literal's label is the caller's.
            Paper { id: "paper-literal", color: "#123456", sx: libero::sx::sx().padding("md").color("#fff"),
                Text { "On a literal fill." }
            }
            Paper { id: "paper-glass", color: "secondary", glass: true,
                sx: libero::sx::sx().padding("md"),
                Text { "A glass tint." }
            }
            // 1085: a glass header tinted by its `color`, like the Paper above.
            Header { id: "header-tint", position: "static", glass: true, color: "info", "Tinted" }
            Paper { id: "paper-gradient", color: "error", gradient: ("info", 90),
                sx: libero::sx::sx().padding("md"),
                Text { "From error to info." }
            }
        }
    }
}
