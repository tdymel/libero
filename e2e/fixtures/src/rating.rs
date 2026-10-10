//! `Rating`.

use dioxus::prelude::*;
use libero::components::{Fields, Flex, Form, Rating, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/rating", || rsx! { RatingPage {} }),
    ("/rating/rtl", || rsx! { RatingRtlPage {} }),
    ("/rating/form", || rsx! { RatingFormPage {} }),
];

#[derive(Clone, PartialEq, Default, Fields)]
struct Review {
    stars: f64,
}

/// Todo 1169: a bound rating in a `Form`; the read-out shows what the last submit posted.
#[component]
fn RatingFormPage() -> Element {
    let value = use_store(Review::default);
    let mut posted = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Form {
                value,
                onsubmit: move |event: FormEvent| {
                    let values: Vec<_> = event
                        .values()
                        .into_iter()
                        .map(|(name, value)| format!("{name}={value:?}"))
                        .collect();
                    posted.set(values.join(" "));
                },
                Rating { id: "review", label: "Stars", name: Review::FIELDS.stars() }
                button { id: "send", r#type: "submit", "Send" }
            }
            Text { id: "posted", "{posted}" }
        }
    }
}

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
            Rating {
                id: "tiny",
                label: "Tiny",
                size: "xs",
                value: 0.0,
                fractions: 2,
                onchange: |_| {},
            }
            Rating {
                id: "small",
                label: "Small",
                size: "sm",
                value: 0.0,
                fractions: 2,
                onchange: |_| {},
            }
            // Tall enough that a phone's vertical swipe has a page to scroll.
            div { style: "height: 2000px" }
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
