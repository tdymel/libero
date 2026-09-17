use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, ColorSchemeButton, Input, Text},
    theme::ThemeSet,
};

#[component]
pub fn ColorSchemeButtonPage() -> Element {
    rsx! {
        DocPage {
            title: "ColorSchemeButton",
            source: "libero/src/components/buttons/color_scheme_button.rs",
            markdown: "/md/color_scheme_button.md",
            properties: vec![props("ColorSchemeButton", vec![
                prop("variant", "Variant")
                    .default("outlined")
                    .doc("Visual style, as on `ActionIcon`."),
                prop("color", "ThemeAwareValue")
                    .default("muted")
                    .doc("Accent color. A theme color name or any CSS color."),
                prop("size", "ThemeAwareValue")
                    .default("md")
                    .doc("Button size. The icon takes 55% of it."),
                prop("radius", "ThemeAwareValue")
                    .default("sm")
                    .doc("Corner radius, independent of `size`."),
                prop("themes", "&'static [&'static ThemeSet]")
                    .doc("Adds the theme picker, a second button that opens a menu of these sets. `class`, `sx` and extra attributes then land on the group around both."),
                prop("label", "Callback<ColorSchemeSetting, String>")
                    .doc("Replaces the three built-in button names. Gets the setting a press moves to and returns what the press does."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables and dims the button."),
            ])],
            lead: rsx! {
                Text {
                    "An icon button that switches the app's colour scheme. Each press steps "
                    "from following the system, to the scheme the system is not showing, to "
                    "the one it is, and back. The icon shows where the next press goes: a sun "
                    "for light, a moon for dark, a half-filled disc for following the system."
                }
                Text {
                    "While it follows the system, a change of the OS setting applies at once. "
                    "A picked scheme stays until the next press. For your own control, such "
                    "as a menu of all three choices, build on "
                    Code { source: "use_color_scheme()" }
                    "."
                }
                Text {
                    "With "
                    Code { source: "themes" }
                    " set, a second button beside it opens a menu of theme sets. This site's "
                    "header uses "
                    Code { source: "ColorSchemeButton {{ themes: ThemeSet::CATALOGUE }}" }
                    "."
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
                    Control::switch("themes").code(|_, values| match values.str("themes").as_str() {
                        "true" => vec!["themes: ThemeSet::CATALOGUE".to_string()],
                        _ => vec![],
                    }),
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
                        themes: (values.str("themes") == "true").then_some(ThemeSet::CATALOGUE),
                    }
                },
            }
        }
    }
}
