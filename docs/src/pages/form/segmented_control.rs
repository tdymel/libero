use crate::components::{
    Control, Demo, DemoFile, DemoValues, DocPage, FieldCopy, Wrap, a11y, description_position_prop,
    field_controls, field_props, indent, label_trailing_prop, or_unset, prop, props, readonly_prop,
    status_prop,
};
use libero::components::Pictogram;
use libero::components::SegmentedControlPart;
use pictogram_icons_lucide as lucide;

use dioxus::prelude::*;
use libero::components::{
    Code, Icon, Input, OptionLabel, OptionList, Options, Orientation, SegmentedControl, Text,
};
use libero::theme::{Responsive, Size, responsive};
use libero::use_theme;

struct AlignmentCopy;

impl FieldCopy for AlignmentCopy {
    const LABEL: &'static str = "Alignment";
    const DESCRIPTION: &'static str = "Where each line starts.";
    const HELPER: &'static str = "Applies to the whole document.";
    const WARNING: &'static str = "Justified text is harder to read.";
    const ERROR: &'static str = "This layout only takes centered text.";
    // No `aria_label` prop: unlabelled, the attribute names it.
    const ARIA_LABEL: &'static str = "\"aria-label\"";
}

/// The page's own source: the printed parts are cut from its live demo.
const FILE: DemoFile = DemoFile(include_str!("segmented_control.rs"));

// demo-code: alignment start
#[derive(Clone, Copy, PartialEq, Options)]
enum Alignment {
    Left,
    Center,
    #[option(label = "Right edge")]
    Right,
}
// demo-code: alignment end

fn renamed(alignment: Alignment) -> OptionLabel {
    // demo-code: renamed start
    match alignment {
        Alignment::Left => "Links".into(),
        Alignment::Center => "Mitte".into(),
        Alignment::Right => "Rechts".into(),
    }
    // demo-code: renamed end
}

// A segment is a `label`, so its content stays phrasing: `Icon` is a `span`, a `Flex` a `div`.
// `currentColor` keeps the icon readable on the filled selected segment.
fn rich(alignment: Alignment) -> OptionLabel {
    // demo-code: rich start
    OptionLabel::with_icon(
        alignment.label(),
        rsx! {
            Icon {
                variant: "standard",
                color: "currentColor",
                size: "sm",
                match alignment {
                    Alignment::Left => rsx! { Pictogram { icon: lucide::text_align_start::outlined } },
                    Alignment::Center => rsx! { Pictogram { icon: lucide::text_align_center::outlined } },
                    Alignment::Right => rsx! { Pictogram { icon: lucide::text_align_end::outlined } },
                }
            }
        },
    )
    // demo-code: rich end
}

fn is_on(values: &DemoValues, name: &str) -> bool {
    values.str(name) == "true"
}

/// The demo's size and orientation from the `md` breakpoint up, `sm` and the other one below it.
fn responsive_on(values: &DemoValues) -> bool {
    values.str("responsive") == "true"
}

fn picked(values: &DemoValues) -> Orientation {
    Orientation::from(values.str("orientation").as_str())
}

fn other(orientation: Orientation) -> Orientation {
    match orientation {
        Orientation::Horizontal => Orientation::Vertical,
        Orientation::Vertical => Orientation::Horizontal,
    }
}

/// The other orientation below `md`, the picked one from it, while the `responsive` switch is on.
fn responsive_orientation(values: &DemoValues) -> Responsive<Orientation> {
    match responsive_on(values) {
        true => responsive(other(picked(values))).md(picked(values)),
        false => picked(values).into(),
    }
}

fn orientation_code(_: &Control, values: &DemoValues) -> Vec<String> {
    match (responsive_on(values), picked(values)) {
        (true, orientation) => vec![format!(
            "orientation: responsive(Orientation::{:?}).md(Orientation::{orientation:?})",
            other(orientation)
        )],
        (false, Orientation::Horizontal) => vec![],
        (false, orientation) => vec![format!("orientation: {:?}", orientation.as_str())],
    }
}

#[component]
pub fn SegmentedControlPage() -> Element {
    let theme = use_theme();
    let mut alignment = use_signal(|| Alignment::Left);

    rsx! {
        DocPage {
            title: "SegmentedControl",
            source: "libero/src/components/form/segmented_control",
            markdown: "/md/segmented_control.md",
            properties: vec![
                props("SegmentedControl", vec![
                    prop("value", "T")
                        .doc("The selected segment. Pair it with `onchange`. A control bound to a `Form` through `name` leaves it out."),
                    prop("onchange", "EventHandler<T>")
                        .doc("Called with the segment to select next."),
                    prop("name", "FieldName<T>")
                        .doc("What the control posts as. A path such as `Settings::FIELDS.align()` also binds it to the surrounding `Form`'s value when it has no `onchange`. The path stays a `FieldName<T>`, not an `Option<T>` as on `Select`: exactly one segment is always selected."),
                    prop("validate", "Validators<T>")
                        .doc("Rules over the selection, shown once the control loses focus or its form is submitted."),
                    prop("options", "OptionSource<T>")
                        .default("T::options()")
                        .doc("Narrows or reorders the strip. A runtime set of `String`s goes here. A `Vec<T>` converts, and an `OptionList<T>` can disable single segments. Named groups are drawn flat, without headings."),
                    prop("option_label", "Callback<T, OptionLabel>")
                        .default("T::label()")
                        .doc("Renames a segment, or draws it with `OptionLabel::with_icon` or `OptionLabel::rich`. Runs during render, so it can read a locale from context. `#[option(label = \"..\")]` renames one at the type."),
                    prop("orientation", "Responsive<Orientation>")
                        .default("horizontal")
                        .doc("A row or a column. Or one per breakpoint, `orientation: responsive(Orientation::Vertical).md(Orientation::Horizontal)`."),
                    prop("variant", "Responsive<Variant>")
                        .default(theme.segmented_control.variant.as_str())
                        .doc("The unselected look, shared by every segment (`filled` draws them tonal, under a solid pick): `filled`, `tonal`, `elevated`, `outlined`, `standard`, or `gradient`, which takes the theme's gradient. Or one per breakpoint, `variant: responsive(Variant::Filled).md(Variant::Outlined)`."),
                    prop("color", "ThemeAwareValue")
                        .default(theme.segmented_control.color.as_str())
                        .doc("Accent color. A theme color name or any CSS color; `theme.segmented_control.color` when unset."),
                    prop("size", "Responsive<Size>").default(theme.button.size.as_str()).doc("Size of the segments and the captions. Or one per breakpoint, `size: responsive(Size::Sm).md(Size::Lg)`."),
                    prop("radius", "Responsive<ThemeAwareValue>")
                        .default(theme.button.radius.as_str())
                        .doc("Radius of the control's outer corners. Inner corners are square. Or any CSS, e.g. `radius: \"0\"`. Or one per breakpoint, `radius: responsive(Size::Sm).md(Size::Xl)`."),
                    prop("gap", "Responsive<ThemeAwareValue>")
                        .doc("Space between the segments, or any CSS, e.g. `gap: \"0\"`. Set, each segment gets its own border and radius. Unset, a wrapped second line keeps the joined look, so set it where the row may wrap. Or one per breakpoint, `gap: responsive(ThemeAwareValue::from(\"0\")).md(Size::Sm.into())`."),
                    prop("full_width", "bool")
                        .default("false")
                        .doc("Segments share the width evenly instead of sizing to their label. A label too long for its segment ends in an ellipsis either way."),
                    prop("selected_check", "bool")
                        .default(if theme.segmented_control.selected_check { "true" } else { "false" })
                        .doc("A check before the selected segment's label. Unset follows `theme.segmented_control.selected_check`. `false` keeps the segment's width; the selected segment keeps its ring and fill."),
                    prop("focusable", "bool")
                        .default("true")
                        .doc("`false` keeps the segments out of the tab order, and a click leaves focus where it is. For a control inside a field's dropdown."),
                    prop("label", "Caption")
                        .doc("The question, and the group's name."),
                    prop("description", "Caption")
                        .doc("Between the label and the segments. How to choose."),
                    label_trailing_prop(),
                    description_position_prop(),
                    prop("helper", "Caption")
                        .doc("Under the segments. What the choice changes."),
                    status_prop(),
                    prop("required", "bool")
                        .default("false")
                        .doc("Sets `aria-required` on the group and marks the label. Inside a `Form`, an empty one fails the submit."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables every segment and dims the captions."),
                    readonly_prop("control"),
                ])
                .parts("SegmentedControlPart", vec![
                    (SegmentedControlPart::Label, "The label above the control."),
                    (SegmentedControlPart::Required, "The required asterisk, in the label."),
                    (SegmentedControlPart::LabelRow, "The row holding the label and its `label_trailing`; drawn only with a trailing slot."),
                    (SegmentedControlPart::LabelTrailing, "The `label_trailing` slot, at the end of the label row."),
                    (SegmentedControlPart::Description, "The caption between the label and the control."),
                    (SegmentedControlPart::Control, "The connected strip."),
                    (SegmentedControlPart::Segment, "One segment's visible label."),
                    (SegmentedControlPart::Helper, "The caption under the control."),
                    (SegmentedControlPart::Status, "The validation message."),
                ]),
                props("Domain enums", vec![
                    prop(".choices(&X)", "Mapped<V, Choice<V>, Infallible>")
                        .doc("On a path to `Option<V>`: binds `name` to a domain enum through a `Choices<V>` list. Pass `X.list()` as `options` too. While the value holds `None`, the control writes the list's default (else its first entry) into it once, so the value holds what the strip shows; again after a reset."),
                ]).without_base_props(),
                props("OptionLabel", vec![
                    prop("name", "String").doc("The segment's accessible name, and its text when there is no `content`."),
                    prop("content", "Element")
                        .doc("Drawn in place of the name, such as an icon and text. `name` still names the segment for screen readers. A segment is a `<label>`, so phrasing content only: an `Icon` works, a `Flex` does not. Give the `Icon` `color: \"currentColor\"`, or it is primary on the filled selected segment."),
                ]),
            ],
            accessibility: a11y()
                .key(["Tab"], "Enters and leaves the whole control.")
                .key(["Left", "Right", "Up", "Down"], "Move the selection.")
                .key(["Space"], "Picks the focused segment.")
                .key(["Enter"], "Outside a `Form`: picks the focused segment. Inside one: submits the form, as on a native radio.")
                .handles(["Under a finger every segment is at least 44px tall, in a wrapped or vertical strip too."])
                .must(["Without a visible label, spread `\"aria-label\"`, since the segments name the options, not the question."])
                .example("An alignment strip with no visible label, `\"aria-label\": \"Alignment\"`: Tab enters the whole strip once, the arrows move the selection, and the next Tab leaves it.")
                .limits(["A row too long for its container wraps (WCAG 1.4.10). Without `gap`, the second line keeps the joined look: its first segment overlaps by 1px and its ends stay square. Set `gap` or a vertical `orientation` where it may wrap.", "With `selected_check`, picking a segment widens it by the check, so the strip reflows; theme `selected_check: false` keeps its width."]),
            lead: rsx! {
                Text {
                    "A connected strip of segments over an enum, exactly one of them selected. "
                    "The segments are the enum's variants, so a misspelled one does not "
                    "compile. Pass "
                    Code { source: "value" }
                    " with "
                    Code { source: "onchange" }
                    ", or bind it to a "
                    Code { source: "Form" }
                    " through "
                    Code { source: "name" }
                    ". A runtime set of "
                    Code { source: "String" }
                    "s goes in "
                    Code { source: "options" }
                    "."
                }
            },
            // snippet: let mut alignment = use_signal(|| Alignment::Left);
            Demo {
                component: "SegmentedControl",
                children_text: "",
                // Printed above the snippet: the strip is the enum, so the
                // code block is a lie without it.
                wrap: Wrap(|_: &DemoValues, source: &str| format!("{}\n\n{source}", FILE.section("alignment"))),
                // Required as a pair, and neither is a value a control varies -
                // the control is only ever as selected as its owner.
                fixed: vec![
                    "value: alignment()".to_string(),
                    "onchange: move |next| alignment.set(next)".to_string(),
                ],
                controls: [vec![
                    Control::toggle("labels", ["derived", "renamed", "rich"])
                        .labels(["Derived", "Renamed", "Rich"])
                        .code(|_, values| match values.str("labels").as_str() {
                            "renamed" => vec![format!(
                                "option_label: |alignment: Alignment| -> OptionLabel {{\n{}}}",
                                indent(&FILE.section("renamed"))
                            )],
                            "rich" => vec![format!(
                                "option_label: |alignment: Alignment| {}",
                                FILE.section("rich")
                            )],
                            _ => vec![],
                        }),
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "standard", "gradient"],
                    ).default(theme.segmented_control.variant.as_str())
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Standard", "Gradient"]),
                    // A bare `primary` is what an unset `color` resolves
                    // to, so that swatch prints nothing.
                    Control::color("color").default(theme.segmented_control.color.as_str()),
                    Control::sizes("size")
                        .default("md")
                        .code(|control, values| match (responsive_on(values), values.str("size")) {
                            (true, size) => vec![format!(
                                "size: responsive(Size::Sm).md(Size::{:?})",
                                Size::from(size.as_str())
                            )],
                            (false, size) if size == control.default => vec![],
                            (false, size) => vec![format!("size: {size:?}")],
                        }),
                    // On, so the breakpoint switch shows without another switch: resize the window.
                    Control::switch("responsive").default("true").code(|_, _| vec![]),
                    // `0` is plain CSS, next to the size scale.
                    Control::slider("radius", ["0", "xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::toggle("orientation", ["horizontal", "vertical"])
                        .labels(["Horizontal", "Vertical"])
                        .code(orientation_code),
                    // "auto" is no gap at all: the segments stay connected
                    // and share their borders.
                    Control::slider("gap", ["auto", "0", "xs", "sm", "md", "lg", "xl", "xxl"]),
                ], field_controls::<AlignmentCopy>(), vec![
                    Control::switch("full_width"),
                    Control::switch("selected_check").default(if theme.segmented_control.selected_check { "true" } else { "false" }),
                    Control::switch("required"),
                    Control::switch("disabled"),
                    Control::switch("readonly"),
                    // The flag lives inside `options`, so the switch stands
                    // for one named segment rather than for a prop of its own.
                    Control::switch("disabled_option").code(|_, values| {
                        match values.str("disabled_option") == "true" {
                            true => vec![format!("options: {}", FILE.section("disabled"))],
                            false => vec![],
                        }
                    }),
                ]].concat(),
                render: move |values: DemoValues| {
                    let field = field_props::<AlignmentCopy>(&values);
                    // A segment this control refuses, not an alignment the type refuses everywhere.
                    let options = if is_on(&values, "disabled_option") {
                        // demo-code: disabled start
                        OptionList::from_options().disabling(|align| *align == Alignment::Center)
                        // demo-code: disabled end
                    } else {
                        OptionList::from_options()
                    };
                    rsx! {
                        SegmentedControl {
                            variant: values.str("variant"),
                            color: match values.str("color").as_str() {
                                "primary" => Input::None,
                                color => Input::from(color),
                            },
                            size: match (responsive_on(&values), Size::from(values.str("size").as_str())) {
                                (true, size) => responsive(Size::Sm).md(size),
                                (false, size) => size.into(),
                            },
                            radius: values.str("radius"),
                            orientation: responsive_orientation(&values),
                            gap: or_unset::<libero::sx::ThemeAwareValue>(values.str("gap")),
                            full_width: is_on(&values, "full_width"),
                            selected_check: is_on(&values, "selected_check"),
                            label: field.label,
                            "aria-label": field.aria_label,
                            description: field.description,
                            description_position: field.description_position,
                            helper: field.helper,
                            status: field.status,
                            required: is_on(&values, "required").then_some(true),
                            disabled: is_on(&values, "disabled").then_some(true),
                            readonly: is_on(&values, "readonly").then_some(true),
                            option_label: match values.str("labels").as_str() {
                                "renamed" => Some(Callback::new(renamed)),
                                "rich" => Some(Callback::new(rich)),
                                _ => None,
                            },
                            options,
                            value: alignment(),
                            onchange: move |next| alignment.set(next),
                        }
                    }
                },
            }
        }
    }
}
