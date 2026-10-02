use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::use_theme;
use libero::{
    components::{Code, Input, Text, ThemeSwitcher, ThemeSwitcherPart},
    theme::{ColorSchemeSetting, ThemeSet},
};

/// The `label` switch's names, printed as `LABEL` shows them.
fn scheme_label(to: ColorSchemeSetting) -> String {
    match to {
        ColorSchemeSetting::Light => "Use light colours".to_string(),
        ColorSchemeSetting::Dark => "Use dark colours".to_string(),
        ColorSchemeSetting::System => "Follow the system's colours".to_string(),
    }
}

// snippet: in ThemeSwitcher { .. }
const LABEL: &str = r#"label: |to: ColorSchemeSetting| match to {
    ColorSchemeSetting::Light => "Use light colours".to_string(),
    ColorSchemeSetting::Dark => "Use dark colours".to_string(),
    ColorSchemeSetting::System => "Follow the system's colours".to_string(),
}"#;

#[component]
pub fn ThemeSwitcherPage() -> Element {
    let theme = use_theme();
    rsx! {
        DocPage {
            title: "ThemeSwitcher",
            source: "libero/src/components/buttons/theme_switcher.rs",
            markdown: "/md/theme_switcher.md",
            properties: vec![props("ThemeSwitcher", vec![
                prop("variant", "Variant")
                    .default(theme.theme_switcher.variant.as_str())
                    .doc("Visual style, as on `ActionIcon`."),
                prop("color", "ThemeAwareValue")
                    .default(theme.theme_switcher.color.as_str())
                    .doc("Accent color. A theme color name or any CSS color."),
                prop("size", "ThemeAwareValue")
                    .default("md")
                    .doc("Button size. The icon takes 55% of it."),
                prop("radius", "ThemeAwareValue")
                    .default("sm")
                    .doc("Corner radius, independent of `size`."),
                prop("themes", "&'static [&'static ThemeSet]")
                    .doc("Adds the theme picker, a second button that opens a menu of these sets. `class`, `sx` and extra attributes then land on the group around both."),
                prop("with_system", "bool")
                    .default("false")
                    .doc("Adds following the system to the cycle. Off, a press flips between light and dark, and flipping to the system's own scheme follows the system again."),
                prop("label", "Callback<ColorSchemeSetting, String>")
                    .doc("Replaces the three built-in button names. Gets the setting a press moves to and returns what the press does."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables and dims the button."),
                prop("parts", "Parts<ThemeSwitcherPart>")
                    .doc("Styles for the inner parts in the Style API tab, under `sx`. `Toggle`, `Picker` and `Chevron` exist with `themes` only."),
                prop("menu_parts", "Parts<MenuPart>")
                    .doc("The theme-set menu's `parts`, the `Menu` page's Style API table. The menu opens in a portal, out of `sx` and `parts`."),
            ])
            .parts("ThemeSwitcherPart", vec![
                (ThemeSwitcherPart::Icon, "The sun, moon or system glyph."),
                (ThemeSwitcherPart::Toggle, "The scheme button beside the picker. Without `themes`, the root is the toggle."),
                (ThemeSwitcherPart::Picker, "The button that opens the theme-set menu."),
                (ThemeSwitcherPart::Chevron, "The picker's chevron."),
            ])],
            accessibility: a11y()
                .handles([
                    "The button's name says what a press does, from the localization's `ThemeSwitcherLabels`: `to_light` and `to_dark`, plus `to_system` with `with_system`.",
                    "With `themes`, both buttons sit in a `role=\"group\"` named by `group`. The picker is named by `picker` and opens a `Menu`, with its keys; the sets are radio items in a group named by `themes`.",
                ])
                .must(["With `label`, return what the press does, not the current scheme."]),
            lead: rsx! {
                Text {
                    "An icon button that switches the app's colour scheme. It starts on the "
                    "system's scheme, and each press flips to the other one. Flipping back to "
                    "the system's scheme follows the system again, so the app never stays "
                    "pinned. The icon shows where the next press goes: a sun for light, a "
                    "moon for dark."
                }
                Text {
                    "With "
                    Code { source: "with_system" }
                    ", following the system is a step of its own. A press goes from following "
                    "the system, to the scheme the system is not showing, to the one it is, "
                    "and back, so under a light system the order is system, dark, light. The "
                    "icon for that step is a half-filled disc."
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
                    Code { source: "ThemeSwitcher {{ themes: ThemeSet::CATALOGUE }}" }
                    "."
                }
            },
            Demo {
                component: "ThemeSwitcher",
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
                    Control::switch("with_system").code(|_, values| match values.str("with_system").as_str() {
                        "true" => vec!["with_system: true".to_string()],
                        _ => vec![],
                    }),
                    Control::switch("label").code(|_, values| match values.str("label").as_str() {
                        "true" => vec![LABEL.to_string()],
                        _ => vec![],
                    }),
                    Control::switch("themes").code(|_, values| match values.str("themes").as_str() {
                        "true" => vec!["themes: ThemeSet::CATALOGUE".to_string()],
                        _ => vec![],
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    ThemeSwitcher {
                        variant: values.str("variant"),
                        color: match values.str("color").as_str() {
                            "muted" => Input::None,
                            color => Input::from(color),
                        },
                        size: values.str("size"),
                        radius: values.str("radius"),
                        disabled: values.str("disabled") == "true",
                        with_system: values.str("with_system") == "true",
                        label: (values.str("label") == "true").then(|| Callback::new(scheme_label)),
                        themes: (values.str("themes") == "true").then_some(ThemeSet::CATALOGUE),
                    }
                },
            }
        }
    }
}
