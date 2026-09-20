use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Box, Code, Input, Paper, Text, Title},
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
// snippet: in Paper { .. }
const PADDING: &str = r#"sx: sx().padding("lg")"#;

// snippet: in Paper { .. }
const NO_SHADOW: &str = r#"sx: sx().padding("lg").box_shadow("none")"#;

/// Sharp stripes behind the glass, so the blur has something to show.
const STRIPES: &str =
    "repeating-linear-gradient(45deg, var(--lsx-primary-fill-6) 0 12px, transparent 12px 24px)";

/// Glass prints the striped `Box` it sits on; off, the bare `Paper`.
fn wrap_backdrop(values: &DemoValues, code: &str) -> String {
    match values.str("glass") == "true" {
        true => format!(
            "Box {{\n    sx: sx().padding(\"xl\").background({STRIPES:?}),\n{}}}",
            indent(code)
        ),
        false => code.to_string(),
    }
}

#[component]
pub fn PaperPage() -> Element {
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "Paper",
            source: "libero/src/components/layout/paper.rs",
            markdown: "/md/paper.md",
            properties: vec![
                props("Paper", vec![
                    prop("radius", "Size")
                        .default(theme.paper.radius.as_str())
                        .doc("Corner radius, a step on the shared radius scale."),
                    prop("shadow", "Size")
                        .default(theme.paper.shadow.as_str())
                        .doc("Elevation, a step on the shared shadow scale. For a flat surface use `sx().box_shadow(\"none\")`."),
                    prop("bordered", "bool")
                        .default("false")
                        .doc("A hairline border in the theme's surface border colour. Works together with a shadow."),
                    prop("glass", "bool")
                        .default("false")
                        .doc("Frosted glass: translucent, blurring what is behind it, tuned by the theme's `paper.glass_background` and `paper.glass_blur`. Use it over app chrome, not over images, where text can lose contrast. Opaque when the user reduces transparency, in forced colours, and in native windows."),
                    prop("gradient", "Gradient")
                        .doc("Fills the surface with a gradient, `Gradient::default()` for the theme's. The text colour is picked to read on both stops. With `glass`, the stops turn translucent. Its stops carry down to any gradient inside it."),
                    prop("component", "HtmlTag")
                        .default("div")
                        .doc("The element to render, such as `section`, `article`, `aside`, or `a` for a clickable card. A `section` or `aside` is a landmark and needs your `aria-label`."),
                    prop("variables", "Variables")
                        .doc("Custom properties set on the element's `style`, for a component built on `Paper`."),
                    prop("framework_sx", "&'static StaticSx")
                        .doc("Base styles for a component built on `Paper`. They replace `Paper`'s own, so start from `paper_sx()`."),
                    prop("children", "Element").doc("The surface's contents."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A surface with a background, a corner radius, an elevation and an "
                    "optional hairline border. Every card, panel and popup in the library "
                    "sits on one, and "
                    Code { source: "Dialog" }
                    " is a "
                    Code { source: "Paper" }
                    " with a role. It has no ARIA of its own. Padding comes from your "
                    Code { source: "sx" }
                    ". "
                    Code { source: "glass" }
                    " turns it into frosted glass for chrome such as a sticky bar, the way this "
                    "site's header is drawn."
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
                    Control::switch("glass"),
                    Control::toggle("component", ["div", "section", "article"])
                        .labels(["Div", "Section", "Article"]),
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

                    let glass = values.str("glass") == "true";
                    let paper = rsx! {
                        Paper {
                            radius: values.str("radius"),
                            shadow,
                            bordered: values.str("bordered") == "true",
                            glass,
                            component: values.str("component"),
                            sx,
                            // `md` is an h4 by size, and the preview sits
                            // straight under the page's h1 - the level is a
                            // document decision, the size a design one
                            // ([[codebase/heading-order]]).
                            Title { size: "md", component: "h2", "Invoice #4021" }
                            Text { "Due 30 September." }
                        }
                    };
                    let backdrop = libero::sx::sx().padding("xl").background(STRIPES);

                    rsx! {
                        if glass {
                            Box { sx: backdrop, {paper} }
                        } else {
                            {paper}
                        }
                    }
                },
                wrap: Wrap(wrap_backdrop),
            }
            DocSection { title: "Accessibility",
                Text {
                    "A "
                    Code { source: "Paper" }
                    " rendered as a "
                    Code { source: "section" }
                    " or "
                    Code { source: "aside" }
                    " is a landmark and needs your "
                    Code { source: "aria-label" }
                    ". As an "
                    Code { source: "a" }
                    " the whole surface is one link, named by its contents, so nothing "
                    "interactive belongs inside it."
                }
            }
        }
    }
}
