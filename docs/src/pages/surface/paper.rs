use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, Input, Paper, Text, Title},
    sx::sx,
    theme::Size,
    use_theme,
};

/// A surface has no size of its own, so the preview needs contents. They are
/// fixed markup rather than a control - there is nothing here to vary.
const CONTENT: &str = r#"Title { size: "md", component: "h2", "Invoice #4021" }
Text { "Due 30 September." }"#;

/// Padding is the caller's `sx`, deliberately: `Paper` is the surface, not the
/// box model. The shadow control rewrites this line when it is turned off.
const PADDING: &str = r#"sx: sx().padding("lg")"#;

const NO_SHADOW: &str = r#"sx: sx().padding("lg").box_shadow("none")"#;

#[component]
pub fn PaperPage() -> Element {
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "Paper",
            source: "libero/src/components/surface/paper.rs",
            markdown: "/md/paper.md",
            properties: vec![
                props("Paper", vec![
                    prop("radius", "Size")
                        .default(theme.paper.radius.as_str())
                        .doc("Corner radius, a step on the shared radius scale."),
                    prop("shadow", "Size")
                        .default(theme.paper.shadow.as_str())
                        .doc("Elevation, a step on the shared shadow scale. A flat surface is sx: sx().box_shadow(\"none\")."),
                    prop("bordered", "bool")
                        .default("false")
                        .doc("A hairline border in the themed surface border colour. Legal together with a shadow."),
                    prop("component", "HtmlTag")
                        .default("div")
                        .doc("Which element to render as - div, section, article, aside, or a for a clickable card. section and aside are landmarks, and the caller owns the aria-label that names them."),
                    prop("variables", "Variables")
                        .doc("Per-instance CSS custom properties, for a component built on Paper."),
                    prop("framework_sx", "Option<&'static StaticSx>")
                        .doc("Base styles for a component built on Paper, on the framework layer. It replaces Paper's own base, so build it from paper_sx()."),
                    prop("children", "Element").doc("The surface's contents."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A surface: a background, a corner radius, an elevation and an optional "
                    "hairline border, on a "
                    Code { source: "div" }
                    " by default. It is what every card, panel and popup in the library sits "
                    "on - "
                    Code { source: "Dialog" }
                    " is a "
                    Code { source: "Paper" }
                    " with a role. It renders no ARIA of its own, because a surface is "
                    "presentational; a "
                    Code { source: "Paper" }
                    " rendered as a "
                    Code { source: "section" }
                    " or an "
                    Code { source: "aside" }
                    " is a landmark, and the caller owns the "
                    Code { source: "aria-label" }
                    " that names it. Padding is yours, through "
                    Code { source: "sx" }
                    "."
                }
            },
            Demo {
                component: "Paper",
                children_text: "",
                children_code: CONTENT.to_string(),
                controls: vec![
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default(theme.paper.radius.as_str()),
                    // `none` is not a step on the scale - the prop is a
                    // `Size`, so switching the shadow off is an `sx` override,
                    // and this control prints the line that does it.
                    Control::slider("shadow", ["none", "xs", "sm", "md", "lg", "xl", "xxl"])
                        .default(theme.paper.shadow.as_str())
                        .code(|control, values| match values.str("shadow").as_str() {
                            "none" => vec![NO_SHADOW.to_string()],
                            // At the themed default the prop would not be
                            // typed, so it does not print - but the padding
                            // still has to.
                            shadow if shadow == control.default => vec![PADDING.to_string()],
                            shadow => {
                                vec![format!("shadow: {shadow:?}"), PADDING.to_string()]
                            }
                        }),
                    Control::switch("bordered"),
                    Control::toggle("component", ["div", "section", "article"]),
                ],
                render: move |values: DemoValues| {
                    // `none` is not a `Size`, so the prop goes unset and the
                    // `sx` override does the work - exactly what the code
                    // block prints.
                    let flat = values.str("shadow") == "none";
                    let sx = match flat {
                        true => sx().padding("lg").box_shadow("none"),
                        false => sx().padding("lg"),
                    };
                    let shadow: Input<Size> = match flat {
                        true => Input::None,
                        false => values.str("shadow").into(),
                    };

                    rsx! {
                        Paper {
                            radius: values.str("radius"),
                            shadow,
                            bordered: values.str("bordered") == "true",
                            component: values.str("component"),
                            sx,
                            // `md` is an h4 by size, and the preview sits
                            // straight under the page's h1 - the level is a
                            // document decision, the size a design one
                            // ([[codebase/heading-order]]).
                            Title { size: "md", component: "h2", "Invoice #4021" }
                            Text { "Due 30 September." }
                        }
                    }
                },
            }
        }
    }
}
