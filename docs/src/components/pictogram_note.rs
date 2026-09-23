use dioxus::prelude::*;
use libero::components::{Anchor, Text};

pub const PICTOGRAM_REPO: &str = "https://github.com/tdymel/pictogram";

/// The pictogram promo at the top of the Icon, ActionIcon, Pictogram and IconProvider pages.
#[component]
pub fn PictogramNote() -> Element {
    rsx! {
        Text { size: "sm", color: "muted",
            "Need icons? "
            Anchor { to: PICTOGRAM_REPO, target: "_blank", "pictogram" }
            " ships lucide, Tabler, Material and more."
        }
    }
}
