use crate::components::{
    Child, Control, Demo, DemoValues, DocPage, UNSET, Wrap, a11y, indent, prop, props,
};
use dioxus::prelude::*;
use libero::theme::{Responsive, Size, responsive};
use libero::{
    components::{Box, Divider, DividerPart, Input, Orientation, Text},
    sx::sx,
    use_theme,
};

/// `xs` below `md`, the chosen size from it, while the `responsive` switch is on.
fn responsive_size(values: &DemoValues) -> Responsive<Size> {
    let size = Size::from(values.str("size").as_str());
    match values.str("responsive") == "true" {
        true => responsive(Size::Xs).md(size),
        false => size.into(),
    }
}

fn size_code(control: &Control, values: &DemoValues) -> Vec<String> {
    match (values.str("responsive") == "true", values.str("size")) {
        (true, size) => vec![format!(
            "size: responsive(Size::Xs).md(Size::{:?})",
            Size::from(size.as_str())
        )],
        (false, size) if size == control.default => vec![],
        (false, size) => vec![format!("size: {size:?}")],
    }
}

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

/// The wrapper's layout for one orientation: a row a vertical rule can span, or a column.
fn layout_code(orientation: Orientation) -> &'static str {
    match orientation {
        Orientation::Vertical => {
            r#".flex_direction("row").align_items("center").justify_content("center").height("120px")"#
        }
        Orientation::Horizontal => r#".flex_direction("column").width("240px")"#,
    }
}

fn layout_sx(orientation: Orientation) -> libero::sx::Sx {
    match orientation {
        Orientation::Vertical => sx()
            .flex_direction("row")
            .align_items("center")
            .justify_content("center")
            // The label takes ~43px: a shorter row leaves stubs of rule.
            .height("120px")
            .width("auto"),
        Orientation::Horizontal => sx().flex_direction("column").width("240px").height("auto"),
    }
}

/// The wrapper follows the rule: per breakpoint when the rule is.
fn wrapper_sx(values: &DemoValues) -> libero::sx::Sx {
    let base = sx().display("flex").text_align("center");
    match responsive_on(values) {
        true => base
            .and(layout_sx(other(picked(values))))
            .breakpoint(Size::Md, layout_sx(picked(values))),
        false => base.and(layout_sx(picked(values))),
    }
}

fn wrapper_code(values: &DemoValues) -> String {
    match responsive_on(values) {
        true => format!(
            "sx().display(\"flex\").text_align(\"center\"){}.breakpoint(Size::Md, sx(){})",
            layout_code(other(picked(values))),
            layout_code(picked(values))
        ),
        false => format!(
            "sx().display(\"flex\").text_align(\"center\"){}",
            layout_code(picked(values))
        ),
    }
}

/// The label is a child, not a prop, so a control has to decide it here.
fn label_child(values: &DemoValues) -> String {
    match values.str("with_label").as_str() {
        "true" => "OR".to_string(),
        _ => String::new(),
    }
}

/// A printed wrapper gives the rule an axis to span and neighbours for `spacing`'s margin.
fn wrap_rule(values: &DemoValues, code: &str) -> String {
    let indented = indent(code);
    format!(
        "Box {{\n    sx: {},\n    Text {{ \"One\" }}\n{indented}    Text {{ \"Two\" }}\n}}",
        wrapper_code(values)
    )
}

#[component]
pub fn DividerPage() -> Element {
    let theme = use_theme();
    rsx! {
        DocPage {
            title: "Divider",
            source: "libero/src/components/layout/divider.rs",
            markdown: "/md/divider.md",
            properties: vec![props("Divider", vec![
                prop("orientation", "Responsive<Orientation>")
                    .default("horizontal")
                    .doc("The direction of the rule. A vertical rule stretches to its row's height, so it needs a flex row parent; in a block parent it is 0px high and does not show. Or one per breakpoint, `orientation: responsive(Orientation::Horizontal).md(Orientation::Vertical)`: the rule switches with the window from the first paint, while `aria-orientation` follows after mount when the page was rendered on a server."),
                prop("size", "Responsive<Size>").default(theme.divider.size.as_str()).doc("Line thickness. Or one per breakpoint, `size: responsive(Size::Xs).md(Size::Sm)`."),
                prop("label_position", "LabelPosition")
                    .default(theme.divider.label_position.as_str())
                    .doc("Where the label sits along the rule: `center`, `start` or `end`."),
                prop("spacing", "Responsive<ThemeAwareValue>")
                    .default("none")
                    .doc("Margin on both sides of the rule, a spacing step or a CSS length. Or one per breakpoint, `spacing: responsive(Size::Xs).md(Size::Lg)`."),
                prop("color", "ThemeAwareValue")
                    .default("muted.4")
                    .doc("Line color. A bare theme color like `primary` resolves to its shade 3."),
                prop("children", "Element")
                    .doc("An optional label in the line."),
                prop("parts", "Parts<DividerPart>")
                    .doc("Styles for the inner parts in the Style API tab, under `sx`."),
            ])
            .parts("DividerPart", vec![
                (DividerPart::Label, "The label between the two line halves, with `children` only."),
            ])],
            accessibility: a11y()
                .handles([
                    "The rule is a `separator`, named by its label. Your own `aria-label` or `aria-labelledby` wins. A `role` other than `separator` turns the label naming and `aria-orientation` off.",
                    "A long label on a horizontal rule wraps inside the line instead of running out of a narrow container. A vertical rule's label stays on one line.",
                ])
                .must(["Pass `role: \"none\"` for a purely visual rule."])
                .example("A rule between two settings groups, `Divider { \"Advanced\" }`: a screen reader reads a separator named \"Advanced\". A rule that only spaces two cards takes `role: \"none\"` and is skipped."),
            lead: rsx! {
                Text { "A horizontal or vertical rule, with an optional label sitting in the line." }
            },
            Demo {
                component: "Divider",
                children_text: "OR",
                controls: vec![
                    Control::toggle("orientation", ["horizontal", "vertical"])
                        .labels(["Horizontal", "Vertical"])
                        .default("horizontal")
                        .code(orientation_code),
                    Control::sizes("size")
                        .default(theme.divider.size.as_str()).code(size_code),
                    // On, so the breakpoint switch shows without another switch: resize the window.
                    Control::switch("responsive").default("true").code(|_, _| vec![]),
                    Control::switch("with_label").default("true").code(|_, _| vec![]),
                    // Meaningless without a label.
                    Control::toggle("label_position", ["start", "center", "end"])
                        .labels(["Start", "Center", "End"])
                        .default(theme.divider.label_position.as_str())
                        .hidden_when(|values| values.str("with_label") != "true"),
                    // Unset draws muted.4, which bare `muted` (shade 3) is not: the unset swatch is painted muted.4.
                    Control::color("color").with_unset()
                    .unset_swatch("muted.4"),
                    Control::slider("spacing", ["auto", "xs", "sm", "md", "lg", "xl"])
                        .default("md")
                        .code(
                            |_, values| match values.str("spacing").as_str() {
                                // The theme's own default, not a value.
                                "auto" => vec![],
                                spacing => vec![format!("spacing: {spacing:?}")],
                            },
                        ),
                ],
                render: move |values: DemoValues| {
                    let divider = rsx! {
                        Divider {
                            orientation: responsive_orientation(&values),
                            size: responsive_size(&values),
                            label_position: values.str("label_position"),
                            color: match values.str("color").as_str() {
                                UNSET => Input::None,
                                color => Input::from(color),
                            },
                            // An empty string would be a literal value,
                            // not "leave it to the theme".
                            spacing: match values.str("spacing").as_str() {
                                "auto" => Input::None,
                                spacing => Input::<libero::theme::Responsive<libero::sx::ThemeAwareValue>>::from(spacing),
                            },
                            children: (values.str("with_label") == "true")
                                .then(|| rsx! { "OR" }),
                        }
                    };
                    rsx! {
                        Box { sx: wrapper_sx(&values),
                            Text { "One" }
                            {divider}
                            Text { "Two" }
                        }
                    }
                },
                wrap: Wrap(wrap_rule),
                child: Child(label_child),
            }
        }
    }
}
