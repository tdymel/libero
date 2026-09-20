//! Gradient fills (todo 937): every component that takes one, on the theme's
//! gradient and on an override, plus glass over a gradient and gradient text.

use dioxus::prelude::*;
use libero::components::{ActionIcon, Badge, Button, Flex, Icon, Paper, Text};
use libero::theme::Gradient;

use crate::Routes;

pub const ROUTES: Routes = &[("/gradient", || rsx! { GradientPage {} })];

#[component]
fn GradientPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "480px",
            Flex { gap: "sm", wrap: "wrap", align: "center",
                Button { id: "button", variant: "gradient", "Theme" }
                Button { id: "override", variant: "gradient",
                    gradient: Gradient::default().from("success").to("info").deg(90),
                    "Override"
                }
                Button { id: "selected", variant: "gradient", selected: true, "Selected" }
                Button { id: "ignored", gradient: Gradient::default().to("info"), "Filled" }
                ActionIcon { id: "action-icon", variant: "gradient", aria_label: "Star", "★" }
                Badge { id: "badge", variant: "gradient", "New" }
                Icon { id: "icon", variant: "gradient", aria_label: "Verified", "✓" }
            }
            Paper { id: "paper", gradient: Gradient::default(), sx: libero::sx::sx().padding("md"),
                Text { "On the theme's gradient." }
            }
            Paper { id: "glass", glass: true, gradient: Gradient::default().to("error"),
                sx: libero::sx::sx().padding("md"),
                Text { "Glass over a gradient." }
            }
            Text { id: "text", gradient: Gradient::default(), size: "xl", "Gradient text" }
        }
    }
}
