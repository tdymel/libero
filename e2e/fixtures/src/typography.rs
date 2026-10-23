//! `Text`, `Title`, `Blockquote`, `Mark` and inline `Code`, every colour the docs offer.
//! Two pages: contrast coverage only counts text inside the viewport.

use dioxus::prelude::*;
use libero::{
    components::{
        Anchor, Blockquote, Code, Flex, Kbd, Mark, Paper, Text, Title,
    },
    sx::sx,
    theme::Gradient,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/typography", || rsx! { TypographyPage {} }),
    ("/typography/quotes", || rsx! { QuotesPage {} }),
    ("/typography/paper", || rsx! { PaperTitlePage {} }),
    ("/typography/truncate", || rsx! { TruncatePage {} }),
];

const SENTENCE: &str = "The quick brown fox jumps over the lazy dog and keeps running";

/// Todo 3966: a 200px column of Text and Title cut at the end, at the start and after two
/// lines, each beside a one-line twin to measure a line by.
#[component]
fn TruncatePage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "200px",
            Text { id: "text-one", "Short" }
            Text { id: "text-plain", "{SENTENCE}" }
            Text { id: "text-end", truncate: "end", "{SENTENCE}" }
            Text { id: "text-start", truncate: "start", "{SENTENCE}" }
            Text { id: "text-clamp", line_clamp: 2, "{SENTENCE}" }
            Text { id: "text-clamp-wins", truncate: "start", line_clamp: 2, "{SENTENCE}" }
            Text { id: "text-clamp-off", line_clamp: 0, "{SENTENCE}" }
            Title { id: "title-one", size: "md", "Short" }
            Title { id: "title-plain", size: "md", "{SENTENCE}" }
            Title { id: "title-end", size: "md", truncate: "end", "{SENTENCE}" }
            Title { id: "title-clamp", size: "md", line_clamp: 2, "{SENTENCE}" }
        }
    }
}

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

/// Wider than a 320px column at body size.
const LONG_WORD: &str = "Donaudampfschifffahrtselektrizitaetenhauptbetriebswerkbauunterbeamtengesellschaft";

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
            // Todo 2589: the same in body text and in a quote.
            Text { span { id: "text-long", "{LONG_WORD}" } }
            Blockquote { span { id: "quote-long", "{LONG_WORD}" } }
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
            // Todo 3065: the caption breaks a long attribution and work too.
            Blockquote {
                id: "quote-caption",
                attribution: rsx! { span { id: "attribution-long", "{LONG_WORD}" } },
                work: LONG_WORD,
                "A quote."
            }
        }
    }
}

/// Todo 2985: a long word in a `Title` on a `Paper` in a 320px column.
#[component]
fn PaperTitlePage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Paper { sx: sx().padding("lg"),
                Title { id: "paper-title", size: "xl", "Donaudampfschifffahrtsgesellschaft" }
            }
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
            // Todo 2521: the link inside was 3.53:1 in the theme's link colour on the tint.
            Blockquote { id: "quote-linked", size: "xs", color: "info",
                attribution: rsx! {
                    "Said by "
                    Anchor { id: "attribution-link", to: "#source", "Albert Einstein" }
                },
                "A quote with a "
                Anchor { id: "quote-link", to: "#source", "link" }
                " and a linked attribution."
            }
            // Todo 2520: a named literal takes the browser's `contrast-color()`.
            Blockquote { id: "quote-named", size: "xs", color: "navy", "A navy quote." }
        }
    }
}
