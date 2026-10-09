use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Avatar, Box, Code, Float, Indicator, Input, Text},
    sx::sx,
    use_theme,
};

static AVATAR_IMAGE: Asset = asset!("/assets/avatar.svg");

/// The count, or `None` for the bare dot.
fn label(values: &DemoValues) -> Option<u32> {
    values.str("label").parse().ok()
}

/// The avatar's name carries the count: the indicator itself is `aria-hidden`.
fn avatar_name(values: &DemoValues) -> String {
    match label(values) {
        Some(count) => format!("Ada Lovelace, {count} unread"),
        None => "Ada Lovelace, online".to_string(),
    }
}

/// Prints the relative box, the avatar and the `Float`, which takes `placement`, not `Indicator`.
fn wrap_anchor(values: &DemoValues, code: &str) -> String {
    format!(
        "Box {{\n    sx: sx().position(\"relative\").display(\"inline-flex\"),\n    Avatar {{ name: {:?}, alt: {:?}, src: AVATAR_IMAGE, size: \"lg\" }}\n    Float {{\n        placement: {:?},\n{}    }}\n}}",
        "Ada Lovelace",
        avatar_name(values),
        values.str("placement"),
        indent(&indent(code)),
    )
}

#[component]
pub fn IndicatorPage() -> Element {
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "Indicator",
            source: "libero/src/components/data_display/indicator.rs",
            markdown: "/md/indicator.md",
            properties: vec![props("Indicator", vec![
                prop("label", "Option<u32>")
                    .default("None")
                    .doc("The count. `None` is the bare dot. A number, so `max` can cap it."),
                prop("max", "Option<u32>")
                    .default(theme.indicator.max.to_string())
                    .doc("Above it, the label renders as `{max}+`. Falls back to the theme's cap."),
                prop("size", "Size")
                    .default(theme.indicator.size.as_str())
                    .doc("The dot's diameter, and the height of a labelled one, from 6px to 22px."),
                prop("color", "ThemeAwareValue")
                    .default(theme.indicator.color.as_str())
                    .doc("The fill, a theme color name or a CSS color. A theme or hex color also sets a label color that reads on it; any other CSS color gets `contrast-color()`."),
                prop("radius", "Size")
                    .default(theme.indicator.radius.as_str())
                    .doc("A step on the indicator's own radius scale, `1px` to `6px`. The default `xxl` is round at every size."),
                prop("with_border", "bool")
                    .default("false")
                    .doc("A ring in the surface color, so the dot reads on top of a picture."),
                prop("processing", "bool")
                    .default("false")
                    .doc("A ping behind the dot that repeats until you set it back to `false`. Stops under `prefers-reduced-motion`."),
            ])],
            accessibility: a11y()
                .handles([
                    "Screen readers never read the indicator.",
                    "A theme or hex color labels the count at 4.5:1 or better.",
                ])
                .must([
                    "Put the count in the name of what it marks, as the demo avatar's `alt: \"Ada Lovelace, 128 unread\"`, or `aria_label: \"Messages, 128 unread\"` on a button.",
                    "To have the indicator read, pass `aria_hidden: \"false\"` and wrap it in your own `role=\"status\"` region.",
                    "`processing` pings until you turn it off. Set it back to `false` when the work ends, since motion that never stops fails WCAG 2.2.2.",
                    "Give the fill 3:1 against what is around it (WCAG 1.4.11). A CSS color other than hex labels the count with `contrast-color()`, which a browser without it ignores.",
                ])
                .example("An unread count on an avatar, `Indicator { label: 128 }` with `Avatar { alt: \"Ada Lovelace, 128 unread\" }`: a screen reader hears the count once, in the avatar's name, and never reads the indicator."),
            lead: rsx! {
                Text {
                    "A dot or a small count pinned to something else, such as an unread "
                    "marker on an avatar. It does not position itself. Put it in a "
                    Code { source: "Float" }
                    " inside a "
                    Code { source: "position: relative" }
                    " parent, which sets the corner, the offset and the layer. To hide it, "
                    "do not render it."
                }
            },
            Demo {
                component: "Indicator",
                children_text: "",
                controls: vec![
                    Control::slider("label", ["none", "1", "9", "128"])
                        .code(|_, values| match label(values) {
                            Some(count) => vec![format!("label: {count}")],
                            None => vec![],
                        }),
                    Control::sizes("size")
                        .default(theme.indicator.size.as_str()),
                    // An unset `color` is the theme's error role, so that
                    // swatch prints nothing.
                    Control::color("color").default(theme.indicator.color.as_str()),
                    Control::toggle("placement", ["top-end", "top-start", "bottom-end", "bottom-start"])
                        .labels(["Top end", "Top start", "Bottom end", "Bottom start"])
                        .code(|_, _| vec![]),
                    Control::switch("with_border"),
                    Control::switch("processing"),
                ],
                render: move |values: DemoValues| rsx! {
                    Box {
                        sx: sx().position("relative").display("inline-flex"),
                        Avatar {
                            name: "Ada Lovelace",
                            alt: avatar_name(&values),
                            src: AVATAR_IMAGE.to_string(),
                            size: "lg",
                        }
                        Float {
                            placement: values.str("placement"),
                            Indicator {
                                label: label(&values),
                                size: values.str("size"),
                                color: match values.str("color").as_str() {
                                    "error" => Input::None,
                                    color => Input::from(color),
                                },
                                with_border: values.str("with_border") == "true",
                                processing: values.str("processing") == "true",
                            }
                        }
                    }
                },
                wrap: Wrap(wrap_anchor),
            }
        }
    }
}
