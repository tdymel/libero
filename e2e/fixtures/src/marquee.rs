//! `Marquee`.

use dioxus::prelude::*;
use libero::components::Marquee;
use libero::sx::sx;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/marquee", || rsx! { MarqueePage {} }),
    ("/marquee-vertical", || rsx! { VerticalPage {} }),
    ("/marquee-rtl", || rsx! { RtlPage {} }),
    ("/marquee-short", || rsx! { ShortPage {} }),
    ("/marquee-fade", || rsx! { FadePage {} }),
    ("/marquee-pair", || rsx! { PairPage {} }),
];

/// A ticker with faded edges, whose fade must leave a focused link alone (todo 2857).
#[component]
fn FadePage() -> Element {
    rsx! {
        div { max_width: "420px",
            Marquee { fade_edges: true,
                for n in 1..=16 {
                    a { href: "#link-{n}", "L{n}" }
                }
            }
        }
    }
}

/// A filling marquee, then a short one: the short one warning proves the filling one
/// was measured before it, in the same batch (todo 2859).
#[component]
fn PairPage() -> Element {
    rsx! {
        div { max_width: "420px",
            Marquee {
                for n in 1..=16 {
                    a { href: "#link-{n}", "L{n}" }
                }
            }
            Marquee { repeat: 2,
                span { "Rust" }
                span { "Dioxus" }
                span { "Libero" }
            }
        }
    }
}

/// Two short copies in a box wider than both: a blank strip opens (todo 2427).
#[component]
fn ShortPage() -> Element {
    rsx! {
        Marquee { repeat: 2,
            span { "Rust" }
            span { "Dioxus" }
            span { "Libero" }
        }
    }
}

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
