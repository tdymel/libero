use crate::components::{
    Control, Demo, DemoValues, DocPage, a11y, gradient_controls, gradient_value, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Code, Highlight, Input, Mark, Text},
    use_theme,
};

const HIGHLIGHT_TEXT: &str = "Rust components for Dioxus, written in Rust";

fn terms(values: &DemoValues) -> Vec<String> {
    values
        .str("terms")
        .split(", ")
        .map(str::to_string)
        .collect()
}

fn no_gradient(values: &DemoValues) -> bool {
    values.str("gradient") != "true"
}

#[component]
pub fn MarkPage() -> Element {
    let theme = use_theme();
    let [gradient_to, gradient_deg] = gradient_controls(no_gradient);

    rsx! {
        DocPage {
            title: "Mark",
            source: "libero/src/components/typography/mark.rs",
            markdown: "/md/mark.md",
            properties: vec![props("Mark", vec![
                prop("color", "ThemeAwareValue")
                    .default("warning, tinted")
                    .doc("A theme color name gets a light shade, and an explicit shade such as `error.4` stays as it is. Any CSS color works too. Under a gradient, its first stop."),
                prop("gradient", "Gradient")
                    .doc("Fills the highlight with a gradient from `color` to a second stop, as `(\"info\", 90)` or `Gradient::default().to(\"info\").deg(90)`; `Gradient::default()` is the theme's. The text turns black or white, whichever reads on both stops and the span between; the contrast of a literal CSS stop is yours to check. Solid in its first stop where a renderer drops the image; in forced colors, the system highlight colors."),
                prop("children", "Element").default("required").doc("The highlighted content."),
            ]),
            props("Highlight", vec![
                prop("text", "String")
                    .default("required")
                    .doc("The plain text searched, in a `<span>`."),
                prop("terms", "Vec<String>")
                    .default("[]")
                    .doc("Every match of any term, whatever its case, is a `Mark`. Where terms overlap, the longest wins; touching matches join into one. Blank terms match nothing."),
                prop("color", "ThemeAwareValue")
                    .default("warning, tinted")
                    .doc("Each match's `color`, as on `Mark`."),
                prop("gradient", "Gradient").doc("Each match's `gradient`, as on `Mark`."),
            ])],
            accessibility: a11y()
                .handles([
                    "Each highlight is a real `<mark>`.",
                    "For a theme color, a shade or a hex, the text takes the tint's contrast color, so it stays readable.",
                    "A link inside is underlined in the text's color, unless its `underline` is `never`, and its focus ring clears 3:1 against the tint.",
                    "In forced colors the tint or gradient gives way to the system highlight colors, `Mark` and `MarkText`.",
                    "Under a `gradient`, the text is black or white, picked to read at 4.5:1 on both stops and the span between in light and dark.",
                ])
                .must([
                    "With a CSS color name such as `gold`, the text keeps the page's color: check its contrast.",
                    "Check the contrast of a literal CSS stop in `gradient`.",
                    "Say in the text why a highlight matters. Not every screen reader announces `<mark>`.",
                ])
                .example("A search result with the query highlighted, `Mark { \"libero\" }`: the text stays readable on the tint, and the result says \"1 match\" in words, since not every screen reader announces the highlight."),
            lead: rsx! {
                Text {
                    "Highlights "
                    Mark { "a chunk" }
                    " of text in a real "
                    Code { source: "<mark>" }
                    ", tinted with a light shade of the theme's "
                    Code { source: "warning" }
                    " color by default. The text color follows the tint, so it stays readable. "
                    Code { source: "Highlight" }
                    " marks every match of its search terms in a text."
                }
            },
            Demo {
                component: "Mark",
                children_text: "this chunk",
                controls: vec![
                    Control::color("color")
                        .default(theme.mark.color.as_str()),
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
                ],
                render: move |values: DemoValues| rsx! {
                    Mark {
                        // The default prints no `color`, so a gradient starts at the theme's stop.
                        color: match values.str("color") {
                            color if color == theme.mark.color.as_str() => Input::None,
                            color => Input::from(color),
                        },
                        gradient: (values.str("gradient") == "true").then(|| gradient_value(&values)),
                        "this chunk"
                    }
                },
            }
            Demo {
                component: "Highlight",
                title: "Highlight",
                children_text: "",
                fixed: vec![format!("text: {HIGHLIGHT_TEXT:?}")],
                controls: vec![
                    Control::select("terms", ["rust", "rust, dioxus", "component"]).code(|_, values| {
                        let terms: Vec<String> = terms(values)
                            .iter()
                            .map(|term| format!("{term:?}.to_string()"))
                            .collect();
                        vec![format!("terms: vec![{}]", terms.join(", "))]
                    }),
                    Control::color("color")
                        .default(theme.mark.color.as_str()),
                ],
                render: move |values: DemoValues| rsx! {
                    Highlight {
                        text: HIGHLIGHT_TEXT,
                        terms: terms(&values),
                        color: match values.str("color") {
                            color if color == theme.mark.color.as_str() => Input::None,
                            color => Input::from(color),
                        },
                    }
                },
            }
        }
    }
}
