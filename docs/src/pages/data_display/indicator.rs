use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, indent, prop, props};
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

/// What the avatar is announced as. The indicator is `aria-hidden`, so the
/// count has to be in the name of the thing it marks - this is the a11y
/// contract, shown rather than described.
fn avatar_name(values: &DemoValues) -> String {
    match label(values) {
        Some(count) => format!("Ada Lovelace, {count} unread"),
        None => "Ada Lovelace, online".to_string(),
    }
}

/// Everything around the indicator prints: the relative box, the avatar it
/// marks, and the `Float` that owns the corner. `placement` is a `Float` prop,
/// so it lands here and not on `Indicator`.
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
                    .doc("The count. `None` is the bare dot. A number, not text, so `max` can cap it."),
                prop("max", "Option<u32>")
                    .default("99")
                    .doc("Above it, the label renders as `{max}+`. The theme's cap unless set."),
                prop("size", "Size")
                    .default("md")
                    .doc("The dot's diameter, and the height of a labelled one, on the indicator's own scale - 6px to 22px."),
                prop("color", "ThemeAwareValue")
                    .default("error")
                    .doc("The fill; a theme color name or a literal CSS color. A theme color also brings the `-contrast` twin the label reads with."),
                prop("radius", "Size")
                    .default("xxl")
                    .doc("A step on the indicator's own radius scale, `1px` to `6px`. The default, `xxl`, is `9999px`: round at every diameter."),
                prop("with_border", "bool")
                    .default("false")
                    .doc("A ring in the surface color, `--lsx-paper-background`, so the dot reads on top of a picture."),
                prop("processing", "bool")
                    .default("false")
                    .doc("A ping growing and fading behind the dot, repeating until you set it back to `false`: turn it off when the work ends (WCAG 2.2.2). Stops under `prefers-reduced-motion`."),
            ])],
            lead: rsx! {
                Text {
                    "A dot or a small count pinned to something else - an unread marker on an "
                    "avatar, a pending count on a button. Renders one "
                    Code { source: "<span aria-hidden=\"true\">" }
                    " and nothing that positions it: the corner, the offset and the layer are "
                    Code { source: "Float" }
                    "'s, inside a "
                    Code { source: "position: relative" }
                    " parent. To hide it, do not render it. The count is not announced - put "
                    "it in the name of what the indicator marks, as the avatar's "
                    Code { source: "alt" }
                    " does below."
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
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default(theme.indicator.size.as_str()),
                    // An unset `color` is the theme's error role, so that
                    // swatch prints nothing.
                    Control::color("color").default("error"),
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
