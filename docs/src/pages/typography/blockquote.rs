use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::theme::{Responsive, Size, responsive};
use libero::{
    components::{Blockquote, BlockquotePart, Code, Text},
    use_theme,
};

/// `sm` below `md`, the chosen size from it, while the `responsive` switch is on.
fn responsive_size(values: &DemoValues) -> Responsive<Size> {
    let size = Size::from(values.str("size").as_str());
    match values.str("responsive") == "true" {
        true => responsive(Size::Sm).md(size),
        false => size.into(),
    }
}

fn size_code(control: &Control, values: &DemoValues) -> Vec<String> {
    match (values.str("responsive") == "true", values.str("size")) {
        (true, size) => vec![format!(
            "size: responsive(Size::Sm).md(Size::{:?})",
            Size::from(size.as_str())
        )],
        (false, size) if size == control.default => vec![],
        (false, size) => vec![format!("size: {size:?}")],
    }
}

/// Long enough to wrap: the padding and the line height only read against two
/// or three lines.
const QUOTE: &str = "Life is like riding a bicycle. To keep your balance, you must keep moving.";
const SPEAKER: &str = "Albert Einstein";
const WORK: &str = "Letter to his son Eduard";

#[component]
pub fn BlockquotePage() -> Element {
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "Blockquote",
            source: "libero/src/components/typography/blockquote.rs",
            markdown: "/md/blockquote.md",
            properties: vec![props("Blockquote", vec![
                prop("size", "Responsive<Size>")
                    .default(theme.blockquote.size.as_str())
                    .doc("Text size, line height, padding and the accent bar's width. Or one per breakpoint, `size: responsive(Size::Sm).md(Size::Lg)`. Breakpoints follow the window, not the parent."),
                prop("color", "ThemeAwareValue")
                    .default(theme.blockquote.color.as_str())
                    .doc("The accent bar and the tint behind the quote. A theme color name gets its lightest shade. Any CSS color works too, and a CSS color or a shade from 6 up fills the quote solid, with no separate bar or tint."),
                prop("radius", "Responsive<ThemeAwareValue>")
                    .default(theme.blockquote.radius.as_str())
                    .doc("Rounds the two corners away from the accent bar, or any CSS, e.g. `radius: \"0\"`. Or one per breakpoint, `radius: responsive(Size::Xs).md(Size::Lg)`."),
                prop("attribution", "Element")
                    .doc("Who said it, shown under the quote. For any join other than a comma, pass the whole line here."),
                prop("work", "String")
                    .doc("The title of the quoted work, such as a book or a talk. Follows `attribution` after a comma."),
                prop("cite_url", "String")
                    .doc("A URL naming the source. Only machines read it, browsers do not show it."),
                prop("parts", "Parts<BlockquotePart>")
                    .doc("Styles for the inner parts in the Style API tab, under `sx`."),
                prop("children", "Element").default("required").doc("The quote."),
            ])
            .parts("BlockquotePart", vec![
                (BlockquotePart::Quote, "The tinted `<blockquote>`."),
                (BlockquotePart::Caption, "The `<figcaption>`, with `attribution` or `work`."),
                (BlockquotePart::Work, "The `<cite>` holding `work`."),
            ])],
            accessibility: a11y()
                .handles([
                    "The quote is a `<blockquote>` in a `<figure>`, and the attribution sits in a `<figcaption>` outside it, so a screen reader does not read the speaker's name as part of the quote. Without `attribution` or `work` the root is a `<div>`, so no nameless figure is announced.",
                    "`work` renders in a `<cite>`, the comma kept outside it.",
                    "The caption dims its text color, not the whole caption, so a link in `attribution` keeps its own color and focus ring.",
                    "The quote text, a link and a focus ring inside take the tint's contrast color, so they read on the tint. A link keeps its underline, as color alone does not mark it. Any other CSS color, such as `navy`, gets black or white from the browser's `contrast-color()`.",
                    "A long word breaks inside the quote and its caption rather than overflowing a narrow column.",
                ])
                .example("A quote with `attribution: rsx! { \"Ada Lovelace\" }` and `work: \"Notes\"`: a screen reader reads the quote, then \"Ada Lovelace, Notes\" as its caption, never the name as part of the quote.")
                .limits(["`cite_url` is for machines only: browsers do not show it, so link the source yourself where readers need it."]),
            lead: rsx! {
                Text {
                    "A quotation in a tinted frame with an accent bar. The attribution sits in a "
                    Code { source: "<figcaption>" }
                    " outside the "
                    Code { source: "<blockquote>" }
                    ", so a screen reader does not read the speaker's name as part of the "
                    "quote. "
                    Code { source: "size" }
                    " scales the text with the frame, on "
                    Code { source: "Text" }
                    "'s scale. The accent bar sits at the start of the line, on the right in a right-to-left writing direction."
                }
            },
            Demo {
                component: "Blockquote",
                children_text: QUOTE,
                controls: vec![
                    Control::sizes("size")
                        .default(theme.blockquote.size.as_str()).code(size_code),
                    // On, so the breakpoint switch shows without another switch: resize the window.
                    Control::switch("responsive").default("true").code(|_, _| vec![]),
                    Control::color("color")
                    .default(theme.blockquote.color.as_str()),
                    Control::sizes("radius")
                        .default(theme.blockquote.radius.as_str()),
                    // Opens on, though unset is the real default, so it prints whenever on.
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
                ],
                render: move |values: DemoValues| rsx! {
                    Blockquote {
                        size: responsive_size(&values),
                        color: values.str("color"),
                        radius: values.str("radius"),
                        attribution: (values.str("attribution") == "true")
                            .then(|| rsx! { {SPEAKER} }),
                        work: (values.str("work") == "true").then(|| WORK.to_string()),
                        {QUOTE}
                    }
                },
            }
        }
    }
}
