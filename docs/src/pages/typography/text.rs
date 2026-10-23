use crate::components::{
    Control, Demo, DemoValues, DocPage, UNSET, a11y, gradient_controls, gradient_value, prop, props,
};
use dioxus::prelude::*;
use libero::components::{Code, Input, Text};
use libero::sx::sx;
use libero::use_theme;

/// Long enough to fill three lines of `CUT_WIDTH`.
const TEXT: &str = "The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs. How vexingly quick daft zebras jump!";

/// Narrow enough that `TEXT` needs a cut.
const CUT_WIDTH: &str = "16rem";

fn no_gradient(values: &DemoValues) -> bool {
    values.str("gradient") != "true"
}

fn is_cut(values: &DemoValues) -> bool {
    values.str("cut") != UNSET
}

/// The `cut` choice is a `truncate` side or a `line_clamp` count.
fn clamp(values: &DemoValues) -> Option<u32> {
    values.str("cut").parse().ok()
}

/// `truncate` or `line_clamp`, and the `sx` that narrows the preview to it.
fn cut_code(values: &DemoValues) -> Vec<String> {
    let mut lines = Vec::new();
    match values.str("cut").as_str() {
        UNSET => {}
        value @ ("end" | "start") => lines.push(format!("truncate: {value:?}")),
        lines_count => lines.push(format!("line_clamp: {lines_count}")),
    }
    if is_cut(values) {
        lines.push(format!("sx: sx().max_width({CUT_WIDTH:?})"));
    }
    lines
}

#[component]
pub fn TextPage() -> Element {
    let theme = use_theme();
    let [gradient_to, gradient_deg] = gradient_controls(no_gradient);
    rsx! {
        DocPage {
            title: "Text",
            source: "libero/src/components/typography/text.rs",
            markdown: "/md/text.md",
            properties: vec![props("Text", vec![
                prop("size", "Size")
                    .default(theme.text.size.as_str())
                    .doc("Visual size, `xs` to `xxl`."),
                prop("component", "HtmlTag")
                    .default("p")
                    .doc("The element to render."),
                prop("color", "ThemeAwareValue")
                    .doc("The text color: a theme color name in its text shade, or any CSS color. Unset, the text inherits. Under a gradient, its first stop."),
                prop("gradient", "Gradient")
                    .doc("Paints the glyphs with a gradient from `color` to a second stop, as `(\"info\", 90)` or `Gradient::default().to(\"info\").deg(90)`; `Gradient::default()` is the theme's. Keep it to large display text: the contrast of a literal CSS stop is yours to check, and a debug build warns when a hex stop reads under 4.5:1 on the page background. Solid in its first stop in print and in native windows, the system text colour in forced colours."),
                prop("truncate", "Truncate")
                    .doc("Cuts the text on one line with an ellipsis at its `end` or `start`. The element becomes a block, so a `span` truncates too. Set a `title` if the cut text matters; the full text stays in the accessible name. `line_clamp` wins when both are set."),
                prop("line_clamp", "u32")
                    .doc("Cuts the text after this many lines with an ellipsis. Over `truncate`; `0` is off. On Blitz there is no line clamp: the text is cut at that many lines, without the ellipsis."),
                prop("children", "Element").default("required").doc("The text."),
            ])],
            accessibility: a11y()
                .handles([
                    "A large size is only styling, so it never makes a heading.",
                    "A long word breaks inside the text rather than overflowing a narrow column.",
                    "A cut text is still whole in the DOM, so a screen reader reads all of it.",
                ])
                .must([
                    "Use `component: \"span\"` for text inside a sentence.",
                    "For a heading, use `Title`.",
                    "Check the contrast of a literal CSS `color` on its background, 4.5:1 for body text: a theme color name takes its text shade, a literal is used as given.",
                    "Keep `gradient` to large display text and check the contrast of a literal CSS stop. A debug build warns when a hex stop reads under 4.5:1 on the page background.",
                    "Pass a text that may be cut as `title` too, so sighted users can read it on hover, or show it in full elsewhere.",
                ])
                .example("A price in a sentence, `Text { component: \"span\", size: \"lg\", \"$12\" }`: it stays part of the sentence, and its large size does not make it a heading.")
                .limits([
                    "`component` keeps the body-text look, so `a`, `strong`, `del` and the like lose their underline, weight or line. For a link, use `Anchor`.",
                    "On Blitz, `line_clamp` cuts the text at its last line without an ellipsis.",
                ]),
            lead: rsx! {
                Text {
                    "Body copy in a "
                    Code { source: "<p>" }
                    ", sized from the theme's text scale. "
                    Code { source: "component" }
                    " changes the element without changing the look. For headings, use "
                    Code { source: "Title" }
                    "."
                }
            },
            Demo {
                component: "Text",
                children_text: TEXT,
                controls: vec![
                    Control::sizes("size")
                        .default(theme.text.size.as_str()),
                    Control::toggle("component", ["p", "span", "div"])
                        .labels(["P", "Span", "Div"]),
                    // Unset inherits the page's ink, so the unset swatch is painted ink.
                    Control::color("color").with_unset().unset_swatch("ink"),
                    // The theme's own second stop and angle print as `Gradient::default()`.
                    Control::switch("gradient").code(|_, values| {
                        let default = values.str("gradient_to") == "secondary"
                            && values.str("gradient_deg") == "45";
                        match values.str("gradient").as_str() {
                            "true" if default => vec!["gradient: Gradient::default()".to_string()],
                            _ => vec![],
                        }
                    }),
                    gradient_to,
                    gradient_deg,
                    // On, so the ellipsis shows without another switch.
                    // `truncate` cuts one line at its end or start, `line_clamp` after 2 or 3 lines.
                    Control::toggle("cut", [UNSET, "end", "start", "2", "3"])
                        .labels(["Unset", "End", "Start", "2 lines", "3 lines"])
                        .default("end")
                        .code(|_, values| cut_code(values)),
                ],
                render: move |values: DemoValues| rsx! {
                    Text {
                        size: values.str("size"),
                        component: values.str("component"),
                        truncate: match values.str("cut").as_str() {
                            side @ ("end" | "start") => Input::from(side),
                            _ => Input::None,
                        },
                        line_clamp: clamp(&values),
                        sx: is_cut(&values).then(|| sx().max_width(CUT_WIDTH)),
                        color: match values.str("color").as_str() {
                            UNSET => Input::None,
                            color => Input::from(color),
                        },
                        gradient: (values.str("gradient") == "true").then(|| gradient_value(&values)),
                        {TEXT}
                    }
                },
            }
        }
    }
}
