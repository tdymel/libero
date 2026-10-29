use crate::components::{
    Control, Demo, DemoValues, DocPage, a11y, gradient_controls, gradient_value, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Code, Highlight, Input, Mark, Text},
    sx::ThemeAwareValue,
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

fn gradient_on(values: &DemoValues) -> bool {
    values.str("gradient") == "true"
}

fn no_gradient(values: &DemoValues) -> bool {
    !gradient_on(values)
}

/// Under a gradient `color` is its first stop, so the selected swatch is always passed and printed.
fn color_code(control: &Control, values: &DemoValues) -> Vec<String> {
    let color = values.str(control.name);
    match gradient_on(values) || color != control.default {
        true => vec![format!("color: {color:?}")],
        false => vec![],
    }
}

fn color_of(values: &DemoValues, default: &str) -> Input<ThemeAwareValue> {
    match values.str("color") {
        color if color == default && !gradient_on(values) => Input::None,
        color => Input::from(color),
    }
}

/// The color, gradient switch and gradient stops both demos share.
fn style_controls(default_color: &str) -> Vec<Control> {
    let [gradient_to, gradient_deg] = gradient_controls(no_gradient);
    vec![
        Control::color("color")
            .default(default_color)
            .code(color_code),
        // The theme's own second stop and angle print as `Gradient::default()`.
        Control::switch("gradient").code(|_, values| {
            let default =
                values.str("gradient_to") == "secondary" && values.str("gradient_deg") == "45";
            match gradient_on(values) && default {
                true => vec!["gradient: Gradient::default()".to_string()],
                false => vec![],
            }
        }),
        // Every palette colour reads into `error`; into `secondary`, three fall to 4.24:1 (4558).
        gradient_to.default("error"),
        gradient_deg,
    ]
}

#[component]
pub fn MarkPage() -> Element {
    let theme = use_theme();
    let default_color = theme.mark.color.as_str();

    rsx! {
        DocPage {
            title: "Mark",
            source: "libero/src/components/typography/mark.rs",
            markdown: "/md/mark.md",
            properties: vec![props("Mark", vec![
                prop("color", "ThemeAwareValue")
                    .default("warning, tinted")
                    .doc("A theme color name gets a light shade, and an explicit shade such as `error.4` stays as it is. Any CSS color works too: a hex turns the text black or white to read on it, any other sets only the background, so set the text color in `sx`. Under a gradient, its first stop; unset, the gradient starts at the theme gradient's first stop, not at `warning`."),
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
                    .doc("Every match of any term is a `Mark`, compared lowercase character by character (no full case folding: `istanbul` does not match `İstanbul`). Where terms overlap, the longest wins; touching matches join into one. Blank terms match nothing."),
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
                    "Check a hand-picked pair of theme stops: in the default light theme, `success` into `secondary` reads only 4.24:1, and a debug build warns in the console. There, any theme color into `error`, the demo's default, reads.",
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
                controls: style_controls(default_color),
                render: move |values: DemoValues| {
                    let color = color_of(&values, theme.mark.color.as_str());
                    let gradient = gradient_on(&values).then(|| gradient_value(&values));
                    rsx! { Mark { color, gradient, "this chunk" } }
                },
            }
            Demo {
                component: "Highlight",
                children_text: "",
                fixed: vec![format!("text: {HIGHLIGHT_TEXT:?}")],
                controls: [
                    vec![
                        Control::toggle("terms", ["rust", "rust, dioxus", "component"])
                            .labels(["Rust", "Rust, Dioxus", "Component"])
                            .code(|_, values| {
                                let terms: Vec<String> = terms(values)
                                    .iter()
                                    .map(|term| format!("{term:?}.to_string()"))
                                    .collect();
                                vec![format!("terms: vec![{}]", terms.join(", "))]
                            }),
                    ],
                    style_controls(default_color),
                ]
                .concat(),
                render: move |values: DemoValues| {
                    let color = color_of(&values, theme.mark.color.as_str());
                    let gradient = gradient_on(&values).then(|| gradient_value(&values));
                    rsx! {
                        Highlight { text: HIGHLIGHT_TEXT, terms: terms(&values), color, gradient }
                    }
                },
            }
        }
    }
}
