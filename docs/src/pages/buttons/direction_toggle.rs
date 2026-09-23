use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, DirectionToggle, Input, Text};

#[component]
pub fn DirectionTogglePage() -> Element {
    rsx! {
        DocPage {
            title: "DirectionToggle",
            source: "libero/src/components/buttons/direction_toggle.rs",
            markdown: "/md/direction_toggle.md",
            properties: vec![props("DirectionToggle", vec![
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
                prop("label", "Callback<Direction, String>")
                    .doc("Replaces the two built-in names. Gets the direction a press turns the text to and returns what the press does."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables and dims the button."),
            ])],
            accessibility: a11y()
                .handles([
                    "The button's name says what a press does, from the localization's `DirectionToggleLabels`: `to_rtl` or `to_ltr`.",
                    "It sets the document's `dir`, so a screen reader and every component follow the new direction.",
                ])
                .must(["With `label`, return what the press does, not the current direction."]),
            lead: rsx! {
                Text {
                    "An icon button that turns the app's text between left to right and "
                    "right to left. It sets the document's "
                    Code { source: "dir" }
                    ", so every component and every overlay turns with it, and the web "
                    "keeps the choice for the next visit. The arrow shows where the next "
                    "press goes."
                }
                Text {
                    "Place it and you are done: it needs no state of its own. For your own "
                    "control, build on "
                    Code { source: "use_direction()" }
                    "; the start direction is "
                    Code { source: "LiberoProvider {{ direction }}" }
                    "."
                }
            },
            Demo {
                component: "DirectionToggle",
                children_text: "",
                controls: vec![
                    Control::toggle("variant", ["outlined", "filled", "tonal", "standard"])
                        .labels(["Outlined", "Filled", "Tonal", "Standard"])
                        .default("outlined"),
                    // `muted` is what an unset `color` resolves to, so that
                    // swatch prints nothing.
                    Control::color("color").default("muted"),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    DirectionToggle {
                        variant: values.str("variant"),
                        color: match values.str("color").as_str() {
                            "muted" => Input::None,
                            color => Input::from(color),
                        },
                        size: values.str("size"),
                        radius: values.str("radius"),
                        disabled: values.str("disabled") == "true",
                    }
                },
            }
        }
    }
}
