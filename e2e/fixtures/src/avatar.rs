//! `Avatar` and `AvatarGroup`: initials in every variant, the fallback from a
//! broken picture, a decorative avatar and the `+N` chip.

use dioxus::prelude::*;
use libero::components::{Avatar, AvatarGroup, AvatarSpec, Variant};

use crate::Routes;

pub const ROUTES: Routes = &[("/avatar", || rsx! { AvatarPage {} })];

/// Not `warning` or `success`: their text roles fall short of 4.5:1 on white,
/// a declined library-wide limit (`codebase/styling/color-roles`).
const COLORS: [&str; 3] = ["primary", "secondary", "error"];

#[component]
fn AvatarPage() -> Element {
    rsx! {
        div { display: "flex", flex_direction: "column", gap: "16px",
            div { id: "variants", display: "flex", flex_wrap: "wrap", gap: "8px",
                for variant in Variant::ALL {
                    for color in COLORS {
                        Avatar {
                            name: "Ada Lovelace",
                            initials: "AL",
                            variant: variant.as_str(),
                            color: color.to_string(),
                        }
                    }
                }
            }
            div { id: "sizes", display: "flex", flex_wrap: "wrap", gap: "8px", align_items: "center",
                for size in ["xs", "sm", "md", "lg", "xl", "xxl"] {
                    Avatar { name: "Ada Lovelace", initials: "AL", size }
                }
            }
            div { display: "flex", gap: "8px", align_items: "center",
                Avatar {
                    id: "broken",
                    name: "Grace Hopper",
                    src: "/does-not-exist.png",
                    initials: "GH",
                }
                Avatar { id: "decorative", name: "Linus", initials: "LT", alt: "" }
                span { "Linus" }
            }
            AvatarGroup {
                id: "group",
                max: 3,
                people: vec![
                    AvatarSpec::from("Ada Lovelace"),
                    AvatarSpec::from("Grace Hopper"),
                    AvatarSpec::from("Radia Perlman"),
                    AvatarSpec::from("Barbara Liskov"),
                ],
            }
        }
    }
}
