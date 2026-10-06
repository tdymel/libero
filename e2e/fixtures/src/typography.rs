//! `Text`, `Title`, `Blockquote`, `Mark` and inline `Code`, every colour the docs offer.
//! Two pages: contrast coverage only counts text inside the viewport.

use dioxus::prelude::*;
use libero::{
    components::{
        Anchor, Blockquote, BlockquotePart, Code, Flex, Kbd, Mark, Parts, Text, Title,
    },
    sx::sx,
    theme::Gradient,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/typography", || rsx! { TypographyPage {} }),
    ("/typography/quotes", || rsx! { QuotesPage {} }),
];

const COLORS: &[&str] = &[
    "primary",
    "secondary",
    "error",
    "info",
    "success",
    "warning",
];

const KBD_SIZES: &[&str] = &["xs", "sm", "md", "lg", "xl", "xxl"];

/// Mid and dark shades, where a brightness-picked twin or the page text failed.
const SHADES: &[&str] = &["info.6", "error.8"];

/// A 320px column, the width WCAG 1.4.10 reflows to.
#[component]
fn TypographyPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Title { size: "xxl", "Heading one" }
            Text { id: "marks",
                "Default "
                Mark { "highlight" }
                for color in COLORS {
                    " "
                    Mark { color: *color, "{color}" }
                }
                // A hex is drawn as given in both schemes, so the text has to follow it.
                " "
                Mark { color: "#1e3a8a", "navy" }
                " "
                Mark { color: "#ffe066", "yellow" }
                // Todo 605: white on info.6 was 2.78:1; page text on dark error.8, 2.36:1.
                for color in SHADES {
                    " "
                    Mark { color: *color, "{color}" }
                }
            }
            Text {
                "Call "
                Code { id: "code-rust", language: "rust", source: "let s: &str = \"x\"; // note" }
                " inline."
            }
            Text {
                "A long name "
                Code { id: "code-long", source: "a_very_long_identifier_without_any_break_opportunity_in_it" }
                " wraps."
            }
            Blockquote { id: "quote-default",
                attribution: rsx! { "Albert Einstein" },
                work: "Letter to his son Eduard",
                "Life is like riding a bicycle."
            }
            // Todo 2489: a coloured and a gradient Text through the 320px and text-spacing passes.
            Text { id: "text-color", color: "error", "Coloured text in the narrow column." }
            Text { id: "text-gradient", gradient: Gradient::default(), "Gradient text in the narrow column." }
            // Todo 2543: one word wider than the column at `xxl`.
            Title { id: "title-long", size: "xxl", "Donaudampfschifffahrtsgesellschaft" }
            // Todo 2547: a key of every size in running text.
            Text { id: "kbds",
                "Press "
                for size in KBD_SIZES {
                    Kbd { size: *size, "Ctrl" }
                    " + "
                }
                "K."
            }
            Title { size: "xl", "Heading two" }
            Title { size: "lg", "Heading three" }
            Title { size: "md", "Heading four" }
            Title { size: "sm", "Heading five" }
            Title { size: "xs", "Heading six" }
            Text { size: "xs", component: "span", "Extra small text in a span." }
        }
    }
}

/// One `Blockquote` per palette colour: tint, body text and attribution.
#[component]
fn QuotesPage() -> Element {
    rsx! {
        Flex { id: "quotes", direction: "column", gap: "xs", max_width: "320px",
            for color in COLORS {
                Blockquote { size: "xs", color: *color, attribution: rsx! { "Speaker" }, "A {color} quote." }
            }
            // A hex is drawn as given in both schemes, so the text has to follow it.
            Blockquote { size: "xs", color: "#1e3a8a", "A navy quote." }
            Blockquote { size: "xs", color: "#ffe066", "A yellow quote." }
            for color in SHADES {
                Blockquote { size: "xs", color: *color, "A {color} quote." }
            }
            // Todo 2484: `opacity` on the caption took this link to 2.4:1.
            Blockquote { id: "quote-linked", size: "xs", color: "info",
                attribution: rsx! { Anchor { id: "attribution-link", to: "#source", "Albert Einstein" } },
                "A quote with a linked attribution."
            }
            // Todo 2485: a named literal leaves the text to the caller, as the page says.
            Blockquote { id: "quote-named", size: "xs", color: "navy",
                parts: Parts::new().part(BlockquotePart::Quote, sx().color("#FFFFFF")),
                "A navy quote."
            }
        }
    }
}
