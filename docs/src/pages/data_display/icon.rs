use crate::components::{
    Control, Demo, DemoValues, DocPage, PictogramNote, a11y, gradient_controls, gradient_value,
    not_gradient_variant, prop, props,
};
use dioxus::prelude::*;
use libero::components::{Code, Icon, Input, Text};
use libero::use_theme;

/// The glyph the box draws; no control varies it.
const GLYPH: &str = "svg: pictogram_icons_lucide::check::outlined";

/// The `aria_label` switch's name for the glyph.
const NAME: &str = "Done";

#[component]
pub fn IconPage() -> Element {
    let theme = use_theme();
    let [gradient_to, gradient_deg] = gradient_controls(not_gradient_variant);
    rsx! {
        DocPage {
            title: "Icon",
            source: "libero/src/components/data_display/icon.rs",
            markdown: "/md/icon.md",
            properties: vec![
                props("Icon", vec![
                    prop("component", "HtmlTag").default("span").doc("Element to render as."),
                    prop("variant", "Variant")
                        .default(theme.icon.variant.as_str())
                        .doc("The look, shared with `Button`. An icon is not interactive, so it has no hover state."),
                    prop("gradient", "Gradient")
                        .doc("With `variant: \"gradient\"`: the second stop and the angle, as `(\"info\", 90)` or `Gradient::default().to(\"info\").deg(90)`. The first stop is `color`. Ignored by the other variants."),
                    prop("color", "ThemeAwareValue")
                        .default("primary")
                        .doc("The CSS color, which an svg drawn in `currentColor` inherits. Under `filled` a theme color also tints the background. Under a gradient, its first stop."),
                    prop("size", "ThemeAwareValue")
                        .default("md")
                        .doc("Width and height."),
                    prop("radius", "ThemeAwareValue")
                        .default("sm")
                        .doc("Corner radius."),
                    prop("src", "String")
                        .doc("An image URL drawn as the glyph, in the icon's color, instead of `children`. Only its shape is used: its own colors are ignored."),
                    prop("svg", "SvgData")
                        .doc("A glyph drawn as a `Pictogram`, such as `pictogram_icons_lucide::check::outlined`, instead of `children`. `src` wins over it."),
                    prop("children", "Element").doc("The svg. Not needed with `src` or `svg`."),
                ]),
            ],
            accessibility: a11y()
                .handles([
                    "An icon is hidden from screen readers (`aria-hidden=\"true\"`).",
                    "`aria_label` or `aria_labelledby` makes it `role=\"img\"`.",
                ])
                .must([
                    "Name an icon that means something with `aria_label` or `aria_labelledby`. A `<title>` inside the svg does not name it, since it is hidden with the rest.",
                    "For a clickable icon, use `ActionIcon`.",
                ]),
            lead: rsx! {
                PictogramNote {}
                Text {
                    "Wraps an svg in a sized, colored box: an "
                    Code { source: "SvgData" }
                    " glyph through "
                    Code { source: "svg" }
                    ", or your own svg as children. An svg drawn in "
                    Code { source: "currentColor" }
                    " takes the box's "
                    Code { source: "color" }
                    ". With "
                    Code { source: "standard" }
                    " the svg fills the box. Every other variant insets it, clear of the "
                    "box's edges."
                }
            },
            Demo {
                component: "Icon",
                children_text: "",
                fixed: vec![GLYPH.to_string()],
                controls: vec![
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "standard", "gradient"],
                    )
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Standard", "Gradient"]),
                    // A bare `primary` is what an unset `color` resolves
                    // to, so that swatch prints nothing.
                    Control::color("color"),
                    gradient_to,
                    gradient_deg,
                    Control::sizes("size").default("md"),
                    // `standard` draws no box, so there is no corner to round.
                    Control::sizes("radius")
                        .default("sm")
                        .hidden_when(|values| values.str("variant") == "standard"),
                    // Off, the icon stays hidden, as one beside a text label should.
                    Control::switch("aria_label").code(|_, values| match values.str("aria_label").as_str() {
                        "true" => vec![format!("aria_label: {NAME:?}")],
                        _ => vec![],
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    Icon {
                        variant: values.str("variant"),
                        gradient: gradient_value(&values),
                        color: match values.str("color").as_str() {
                            "primary" => Input::None,
                            color => Input::from(color),
                        },
                        size: values.str("size"),
                        radius: values.str("radius"),
                        svg: pictogram_icons_lucide::check::outlined,
                        aria_label: (values.str("aria_label") == "true").then_some(NAME),
                    }
                },
            }
        }
    }
}
