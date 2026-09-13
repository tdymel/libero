//! `Marquee`.

use dioxus::prelude::*;
use libero::components::Marquee;

use crate::Routes;

pub const ROUTES: Routes = &[("/marquee", || rsx! { MarqueePage {} })];

/// A ticker of short links, longer than its box, so the links of the one live
/// copy pass the pause toggle on their way out.
#[component]
fn MarqueePage() -> Element {
    rsx! {
        div { max_width: "420px",
            Marquee {
                for n in 1..=16 {
                    a { href: "#link-{n}", "L{n}" }
                }
            }
        }
    }
}
