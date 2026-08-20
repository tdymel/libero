use dioxus::prelude::*;
use libero::{
    components::{Box, Code, Flex, Text, ToggleButton, ToggleButtonGroup},
    sx::sx,
    theme::{CODE_BORDER, Size},
};

use crate::icons::CheckmarkIcon;

use super::Control;

/// `"color"` -> `"Color"`. Prop names are the control labels.
fn label(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// The current value of every control, keyed by prop name.
#[derive(Clone, PartialEq)]
pub struct DemoValues(Vec<(&'static str, String)>);

impl DemoValues {
    pub fn str(&self, name: &str) -> String {
        self.0
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| value.clone())
            .unwrap_or_default()
    }
}

/// A live example: `render` on the left, a control per prop on the right,
/// and the rsx those values add up to below.
#[component]
pub fn Demo(
    component: String,
    children_text: String,
    controls: Vec<Control>,
    render: Callback<DemoValues, Element>,
) -> Element {
    let mut values = use_signal(|| {
        DemoValues(
            controls
                .iter()
                .map(|control| (control.name, control.default.clone()))
                .collect(),
        )
    });

    let source = super::generate_code(&component, &children_text, &controls, &values().0);
    // The color belongs *inside* each shorthand: a later `border-left: 1px
    // solid` would otherwise reset the color to `currentColor`.
    let border = format!("1px solid {}", CODE_BORDER.value());

    rsx! {
        Box {
            sx: sx()
                .border(border.clone())
                .border_radius("6px")
                // Keeps the code block's own square corners inside the card's
                // rounded ones.
                .overflow("hidden"),
            Flex {
                direction: "row",
                wrap: "wrap",
                align: "stretch",
                sx: sx().border_bottom(border.clone()),
                Box {
                    sx: sx()
                        .flex("1")
                        .min_width("240px")
                        .padding("24px")
                        .display("flex")
                        .align_items("center")
                        .justify_content("center"),
                    {render.call(values())}
                }
                Flex {
                    direction: "column",
                    gap: "md",
                    // Wrapped, the controls sit below the preview, so the
                    // divider has to move with them.
                    sx: sx()
                        .width("100%")
                        .flex_shrink("0")
                        .padding("24px")
                        .border_top(border.clone())
                        .breakpoint(
                            Size::Sm,
                            sx()
                                // 220px of controls plus the padding either
                                // side - `box-sizing` is border-box here.
                                .width("268px")
                                .border_top("none")
                                .border_left(border.clone()),
                        ),
                    for (index, control) in controls.iter().enumerate() {
                        Flex {
                            key: "{control.name}",
                            direction: "column",
                            gap: "xs",
                            Text {
                                size: "sm",
                                sx: sx().font_weight("600"),
                                {label(control.name)}
                            }
                            ToggleButtonGroup {
                                size: "sm",
                                full_width: true,
                                // Swatches read as separate chips, not one
                                // segmented control.
                                gap: "xs",
                                variant: "text",
                                value: vec![values().str(control.name)],
                                // Clicking the selected swatch would otherwise
                                // clear it, leaving the prop with no value.
                                onchange: move |next: Vec<String>| {
                                    if let Some(value) = next.into_iter().next() {
                                        values.write().0[index].1 = value;
                                    }
                                },
                                for option in control.options.iter() {
                                    ToggleButton {
                                        key: "{option}",
                                        value: "{option}",
                                        aria_label: "{option}",
                                        // The swatch is the whole button, so
                                        // the fill reaches the border.
                                        sx: sx().padding("0"),
                                        Box {
                                            sx: sx()
                                                .width("100%")
                                                .height("100%")
                                                .border_radius("inherit")
                                                .background(option.clone())
                                                // The palette's own contrast
                                                // color, so the tick reads on
                                                // every swatch.
                                                .color(format!("{option}-contrast"))
                                                .display("flex")
                                                .align_items("center")
                                                .justify_content("center")
                                                .selector(
                                                    "& svg",
                                                    sx().width("18px").height("18px"),
                                                ),
                                            if values().str(control.name) == *option {
                                                CheckmarkIcon {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Code {
                block: true,
                language: "rust",
                source,
                header: false,
                sx: sx().border("none").border_radius("0"),
            }
        }
    }
}
