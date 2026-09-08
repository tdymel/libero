use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, ColorSchemeButton, Input, Text};

#[component]
pub fn ColorSchemeButtonPage() -> Element {
    rsx! {
        DocPage {
            title: "ColorSchemeButton",
            source: "libero/src/components/inputs/color_scheme_button.rs",
            markdown: "/md/color_scheme_button.md",
            properties: vec![props("ColorSchemeButton", vec![
                prop("variant", "Variant")
                    .default("outlined")
                    .doc("`ActionIcon`'s chrome. Unset, the theme's `color_scheme_button.variant`."),
                prop("color", "ThemeAwareValue")
                    .default("muted")
                    .doc("Accent color. Unset, the theme's `color_scheme_button.color`."),
                prop("size", "ThemeAwareValue")
                    .default("md")
                    .doc("Button size; the glyph takes 55% of it."),
                prop("radius", "ThemeAwareValue")
                    .default("sm")
                    .doc("Corner radius, independent of size."),
                prop("label", "Callback<ColorScheme, String>")
                    .doc("Replaces the theme's two names. Given the scheme on screen, it names what a press does."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables interaction and dims the button."),
            ])],
            lead: rsx! {
                Text {
                    "An icon button that flips the app between its light and dark theme: a "
                    "moon while the light scheme shows, a sun while the dark one does. It is an "
                    Code { source: "ActionIcon" }
                    " over "
                    Code { source: "use_color_scheme()" }
                    ", named for what a press does."
                }
                Text {
                    "A press pins the other scheme only while it differs from the platform's. "
                    "Flipping back hands the choice to the platform again, so an OS switch - or "
                    "a devtools emulation of one - is followed from then on."
                }
            },
            Demo {
                component: "ColorSchemeButton",
                children_text: "",
                controls: vec![
                    Control::toggle("variant", ["outlined", "filled", "tonal", "standard"])
                        .labels(["Outlined", "Filled", "Tonal", "Standard"])
                        .default("outlined"),
                    // `muted` is what an unset `color` resolves to, so that
                    // swatch prints nothing.
                    Control::color("color").default("muted"),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    ColorSchemeButton {
                        variant: values.str("variant"),
                        color: match values.str("color").as_str() {
                            "muted" => Input::None,
                            color => Input::from(color),
                        },
                        size: values.str("size"),
                        disabled: values.str("disabled") == "true",
                    }
                },
            }
        }
    }
}
