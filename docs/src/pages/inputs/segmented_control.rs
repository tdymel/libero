use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, Wrap, or_unset, prop, props,
};
use crate::icons::{AlignCenterIcon, AlignLeftIcon, AlignRightIcon};
use dioxus::prelude::*;
use libero::components::{Code, Icon, Input, OptionLabel, Options, SegmentedControl, Text};

/// The enum is the strip, so the snippet has to show it.
const ALIGNMENT_ENUM: &str = r#"#[derive(Clone, Copy, PartialEq, Options)]
enum Alignment {
    Left,
    Center,
    #[option(label = "Right edge")]
    Right,
}

"#;

/// Printed verbatim when the `labels` control asks for it, and rendered by the
/// closure right below - the block is a promise that the two are the same.
const RENAMED: &str = r#"label: |alignment: Alignment| match alignment {
    Alignment::Left => "Links".into(),
    Alignment::Center => "Mitte".into(),
    Alignment::Right => "Rechts".into(),
}"#;

// A segment is a `label`, so its content has to stay phrasing content: `Icon`
// is an inline-flex `span` (and it is what sizes the raw svg), a `Flex` is a
// `div`.
const RICH: &str = r#"label: |alignment: Alignment| OptionLabel::rich(
    alignment.label(),
    rsx! {
        Icon {
            variant: "transparent",
            size: "sm",
            match alignment {
                Alignment::Left => rsx! { AlignLeftIcon {} },
                Alignment::Center => rsx! { AlignCenterIcon {} },
                Alignment::Right => rsx! { AlignRightIcon {} },
            }
        }
        "{alignment.label()}"
    },
)"#;

const DISABLED: &str = "disabled: vec![Alignment::Center]";

#[derive(Clone, Copy, PartialEq, Options)]
enum Alignment {
    Left,
    Center,
    #[option(label = "Right edge")]
    Right,
}

fn renamed(alignment: Alignment) -> OptionLabel {
    match alignment {
        Alignment::Left => "Links".into(),
        Alignment::Center => "Mitte".into(),
        Alignment::Right => "Rechts".into(),
    }
}

fn rich(alignment: Alignment) -> OptionLabel {
    OptionLabel::rich(
        alignment.label(),
        rsx! {
            Icon {
                variant: "transparent",
                size: "sm",
                match alignment {
                    Alignment::Left => rsx! { AlignLeftIcon {} },
                    Alignment::Center => rsx! { AlignCenterIcon {} },
                    Alignment::Right => rsx! { AlignRightIcon {} },
                }
            }
            "{alignment.label()}"
        },
    )
}

#[component]
pub fn SegmentedControlPage() -> Element {
    let mut alignment = use_signal(|| Alignment::Left);

    rsx! {
        DocPage {
            title: "SegmentedControl",
            source: "libero/src/components/inputs/segmented_control",
            markdown: "/md/segmented_control.md",
            properties: vec![
                props("SegmentedControl", vec![
                    prop("value", "T")
                        .doc("Strictly controlled - pair it with `onchange`. Exactly one segment is selected, which is what makes this a radio group and not a row of toggles."),
                    prop("onchange", "EventHandler<T>")
                        .doc("Called with the segment that should become selected."),
                    prop("segments", "Vec<T>")
                        .default("T::options()")
                        .doc("Narrows or reorders the strip. A runtime set of `String`s passes them here, since `String` lists no options of its own."),
                    prop("label", "Callback<T, OptionLabel>")
                        .default("T::label()")
                        .doc("Overrides what the derive named a segment. Runs during render, so it can read a locale from context."),
                    prop("disabled", "Vec<T>")
                        .doc("Segments that render but cannot be picked."),
                    prop("orientation", "Orientation")
                        .default("horizontal")
                        .doc("Row or column layout."),
                    prop("variant", "ButtonVariant")
                        .default("outlined")
                        .doc("The unselected look, shared by every segment."),
                    prop("color", "ThemeAwareValue")
                        .default("primary")
                        .doc("Accent color; a theme color name or a literal CSS color."),
                    prop("size", "Size").default("md").doc("Shared by every segment."),
                    prop("radius", "Size")
                        .default("md")
                        .doc("Corner radius of the control's outer corners; inner ones are square."),
                    prop("gap", "Size")
                        .doc("Space between the segments. Set it and they stop sharing borders - each keeps its own, and its own radius."),
                    prop("full_width", "bool")
                        .default("false")
                        .doc("Segments share the width evenly instead of sizing to their label."),
                ]),
                props("OptionLabel", vec![
                    prop("name", "String").doc("The segment's accessible name, and its text when there is no `content`."),
                    prop("content", "Element")
                        .doc("Drawn in place of the name, via `OptionLabel::rich` - an icon or a badge. `name` still names the segment, since the rsx is what a screen reader cannot use."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A connected strip of segments over an enum, exactly one of them selected. "
                    "The segments are the enum's variants - "
                    Code { source: "#[derive(Options)]" }
                    " lists them in declaration order and names each one - so a misspelled "
                    "segment is a compile error rather than a selection that never matches. "
                    "Strictly controlled: "
                    Code { source: "value" }
                    " drives the look, "
                    Code { source: "onchange" }
                    " reports the segment that should become selected. A runtime set of "
                    Code { source: "String" }
                    "s goes through "
                    Code { source: "segments" }
                    " instead."
                }
            },
            Demo {
                component: "SegmentedControl",
                children_text: "",
                // Printed above the snippet: the strip is the enum, so the
                // code block is a lie without it.
                wrap: Wrap(|_: &DemoValues, source: &str| format!("{ALIGNMENT_ENUM}{source}")),
                // Required as a pair, and neither is a value a control varies -
                // the control is only ever as selected as its owner.
                fixed: vec![
                    "value: alignment()".to_string(),
                    "onchange: move |next| alignment.set(next)".to_string(),
                ],
                controls: vec![
                    Control::toggle("labels", ["derived", "renamed", "rich"])
                        .labels(["Derived", "Renamed", "Rich"])
                        .code(|_, values| match values.str("labels").as_str() {
                            "renamed" => vec![RENAMED.to_string()],
                            "rich" => vec![RICH.to_string()],
                            _ => vec![],
                        }),
                    Control::toggle("variant", ["outlined", "filled", "text"])
                        .labels(["Outlined", "Filled", "Text"]),
                    // A bare `primary` is what an unset `color` resolves
                    // to, so that swatch prints nothing.
                    Control::color(
                        "color",
                        ["primary", "secondary", "success", "error", "warning", "info"],
                    ),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::toggle("orientation", ["horizontal", "vertical"]),
                    // "auto" is no gap at all: the segments stay connected
                    // and share their borders.
                    Control::slider("gap", ["auto", "xs", "sm", "md", "lg", "xl", "xxl"]),
                    Control::switch("full_width"),
                    // `disabled` is a `Vec<T>`, not a bool, so the switch
                    // stands for one named segment rather than the prop.
                    Control::switch("disabled").code(|_, values| {
                        match values.str("disabled") == "true" {
                            true => vec![DISABLED.to_string()],
                            false => vec![],
                        }
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    SegmentedControl {
                        variant: values.str("variant"),
                        color: match values.str("color").as_str() {
                            "primary" => Input::None,
                            color => Input::from(color),
                        },
                        size: values.str("size"),
                        radius: values.str("radius"),
                        orientation: values.str("orientation"),
                        gap: or_unset(values.str("gap")),
                        full_width: values.str("full_width") == "true",
                        label: match values.str("labels").as_str() {
                            "renamed" => Some(Callback::new(renamed)),
                            "rich" => Some(Callback::new(rich)),
                            _ => None,
                        },
                        disabled: match values.str("disabled") == "true" {
                            true => vec![Alignment::Center],
                            false => Vec::new(),
                        },
                        value: alignment(),
                        onchange: move |next| alignment.set(next),
                    }
                },
            }
            DocSection {
                title: "Accessibility",
                Text {
                    "The root is a "
                    Code { source: "role=\"radiogroup\"" }
                    " and every segment is a "
                    Code { source: "label" }
                    " around a visually hidden "
                    Code { source: "input type=\"radio\"" }
                    ". That is what a segmented control is: exactly one of a set, never none "
                    "and never two - so a screen reader announces \"1 of 3\", and arrow keys "
                    "move the selection while Tab enters and leaves the whole control. All of "
                    "it is the browser's own, so there is no roving tabindex to maintain."
                }
                Text {
                    "The radio's click is cancelled and the selection written from "
                    Code { source: "value" }
                    " instead, so the DOM property, "
                    Code { source: ":checked" }
                    " and the accessibility tree can never disagree with Rust. Name the control "
                    "with an "
                    Code { source: "aria_label" }
                    " where its purpose is not obvious from the segments themselves."
                }
                SegmentedControl {
                    aria_label: "Text alignment",
                    value: alignment(),
                    onchange: move |next| alignment.set(next),
                }
            }
            DocSection {
                title: "Independent toggles are a different component",
                Text {
                    "Bold, italic and underline are three booleans, not one selection - so they "
                    "are not segments. A row of "
                    Code { source: "Button" }
                    "s with "
                    Code { source: "selected" }
                    " set is the right shape there, and it takes separators and dropdowns "
                    "between them, which a strip built from "
                    Code { source: "T::options()" }
                    " cannot."
                }
            }
        }
    }
}
