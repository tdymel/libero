use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Blockquote, Code, Text},
    use_theme,
};

/// Long enough to wrap: the padding and the line height only read against two
/// or three lines.
const QUOTE: &str = "Life is like riding a bicycle. To keep your balance, you must keep moving.";
const SPEAKER: &str = "Albert Einstein";
const WORK: &str = "Letter to his son Eduard";
const SOURCE_URL: &str = "https://example.org/letters/1930-02-05";

#[component]
pub fn BlockquotePage() -> Element {
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "Blockquote",
            source: "libero/src/components/typography/blockquote.rs",
            markdown: "/md/blockquote.md",
            properties: vec![props("Blockquote", vec![
                prop("size", "Size")
                    .default("md")
                    .doc("Body font size, line height, padding and the accent bar's width."),
                prop("color", "ThemeAwareValue")
                    .default("primary, tinted")
                    .doc("The accent bar, and the background tint derived from it. A bare theme color is tinted to its lightest shade."),
                prop("radius", "Size")
                    .default("sm")
                    .doc("Rounds the two corners away from the accent bar."),
                prop("attribution", "Element")
                    .doc("Who said it, rendered in a `<figcaption>` outside the quote. A person's name goes here as plain text - it does not become a `<cite>`."),
                prop("work", "String")
                    .doc("The title of the work quoted, rendered as a `<cite>` in the `<figcaption>`. Not a person. Follows `attribution` after a comma when both are set."),
                prop("cite_url", "String")
                    .doc("The `cite` attribute on `<blockquote>`: a URL naming the source document. Machine-readable only, no browser renders it."),
                prop("children", "Element").doc("The quote."),
            ])],
            lead: rsx! {
                Text {
                    "A quotation with its attribution. Renders a real "
                    Code { source: "<figure>" }
                    " holding a "
                    Code { source: "<blockquote>" }
                    " and, when there is one, a "
                    Code { source: "<figcaption>" }
                    " - the attribution sits outside the quote, which is where the HTML "
                    "spec puts it, so assistive technology does not read a speaker's name as "
                    "quoted words. "
                    Code { source: "size" }
                    " scales the body text along with the frame, on the same scale Text uses. "
                    "The accent bar is on the left in every writing direction."
                }
            },
            Demo {
                component: "Blockquote",
                children_text: QUOTE,
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default(theme.blockquote.size.as_str()),
                    Control::color(
                        "color",
                        ["primary", "secondary", "success", "error", "warning", "info"],
                    )
                    .default(theme.blockquote.color.as_str()),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default(theme.blockquote.radius.as_str()),
                    // Opens on, because a quote with nobody under it is half
                    // the component - but unset is the real default, so it
                    // prints whenever it is on.
                    Control::switch("attribution").default("true").code(|_, values| {
                        match values.str("attribution").as_str() {
                            "true" => vec![format!("attribution: rsx! {{ {SPEAKER:?} }}")],
                            _ => vec![],
                        }
                    }),
                    Control::switch("work").code(|_, values| {
                        match values.str("work").as_str() {
                            "true" => vec![format!("work: {WORK:?}")],
                            _ => vec![],
                        }
                    }),
                    Control::switch("cite_url").code(|_, values| {
                        match values.str("cite_url").as_str() {
                            "true" => vec![format!("cite_url: {SOURCE_URL:?}")],
                            _ => vec![],
                        }
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    Blockquote {
                        size: values.str("size"),
                        color: values.str("color"),
                        radius: values.str("radius"),
                        attribution: (values.str("attribution") == "true")
                            .then(|| rsx! { {SPEAKER} }),
                        work: (values.str("work") == "true").then(|| WORK.to_string()),
                        cite_url: (values.str("cite_url") == "true")
                            .then(|| SOURCE_URL.to_string()),
                        {QUOTE}
                    }
                },
            }
        }
    }
}
