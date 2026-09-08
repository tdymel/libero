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
                prop("themes", "&'static [&'static ThemeSet]")
                    .doc("Opts into the theme picker: a chevron beside the toggle opening a menu of these sets, the active one checked. The pair is then a named `group`, and `class`, `sx` and extra attributes land on it."),
                prop("label", "Callback<ColorSchemeSetting, String>")
                    .doc("Replaces the theme's three toggle names. Given the setting a press moves to, it names what the press does."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables interaction and dims the button."),
            ])],
            lead: rsx! {
                Text {
                    "An icon button that steps the app's colour scheme: following the "
                    "platform, then the scheme the platform is not showing, then the one it "
                    "is, then back to following it. The glyph shows the setting in effect - a "
                    "half-filled disc, a sun or a moon - and the name says what a press does. "
                    "It is an "
                    Code { source: "ActionIcon" }
                    " over "
                    Code { source: "use_color_scheme()" }
                    "."
                }
                Text {
                    "With "
                    Code { source: "themes" }
                    " it becomes a split button: a chevron beside the toggle opens a menu of "
                    "theme sets. Two buttons rather than one with a second gesture, so each "
                    "has one job, one name and its own tab stop. The one in this site's "
                    "header is "
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
