//! `Rating`.

use dioxus::prelude::*;
use libero::components::{Flex, Rating, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/rating", || rsx! { RatingPage {} }),
    ("/rating/rtl", || rsx! { RatingRtlPage {} }),
];

/// Halves from 0 with a hover read-out, a clearable one at 3, read-only and
/// display-only. Each read-out shows what its rating last emitted.
#[component]
fn RatingPage() -> Element {
    let mut stars = use_signal(|| 0.0f64);
    let mut hovered = use_signal(|| None::<f64>);
    let mut cleared = use_signal(|| 3.0f64);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Rating {
                id: "stars",
                label: "Stars",
                value: stars(),
                fractions: 2,
                onchange: move |value| stars.set(value),
                onhover: move |value| hovered.set(value),
            }
            Text { id: "stars-value", "{stars}" }
            Text { id: "stars-hover", {hovered().map(|value| value.to_string()).unwrap_or_default()} }
            Rating {
                id: "clear",
                label: "Clearable",
                value: cleared(),
                clearable: true,
                onchange: move |value| cleared.set(value),
            }
            Rating {
                id: "fixed",
                label: "Fixed",
                value: 2.0,
                readonly: true,
                onchange: move |_| {},
            }
            Rating { id: "average", label: "Average", value: 4.3, focusable: false }
        }
    }
}

/// One rating right to left: its first symbol is the rightmost.
#[component]
fn RatingRtlPage() -> Element {
    let mut stars = use_signal(|| 0.0f64);

    rsx! {
        div { dir: "rtl",
            Rating {
                id: "rtl",
                label: "Stars",
                value: stars(),
                onchange: move |value| stars.set(value),
            }
        }
    }
}
