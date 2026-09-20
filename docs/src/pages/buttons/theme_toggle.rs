use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, Input, Text, ThemeToggle},
    theme::ThemeSet,
};

#[component]
pub fn ThemeTogglePage() -> Element {
    rsx! {
        DocPage {
            title: "ThemeToggle",
            source: "libero/src/components/buttons/theme_toggle.rs",
            markdown: "/md/theme_toggle.md",
            properties: vec![props("ThemeToggle", vec![
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
            accessibility: a11y()
                .handles([
                    "The button's name says what a press does, from the localization's `ThemeToggleLabels`: `to_light`, `to_dark` or `to_system`.",
                    "With `themes`, both buttons sit in a `role=\"group\"` named by `group`. The picker is named by `picker` and opens a `Menu`, with its keys; the sets are radio items in a group named by `themes`.",
                ])
                .must(["With `label`, return what the press does, not the current scheme."]),
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
                    ": it reads the setting and what it resolves to, and "
                    Code { source: "set" }
                    ", "
                    Code { source: "toggle" }
                    " and "
                    Code { source: "cycle" }
                    " change it."
                }
                Text {
                    "With "
                    Code { source: "themes" }
                    " set, a second button beside it opens a menu of theme sets. This site's "
                    "header uses "
                    Code { source: "ThemeToggle {{ themes: ThemeSet::CATALOGUE }}" }
                    "."
                }
            },
            Demo {
                component: "ThemeToggle",
                children_text: "",
                controls: vec![
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "standard"],
                    )
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Standard"])
                    .default("outlined"),
                    // `muted` is what an unset `color` resolves to, so that
                    // swatch prints nothing.
                    Control::color("color").default("muted"),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm"),
                    Control::switch("disabled"),
                    Control::switch("themes").code(|_, values| match values.str("themes").as_str() {
                        "true" => vec!["themes: ThemeSet::CATALOGUE".to_string()],
                        _ => vec![],
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    ThemeToggle {
                        variant: values.str("variant"),
                        color: match values.str("color").as_str() {
                            "muted" => Input::None,
                            color => Input::from(color),
                        },
                        size: values.str("size"),
                        radius: values.str("radius"),
                        disabled: values.str("disabled") == "true",
                        themes: (values.str("themes") == "true").then_some(ThemeSet::CATALOGUE),
                    }
                },
            }
        }
    }
}
