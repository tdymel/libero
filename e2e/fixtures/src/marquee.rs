//! `Marquee`.

use dioxus::prelude::*;
use libero::components::Marquee;
use libero::sx::sx;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/marquee", || rsx! { MarqueePage {} }),
    ("/marquee-vertical", || rsx! { VerticalPage {} }),
    ("/marquee-rtl", || rsx! { RtlPage {} }),
];

/// The ticker right to left (todo 2401).
#[component]
fn RtlPage() -> Element {
    rsx! {
        div { dir: "rtl", max_width: "420px",
            Marquee {
                for n in 1..=16 {
                    a { href: "#link-{n}", "L{n}" }
                }
            }
        }
    }
}

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

/// The same links stacked, in a strip shorter than one copy.
#[component]
fn VerticalPage() -> Element {
    rsx! {
        Marquee { orientation: "vertical", sx: sx().height("160px"),
            for n in 1..=16 {
                a { href: "#link-{n}", "L{n}" }
            }
        }
    }
}
