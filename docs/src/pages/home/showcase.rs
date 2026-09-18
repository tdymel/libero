use dioxus::prelude::*;
use libero::{
    components::{
        Box, Button, CodeBlock, Flex, OptionLabel, Paper, SegmentedControl, Switch, Text,
    },
    sx::sx,
    theme::{CODE_BLOCK_BORDER, Size},
};

use super::{SectionHead, tint};
use crate::components::{Control, DemoValues, generate_code};

const LABEL: &str = "Save changes";

/// The props the showcase varies, each at the component's own default.
fn controls() -> Vec<Control> {
    vec![
        Control::toggle("variant", ["filled", "tonal", "outlined"])
            .labels(["Filled", "Tonal", "Outlined"]),
        Control::toggle("color", ["primary", "success", "error"])
            .labels(["Primary", "Success", "Error"]),
        Control::toggle("size", ["sm", "md", "lg", "xl"]).default("md"),
        Control::switch("loading"),
    ]
}

/// One segmented row of the panel, captioned by the prop's name.
#[component]
fn Segments(control: Control, value: String, onchange: EventHandler<String>) -> Element {
    let caption = control.name;
    rsx! {
        Flex { direction: "column", gap: "xs",
            Text { size: "sm", sx: sx().font_weight("600"), "{caption}" }
            SegmentedControl {
                variant: "outlined",
                size: "sm",
                full_width: true,
                "aria-label": caption,
                value,
                options: control.options.clone(),
                option_label: move |option: String| OptionLabel::from(control.label_of(&option)),
                onchange: move |next: String| onchange.call(next),
            }
        }
    }
}

/// A live `Button` beside the rsx that draws it, built by the docs' demo code.
#[component]
pub fn Showcase() -> Element {
    let base = controls();
    // Starts off the defaults, so the first snippet has props to read.
    let mut values = use_signal(|| {
        vec![
            ("variant", "tonal".to_string()),
            ("color", "success".to_string()),
            ("size", "lg".to_string()),
            ("loading", "false".to_string()),
        ]
    });
    #[cfg(test)]
    use_hook(|| {
        crate::snippets::record(&crate::components::DemoCode {
            component: "Button".into(),
            children_text: LABEL.into(),
            children_code: None,
            code_child: None,
            fixed: vec![],
            controls: controls(),
            wrap: None,
            child: None,
        })
    });

    let value = move |name: &str| {
        values()
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| value.clone())
            .unwrap_or_default()
    };
    let mut set = move |name: &'static str, next: String| {
        if let Some(entry) = values.write().iter_mut().find(|(key, _)| *key == name) {
            entry.1 = next;
        }
    };
    // The current values as defaults, against the real defaults in `base`, so
    // the generator still leaves out what a caller would not type.
    let current: Vec<Control> = base
        .iter()
        .map(|control| control.clone().default(value(control.name)))
        .collect();
    let source = generate_code(
        "Button",
        LABEL,
        None,
        &[],
        &base,
        &DemoValues::defaults(&current),
    );
    let divider = format!("1px solid {}", CODE_BLOCK_BORDER.value());

    rsx! {
        section { "aria-labelledby": "showcase-title",
            Flex { direction: "column", gap: "xl",
                SectionHead { id: "showcase-title", eyebrow: "Code", title: "Props you can read",
                    "Change a prop and the rsx follows. The code beside the preview is what you "
                    "would type, and every docs page works the same way."
                }
                Paper {
                    bordered: true,
                    shadow: "sm",
                    radius: "lg",
                    sx: sx()
                        .overflow("hidden")
                        .display("flex")
                        .flex_direction("column")
                        .breakpoint(Size::Lg, sx().flex_direction("row")),
                    Flex {
                        direction: "column",
                        sx: sx().min_width("0").breakpoint(Size::Lg, sx().flex("1 1 0")),
                        Box {
                            sx: sx()
                                .display("flex")
                                .align_items("center")
                                .justify_content("center")
                                .min_height("160px")
                                .padding("xl")
                                .background(format!(
                                    "radial-gradient(circle at 50% 50%, {} 0%, transparent 70%)",
                                    tint(16)
                                )),
                            Button {
                                variant: value("variant"),
                                color: value("color"),
                                size: value("size"),
                                loading: value("loading") == "true",
                                "{LABEL}"
                            }
                        }
                        Flex { direction: "column", gap: "md", sx: sx().padding("lg"),
                            for control in base.iter().filter(|c| c.name != "loading").cloned() {
                                Segments {
                                    key: "{control.name}",
                                    value: value(control.name),
                                    onchange: move |next: String| set(control.name, next),
                                    control,
                                }
                            }
                            Switch {
                                label: "loading",
                                checked: value("loading") == "true",
                                onchange: move |on: bool| set("loading", on.to_string()),
                            }
                        }
                    }
                    CodeBlock {
                        language: "rust",
                        source,
                        sx: sx()
                            .min_width("0")
                            .border("none")
                            .border_radius("0")
                            .border_top(divider.clone())
                            .breakpoint(
                                Size::Lg,
                                sx().flex("1 1 0").border_top("none").border_left(divider),
                            ),
                    }
                }
            }
        }
    }
}
