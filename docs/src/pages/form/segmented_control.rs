use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, Wrap, or_unset, prop, props,
};
use crate::icons::{AlignCenterIcon, AlignLeftIcon, AlignRightIcon};
use dioxus::prelude::*;
use libero::components::{
    Code, FieldStatus, Icon, Input, OptionLabel, OptionList, Options, SegmentedControl, Text,
};

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
// snippet: after ALIGNMENT_ENUM
// snippet: let mut alignment = use_signal(|| Alignment::Left);
// snippet: in SegmentedControl { value: alignment(), onchange: move |next| alignment.set(next), .. }
const RENAMED: &str = r#"option_label: |alignment: Alignment| -> OptionLabel {
    match alignment {
        Alignment::Left => "Links".into(),
        Alignment::Center => "Mitte".into(),
        Alignment::Right => "Rechts".into(),
    }
}"#;

// A segment is a `label`, so its content has to stay phrasing content: `Icon`
// is an inline-flex `span` (and it is what sizes the raw svg), a `Flex` is a
// `div`.
// snippet: after ALIGNMENT_ENUM
// snippet: item #[component] fn AlignLeftIcon() -> Element { rsx! {} }
// snippet: item #[component] fn AlignCenterIcon() -> Element { rsx! {} }
// snippet: item #[component] fn AlignRightIcon() -> Element { rsx! {} }
// snippet: let mut alignment = use_signal(|| Alignment::Left);
// snippet: in SegmentedControl { value: alignment(), onchange: move |next| alignment.set(next), .. }
const RICH: &str = r#"option_label: |alignment: Alignment| OptionLabel::rich(
    alignment.label(),
    rsx! {
        Icon {
            variant: "standard",
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

/// The flag sits on the option, inside the one `options` prop - a segment
/// this control refuses, rather than an alignment the type refuses everywhere.
// snippet: after ALIGNMENT_ENUM
// snippet: let mut alignment = use_signal(|| Alignment::Left);
// snippet: in SegmentedControl { value: alignment(), onchange: move |next| alignment.set(next), .. }
const DISABLED_OPTION: &str =
    r#"options: OptionList::from_options().disabling(|align| *align == Alignment::Center)"#;

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
                variant: "standard",
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

fn is_on(values: &DemoValues, name: &str) -> bool {
    values.str(name) == "true"
}

#[component]
pub fn SegmentedControlPage() -> Element {
    let mut alignment = use_signal(|| Alignment::Left);

    rsx! {
        DocPage {
            title: "SegmentedControl",
            source: "libero/src/components/form/segmented_control",
            markdown: "/md/segmented_control.md",
            properties: vec![
                props("SegmentedControl", vec![
                    prop("value", "T")
                        .doc("Strictly controlled - pair it with `onchange`. Exactly one segment is selected, which is what makes this a radio group and not a row of toggles. A control bound to a `Form` through `name` leaves it out."),
                    prop("onchange", "EventHandler<T>")
                        .doc("Called with the segment that should become selected."),
                    prop("name", "FieldName<T>")
                        .doc("What the control posts as. A path - `Settings::FIELDS.align()` - also binds it to the surrounding `Form`'s value when it has no `onchange`."),
                    prop("validate", "Validators<T>")
                        .doc("Rules over the selection, shown once the control loses focus or its form is submitted."),
                    prop("options", "OptionSource<T>")
                        .default("T::options()")
                        .doc("Narrows or reorders the strip. A runtime set of `String`s passes them here, since `String` lists no options of its own. A `Vec<T>` converts; an `OptionList<T>` adds per-option `disabled`. Named groups are accepted and drawn flattened - a single row of segments has nowhere to put headings."),
                    prop("option_label", "Callback<T, OptionLabel>")
                        .default("T::label()")
                        .doc("Overrides what the derive named a segment. Runs during render, so it can read a locale from context."),
                    prop("orientation", "Orientation")
                        .default("horizontal")
                        .doc("Row or column layout."),
                    prop("variant", "Variant")
                        .default("filled")
                        .doc("The unselected look, shared by every segment."),
                    prop("color", "ThemeAwareValue")
                        .default("primary")
                        .doc("Accent color; a theme color name or a literal CSS color."),
                    prop("size", "Size").default("md").doc("Shared by every segment, and by the captions around them."),
                    prop("radius", "Size")
                        .default("md")
                        .doc("Corner radius of the control's outer corners; inner ones are square."),
                    prop("gap", "Size")
                        .doc("Space between the segments. Set it and they stop sharing borders - each keeps its own, and its own radius."),
                    prop("full_width", "bool")
                        .default("false")
                        .doc("Segments share the width evenly instead of sizing to their label. Either way a label too long for the strip ends in an ellipsis, with the whole name as its `title`."),
                    prop("focusable", "bool")
                        .default("true")
                        .doc("`false` keeps the segments out of the tab order, and a click leaves focus where it is - for a control inside a field's dropdown."),
                    prop("label", "Caption")
                        .doc("The question. Names the group through `aria-labelledby`, since `for` cannot name a `role=\"radiogroup\"`."),
                    prop("description", "Caption")
                        .doc("Between the label and the segments: how to choose."),
                    prop("helper", "Caption")
                        .doc("Under the segments. Consequences of the choice."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, rendered under the helper. A bare `&str` is an error."),
                    prop("required", "bool")
                        .default("false")
                        .doc("Adds `aria-required` to the group and marks the label."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables every segment and dims the captions."),
                    prop("readonly", "bool")
                        .default("false")
                        .doc("Focusable and posted with the form, but not editable - unlike `disabled`, which drops the field from the tab order and from the post."),
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
                    Code { source: "options" }
                    " instead."
                }
                Text {
                    "It is a field like "
                    Code { source: "RadioGroup" }
                    ": a label, captions and a status around the strip, a "
                    Code { source: "name" }
                    " that binds it to a "
                    Code { source: "Form" }
                    ", and rules through "
                    Code { source: "validate" }
                    "."
                }
            },
            // snippet: let mut alignment = use_signal(|| Alignment::Left);
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
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "text"],
                    )
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Text"]),
                    // A bare `primary` is what an unset `color` resolves
                    // to, so that swatch prints nothing.
                    Control::color("color"),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::toggle("orientation", ["horizontal", "vertical"]),
                    // "auto" is no gap at all: the segments stay connected
                    // and share their borders.
                    Control::slider("gap", ["auto", "xs", "sm", "md", "lg", "xl", "xxl"]),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                r#"status: FieldStatus::Warning("Justified text is harder to read.".into())"#
                                    .to_string(),
                            ],
                            "error" => vec![r#"status: "Pick an alignment.""#.to_string()],
                            _ => vec![],
                        }),
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec![r#"label: "Alignment""#.to_string()],
                            // Unlabelled, it still needs a name.
                            _ => vec![r#""aria-label": "Alignment""#.to_string()],
                        }
                    }),
                    Control::switch("description").code(|_, values| {
                        match values.str("description").as_str() {
                            "true" => vec![r#"description: "Where each line starts.""#.to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("helper").code(|_, values| {
                        match values.str("helper").as_str() {
                            "true" => vec![r#"helper: "Applies to the whole document.""#.to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("full_width"),
                    Control::switch("required"),
                    Control::switch("disabled"),
                    // The flag lives inside `options`, so the switch stands
                    // for one named segment rather than for a prop of its own.
                    Control::switch("disabled_option").code(|_, values| {
                        match values.str("disabled_option") == "true" {
                            true => vec![DISABLED_OPTION.to_string()],
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
                        full_width: is_on(&values, "full_width"),
                        label: is_on(&values, "label").then(|| "Alignment".to_string()),
                        "aria-label": (!is_on(&values, "label")).then_some("Alignment"),
                        description: is_on(&values, "description")
                            .then(|| "Where each line starts.".to_string()),
                        helper: is_on(&values, "helper")
                            .then(|| "Applies to the whole document.".to_string()),
                        status: match values.str("status").as_str() {
                            "warning" => FieldStatus::Warning("Justified text is harder to read.".to_string()),
                            "error" => FieldStatus::Error("Pick an alignment.".to_string()),
                            _ => FieldStatus::Valid,
                        },
                        required: is_on(&values, "required").then_some(true),
                        disabled: is_on(&values, "disabled").then_some(true),
                        option_label: match values.str("labels").as_str() {
                            "renamed" => Some(Callback::new(renamed)),
                            "rich" => Some(Callback::new(rich)),
                            _ => None,
                        },
                        options: {
                            let off = is_on(&values, "disabled_option");
                            OptionList::from_options()
                                .disabling(move |align| off && *align == Alignment::Center)
                        },
                        value: alignment(),
                        onchange: move |next| alignment.set(next),
                    }
                },
            }
            DocSection {
                title: "Accessibility",
                Text {
                    "Arrow keys move the selection; Tab enters and leaves the whole control. "
                    "Space picks the focused segment, and so does Enter outside a Form; inside one Enter submits it, as on a native radio. Without a visible label, name it by spreading "
                    Code { source: "\"aria-label\"" }
                    ": the segments name the options, not the question."
                }
            }
        }
    }
}
