use crate::components::{Control, Demo, DemoFile, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::Pictogram;
use libero::components::{ActionIcon, Button, ButtonGroup, Input, Text};
use pictogram_icons_lucide as lucide;
use std::collections::HashSet;

/// The page's own source: the printed parts are cut from its live demo.
const FILE: DemoFile = DemoFile(include_str!("button_group.rs"));

#[component]
pub fn ButtonGroupPage() -> Element {
    let mut selected = use_signal(|| HashSet::from(["bold".to_string()]));
    rsx! {
        DocPage {
            title: "ButtonGroup",
            source: "libero/src/components/buttons/button_group.rs",
            markdown: "/md/button_group.md",
            properties: vec![props("ButtonGroup", vec![
                prop("orientation", "Orientation")
                    .default("horizontal")
                    .doc("`\"vertical\"` stacks the buttons, each as wide as the widest."),
                prop("variant", "Variant")
                    .doc("Default `variant` of the buttons inside; a button's own prop wins, as for `color`, `size`, `radius`, `disabled` and `focusable_when_disabled`. Between two buttons without a visible border of their own (every variant but `outlined`), the group draws a thin divider."),
                prop("color", "ThemeAwareValue")
                    .doc("Default `color` of the buttons inside."),
                prop("size", "Size")
                    .doc("Default `size` of the buttons inside."),
                prop("radius", "ThemeAwareValue")
                    .doc("The group's outer corners, or any CSS, e.g. `radius: \"0\"`. The corners between two buttons are always square."),
                prop("disabled", "bool")
                    .doc("Disables every button inside that does not set `disabled` itself, and takes it out of the Tab order unless `focusable_when_disabled` says otherwise."),
                prop("focusable_when_disabled", "bool")
                    .doc("Default `focusable_when_disabled` of the buttons inside: a disabled one stays a Tab stop."),
                prop("value", "HashSet<String>")
                    .doc("The `value`s of the pressed buttons, strictly controlled. Set, every `Button` and `ActionIcon` with a `value` is a toggle with `aria-pressed`, unless it sets `selected` itself. Unset, a plain group."),
                prop("multiple", "bool")
                    .default("false")
                    .doc("With `value`: any number of buttons pressed. Off, pressing one releases the other, and pressing the pressed one releases it."),
                prop("onchange", "EventHandler<HashSet<String>>")
                    .doc("Asks for the next set of pressed `value`s. Without it the selection can never change."),
                prop("children", "Element")
                    .default("required")
                    .doc("`Button`s, `ActionIcon`s, and components built on them, such as `ThemeSwitcher`."),
            ])],
            accessibility: a11y()
                .handles([
                    "The root is a `role=\"group\"`, so a screen reader announces the buttons as one set.",
                    "Each button stays its own Tab stop, and the focused one's ring is drawn above its neighbours.",
                    "With `value`, a button with a `value` is a toggle button: `aria-pressed` says whether it is pressed, and Space and Enter toggle it.",
                ])
                .must(["Name the group with `aria-label`, or `aria-labelledby` on a visible heading."])
                .example("Zoom out, Reset and Zoom in in a `ButtonGroup` with `\"aria-label\": \"Zoom\"`: a screen reader reads the three buttons as the \"Zoom\" group, and Tab stops on each one.")
                .limits([
                    "It is not a `toolbar`: no arrow-key navigation between the buttons.",
                    "For one choice out of several, use `SegmentedControl`, which is a radio group.",
                    "A child hidden with `display: none` still counts as first or last and squares its neighbour's outer corners; render a conditional control only when shown.",
                ]),
            lead: rsx! {
                Text {
                    "Buttons and action icons side by side as one control. Neighbours share "
                    "one seam and only the group's outer corners are round, on logical sides, "
                    "so the ends swap under right-to-left text."
                }
            },
            // snippet: let mut selected = use_signal(|| std::collections::HashSet::from(["bold".to_string()]));
            Demo {
                component: "ButtonGroup",
                children_text: "",
                children_code: FILE.section("children"),
                fixed: vec!["\"aria-label\": \"Text style\"".to_string()],
                controls: vec![
                    Control::toggle("orientation", ["horizontal", "vertical"])
                        .labels(["Horizontal", "Vertical"])
                        .default("horizontal"),
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "standard", "gradient"],
                    )
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Standard", "Gradient"])
                    .default("filled"),
                    // A bare `primary` is what an unset `color` resolves to, so that swatch prints nothing.
                    Control::color("color"),
                    Control::sizes("size").default("md"),
                    Control::sizes("radius")
                        .default("sm"),
                    Control::switch("disabled"),
                    Control::switch("focusable_when_disabled"),
                    // On, so the pressed look shows without another switch.
                    Control::toggle("selection", ["off", "single", "multiple"])
                        .labels(["Off", "Single", "Multiple"])
                        .default("single")
                        .code(|_, values| match values.str("selection").as_str() {
                            "off" => vec![],
                            "single" => vec![
                                "value: selected()".to_string(),
                                "onchange: move |next| selected.set(next)".to_string(),
                            ],
                            _ => vec![
                                "value: selected()".to_string(),
                                "multiple: true".to_string(),
                                "onchange: move |next| selected.set(next)".to_string(),
                            ],
                        }),
                ],
                render: move |values: DemoValues| rsx! {
                    ButtonGroup {
                        "aria-label": "Text style",
                        orientation: values.str("orientation"),
                        value: (values.str("selection") != "off").then(|| selected.cloned()),
                        multiple: values.str("selection") == "multiple",
                        onchange: move |next| selected.set(next),
                        variant: values.str("variant"),
                        color: match values.str("color").as_str() {
                            "primary" => Input::None,
                            color => Input::from(color),
                        },
                        size: values.str("size"),
                        radius: values.str("radius"),
                        disabled: values.str("disabled") == "true",
                        focusable_when_disabled: values.str("focusable_when_disabled") == "true",
                        // demo-code: children start
                        Button { value: "bold", "Bold" }
                        Button { value: "italic", "Italic" }
                        ActionIcon {
                            value: "underline",
                            aria_label: "Underline",
                            Pictogram { icon: lucide::underline::outlined }
                        }
                        // demo-code: children end
                    }
                },
            }
        }
    }
}
