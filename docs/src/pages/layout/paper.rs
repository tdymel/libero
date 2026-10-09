use crate::components::{
    Control, Demo, DemoValues, DocPage, UNSET, Wrap, a11y, gradient_controls, gradient_value,
    indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Box, Code, Flex, Input, Paper, Text, Title},
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

/// An id nothing has, so the demo link scrolls and navigates nowhere.
const HREF: &str = "#order-4021";

/// Large squares, well past the blur radius, in a colour and the page's ink, so the
/// blur softens hard edges the glass overlaps.
const BACKDROP: &str = "repeating-conic-gradient(var(--lsx-secondary-fill-6) 0 25%, var(--lsx-ink) 0 50%) 0 0 / 96px 96px";

/// The second glass beside the demo's: tinted if the demo's is plain, plain if it is tinted.
fn other_glass(values: &DemoValues) -> (Option<&'static str>, &'static str) {
    match values.str("color").as_str() {
        UNSET => (Some("primary"), "Tinted glass"),
        _ => (None, "Plain glass"),
    }
}

/// Glass prints the backdrop `Box` it sits on, with the second glass beside it; off, the bare `Paper`.
fn wrap_backdrop(values: &DemoValues, code: &str) -> String {
    if values.str("glass") != "true" {
        return code.to_string();
    }
    let (color, label) = other_glass(values);
    let color = color
        .map(|color| format!("color: {color:?}, "))
        .unwrap_or_default();
    format!(
        "Box {{\n    sx: sx().padding(\"xl\").background({BACKDROP:?}),\n    Flex {{ gap: \"md\", wrap: \"wrap\",\n{}\n        Paper {{ glass: true, {color}sx: sx().padding(\"lg\"), Text {{ {label:?} }} }}\n    }}\n}}",
        indent(&indent(code))
    )
}

fn no_gradient(values: &DemoValues) -> bool {
    values.str("gradient") != "true"
}

#[component]
pub fn PaperPage() -> Element {
    let theme = use_theme();
    let [gradient_to, gradient_deg] = gradient_controls(no_gradient);

    rsx! {
        DocPage {
            title: "Paper",
            source: "libero/src/components/layout/paper.rs",
            markdown: "/md/paper.md",
            properties: vec![
                props("Paper", vec![
                    prop("radius", "ThemeAwareValue")
                        .default(theme.paper.radius.as_str())
                        .doc("Corner radius, a step on the shared radius scale, or any CSS, e.g. `radius: \"0\"`."),
                    prop("shadow", "Size")
                        .default(theme.paper.shadow.as_str())
                        .doc("Elevation, a step on the shared shadow scale. For a flat surface use `sx().box_shadow(\"none\")`."),
                    prop("bordered", "bool")
                        .default("false")
                        .doc("A hairline border in the theme's surface border colour. Works together with a shadow."),
                    prop("glass", "bool")
                        .default("false")
                        .doc("Frosted glass: translucent, blurring what is behind it, tuned by the theme's `paper.glass_background` and `paper.glass_blur`. A `color` or `gradient` tints it, and a tinted glass gets glass cues: a saturated backdrop, a top highlight and a light sheen. Use it over app chrome such as a sticky bar (this site's header is drawn with it), not over images, where text can lose contrast. Opaque when the user reduces transparency, in forced colours, and in native windows, where a coloured one is its solid fill."),
                    prop("color", "ThemeAwareValue")
                        .doc("Fills the surface. A theme color name paints its shade 6 under a text colour picked to read on it; any other CSS color is used as given, and its text colour is yours to set. With `glass`, a translucent tint of it, its share raised until the text reads 4.5:1: the more a colour must carry text, the less see-through the glass. Under a gradient, its first stop."),
                    prop("gradient", "Gradient")
                        .doc("Fills the surface with a gradient from `color` to a second stop, as `(\"info\", 90)` or `Gradient::default().to(\"info\").deg(90)`; `Gradient::default()` is the theme's. The text colour is picked to read on both stops. With `glass`, the stops turn translucent, their share raised until the text reads 4.5:1; a literal stop's text is yours to check. Its stops carry down to any gradient inside it."),
                    prop("component", "HtmlTag")
                        .default("div")
                        .doc("The element to render, such as `section`, `article`, `aside`, or `a` for a clickable card. An `aside` is a landmark and needs your `aria-label`; a `section` becomes one once you give it an `aria-label`. A surface that takes a press, an `a` with an `href`, a `button` or any element with an `onclick`, also tints on hover, tints deeper while pressed and shows the focus ring (the `interactive` state)."),
                    prop("variables", "Variables")
                        .doc("Custom properties set on the element's `style`, for a component built on `Paper`."),
                    prop("framework_sx", "&'static StaticSx")
                        .doc("Base styles for a component built on `Paper`. They replace `Paper`'s own, so start from `paper_sx()`."),
                    prop("children", "Element").doc("The surface's contents."),
                ]),
            ],
            accessibility: a11y().must([
                "Give a `Paper` rendered as an `aside` an `aria-label`: it is a landmark. A `section` becomes a landmark once you give it an `aria-label`, so name the ones a reader should find.",
                "Put nothing interactive inside a `Paper` rendered as an `a`: the whole surface is one link, named by its contents.",
                "Render a pressable `Paper` as an `a` or a `button`. An `onclick` on a `div` gives the look but not a role, a tab stop or Enter and Space: add those yourself.",
            ])
                .example("A related-links panel, `Paper { component: \"aside\", \"aria-label\": \"Related articles\", .. }`: a screen reader lists it as the \"Related articles\" landmark."),
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
                    "."
                }
            },
            Demo {
                component: "Paper",
                children_text: "",
                children_code: CONTENT.to_string(),
                controls: vec![
                    // `0` is plain CSS, next to the size scale.
                    Control::slider("radius", ["0", "xs", "sm", "md", "lg", "xl", "xxl"])
                        .default(theme.paper.radius.as_str()),
                    // `none` is not a `Size`: it prints the `sx` override that does it.
                    Control::slider("shadow", ["none", "xs", "sm", "md", "lg", "xl", "xxl"])
                        .default(theme.paper.shadow.as_str())
                        .code(|control, values| match values.str("shadow").as_str() {
                            "none" => vec![NO_SHADOW.to_string()],
                            // At the default only the padding prints.
                            shadow if shadow == control.default => vec![PADDING.to_string()],
                            shadow => {
                                vec![format!("shadow: {shadow:?}"), PADDING.to_string()]
                            }
                        }),
                    Control::switch("bordered"),
                    Control::switch("glass"),
                    // Not a prop: a link is what makes the surface take a press.
                    Control::switch("link").default("true").code(|_, values| {
                        match values.str("link").as_str() {
                            "true" => vec!["component: \"a\"".to_string(), format!("href: {HREF:?}")],
                            _ => vec![],
                        }
                    }),
                    Control::color("color").with_unset(),
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
                    Control::toggle("component", ["div", "section", "article"])
                        .labels(["Div", "Section", "Article"])
                        .hidden_when(|values| values.str("link") == "true"),
                ],
                render: move |values: DemoValues| {
                    let flat = values.str("shadow") == "none";
                    let link = values.str("link") == "true";
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
                            color: match values.str("color").as_str() {
                                UNSET => Input::None,
                                color => Input::from(color),
                            },
                            gradient: (values.str("gradient") == "true")
                                .then(|| gradient_value(&values)),
                            component: match link {
                                true => "a".into(),
                                false => Input::from(values.str("component")),
                            },
                            href: link.then_some(HREF),
                            sx,
                            // Level is a document decision, size a design one ([[codebase/heading-order]]).
                            Title { size: "md", component: "h2", "Invoice #4021" }
                            Text { "Due 30 September." }
                        }
                    };
                    let backdrop = libero::sx::sx().padding("xl").background(BACKDROP);
                    let (color, label) = other_glass(&values);

                    rsx! {
                        if glass {
                            Box { sx: backdrop,
                                Flex { gap: "md", wrap: "wrap",
                                    {paper}
                                    Paper {
                                        glass: true,
                                        color: match color {
                                            Some(color) => Input::from(color),
                                            None => Input::None,
                                        },
                                        sx: libero::sx::sx().padding("lg"),
                                        Text { "{label}" }
                                    }
                                }
                            }
                        } else {
                            {paper}
                        }
                    }
                },
                wrap: Wrap(wrap_backdrop),
            }
        }
    }
}
