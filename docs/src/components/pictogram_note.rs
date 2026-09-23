use dioxus::prelude::*;
use libero::components::{Alert, Anchor, Code, Text};

pub const PICTOGRAM_REPO: &str = "https://github.com/tdymel/pictogram";
pub const PICTOGRAM_CRATE: &str = "https://crates.io/crates/pictogram";

/// Where the glyphs come from, at the top of the Icon, ActionIcon, Pictogram and IconProvider pages.
#[component]
pub fn PictogramNote() -> Element {
    rsx! {
        Alert { color: "info", variant: "tonal", title: "Icons from pictogram",
            Text {
                Anchor { to: PICTOGRAM_REPO, target: "_blank", "pictogram" }
                " ("
                Anchor { to: PICTOGRAM_CRATE, target: "_blank", "crates.io" }
                ") ships lucide, Tabler, Material and nine more icon sets as "
                Code { source: "SvgData" }
                " consts, one crate each. Use its 0.4 line, the one libero builds on: another minor is a second "
                Code { source: "SvgData" }
                " type and does not compile."
            }
        }
    }
}
