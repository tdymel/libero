use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        common::{
            HtmlTag, Input, Orientation, Part, Variables, base_props, input_from_str, parts_enum,
            variables,
        },
        layout::use_box,
    },
    hooks::{use_css, use_id, use_theme},
    str_enum::str_enum,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{
        ColorCss, ColorShade, ColorValue, CssVar, DIVIDER_LINE, DividerDefaults, Size, SizeCss,
    },
};

fn divider_color_value(value: &ThemeAwareValue) -> ThemeAwareValue {
    match value {
        ThemeAwareValue::Color(color) => {
            ThemeAwareValue::ColorValue(ColorValue::Shade(*color, ColorShade::S3))
        }
        other => other.clone(),
    }
}

str_enum! {
    /// Where a [`Divider`]'s label sits along the line.
    pub enum LabelPosition {
        Start = "start",
        #[default]
        Center = "center",
        End = "end",
    }
}

input_from_str!(LabelPosition);

const DIVIDER_COLOR_VAR: CssVar = CssVar::new("--lsx-divider-color");
const DIVIDER_SPACING_VAR: CssVar = CssVar::new("--lsx-divider-spacing");

/// The caller's `color` if they set one, else the theme's grey-4.
fn divider_color() -> String {
    DIVIDER_COLOR_VAR.value_or(ColorCss::MUTED.value(ColorShade::S4))
}

fn divider_line() -> String {
    DIVIDER_LINE.value()
}

/// The caller's `spacing` if they set one, else none.
fn divider_spacing() -> String {
    DIVIDER_SPACING_VAR.value_or(0)
}

static DIVIDER_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().per_size(DividerDefaults::size_sx)
        .flex_shrink("0")
        .border_width("0")
        .border_style("solid")
        .border_color(divider_color())
        .when(
            "vertical",
            sx().border_right(format!("{} solid {}", divider_line(), divider_color()))
                .align_self("stretch")
                .margin_left(divider_spacing())
                .margin_right(divider_spacing())
                .and(DividerDefaults::vertical_sx()),
        )
        .when(
            "horizontal",
            sx().border_bottom(format!("{} solid {}", divider_line(), divider_color()))
                .height(divider_line())
                .margin_top(divider_spacing())
                .margin_bottom(divider_spacing())
                .and(DividerDefaults::horizontal_sx()),
        )
        .when(
            "label",
            sx().display("flex")
                .align_items("center")
                .border("0")
                .color("muted.9")
                .selector(
                    "&::before, &::after",
                    sx().content("\"\"").flex("1").background(divider_color()),
                ),
        )
        .when(
            "horizontal && label",
            // "horizontal" is unconditional and also matches here; these two
            // out-specificity its border/height.
            sx().border_bottom("0")
                .height("auto")
                .width("100%")
                .selector("&::before, &::after", sx().height(divider_line())),
        )
        .when(
            "vertical && label",
            sx().flex_direction("column")
                .align_self("stretch")
                .selector("&::before, &::after", sx().width(divider_line())),
        )
        // Only with "label" - out-specificities its even-flex
        // ::before/::after above.
        .when(
            "label && label-start",
            sx().selector("&::before", sx().flex("0 0 10%"))
                .selector("&::after", sx().flex("1")),
        )
        .when(
            "label && label-end",
            sx().selector("&::before", sx().flex("1"))
                .selector("&::after", sx().flex("0 0 10%")),
        )
});

fn divider_variables(
    color: Option<&ThemeAwareValue>,
    spacing: Option<&ThemeAwareValue>,
) -> Variables {
    variables()
        .with(
            DIVIDER_COLOR_VAR,
            color.map(divider_color_value).and_then(|v| v.resolve(None)),
        )
        .with(
            DIVIDER_SPACING_VAR,
            spacing.and_then(|v| v.resolve(Some(SizeCss::SPACING))),
        )
}

static DIVIDER_LABEL_HORIZONTAL_SX: StaticSx = StaticSx::new(|| {
    sx().padding("0 12px")
        .white_space("nowrap")
        .user_select("none")
});

static DIVIDER_LABEL_VERTICAL_SX: StaticSx = StaticSx::new(|| {
    sx().padding("8px 0")
        .white_space("nowrap")
        .user_select("none")
});

parts_enum! {
    /// [`Divider`]'s inner parts, for its `parts` prop.
    pub enum DividerPart {
        /// The label between the two line halves, with `children` only.
        Label = "label" => "& > [data-slot='label']",
    }
}

base_props! {
    parts(DividerPart);
    pub struct DividerProps {
        /// `"horizontal"` (the default) or `"vertical"`.
        #[props(default, into)]
        orientation: Input<Orientation>,
        /// Line thickness. Defaults to `xs`.
        #[props(default, into)]
        size: Input<Size>,
        #[props(default, into)]
        label_position: Input<LabelPosition>,
        /// Margin on both sides of the line.
        #[props(default, into)]
        spacing: Input<ThemeAwareValue>,
        /// A bare color is tinted to its shade 3.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// An optional label, which also names the separator.
        children: Option<Element>,
    }
}

/// A horizontal or vertical separator line, optionally labelled.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::Divider;
/// # fn app() -> Element {
/// rsx! {
///     Divider {}
///     Divider { "or" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/layout/divider>
#[component]
pub fn Divider(props: DividerProps) -> Element {
    let orientation = props.orientation.copied_or(Orientation::Horizontal);
    let vertical = orientation == Orientation::Vertical;
    let has_label = props.children.is_some();
    let label_position = props.label_position.copied_or_default();
    let theme = use_theme();
    let size = props.size.copied_or(theme.divider.size);

    let divider_states = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with("vertical", vertical)
        .with("horizontal", !vertical)
        .with("label", has_label)
        .with("label-start", label_position == LabelPosition::Start)
        .with("label-end", label_position == LabelPosition::End);

    let variables: Input<Variables> =
        divider_variables(props.color.as_ref(), props.spacing.as_ref()).into();
    let data_state = divider_states.data_state();
    let aria_orientation = vertical.then_some("vertical");

    // A static class, not a component: `use_css` always, the span only with a label.
    let label_class = use_css(
        Some(if vertical {
            &DIVIDER_LABEL_VERTICAL_SX
        } else {
            &DIVIDER_LABEL_HORIZONTAL_SX
        }),
        CssLayer::Framework,
    );

    // A separator is named only by its author, never by its content. Not with a
    // caller's role: a global aria attribute would undo a `role: "none"`.
    let label_id = use_id();
    let caller_owns_it = props
        .attributes
        .iter()
        .any(|attribute| matches!(attribute.name, "role" | "aria-label" | "aria-labelledby"));
    let labelled_by = (has_label && !caller_owns_it).then_some(label_id.cloned());

    let label = match props.children {
        Some(children) => rsx! {
            span { "data-slot": DividerPart::Label.slot(), id: label_id, class: label_class, {children} }
        },
        None => rsx! {},
    };

    use_box()
        .framework_sx(&DIVIDER_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .variables(&variables)
        .prepare()
        // A default, so a caller's `role: "none"` makes the rule decorative.
        .attr_default("role", "separator")
        .attr("aria-labelledby", labelled_by)
        .attr("aria-orientation", aria_orientation)
        .attr("data-state", data_state)
        .render(HtmlTag::Div, props.attributes, label)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        components::common::part_table,
        tokens::{Color, Size},
    };

    #[test]
    fn a_bare_color_is_tinted_to_the_divider_shade() {
        let color = ThemeAwareValue::Color(Color::Primary);
        let variables = divider_variables(Some(&color), None);

        assert_eq!(
            variables.to_string(),
            format!(
                "{}:{};",
                DIVIDER_COLOR_VAR.name(),
                ColorValue::Shade(Color::Primary, ColorShade::S3).value()
            )
        );
    }

    /// Only a bare `Color` is tinted; an explicit shade passes through.
    #[test]
    fn an_explicit_color_value_is_left_alone() {
        let color = ThemeAwareValue::ColorValue(ColorValue::Shade(Color::Error, ColorShade::S9));
        let variables = divider_variables(Some(&color), None);

        assert_eq!(
            variables.to_string(),
            format!(
                "{}:{};",
                DIVIDER_COLOR_VAR.name(),
                ColorValue::Shade(Color::Error, ColorShade::S9).value()
            )
        );
    }

    #[test]
    fn spacing_resolves_through_the_spacing_scale() {
        let spacing = ThemeAwareValue::Size(Size::Lg);
        let variables = divider_variables(None, Some(&spacing));

        assert_eq!(
            variables.to_string(),
            format!(
                "{}:{};",
                DIVIDER_SPACING_VAR.name(),
                SizeCss::SPACING.value(Size::Lg)
            )
        );
    }

    #[test]
    fn neither_set_emits_nothing() {
        assert_eq!(divider_variables(None, None).to_string(), "");
    }

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<DividerPart>(),
            [("label", "& > [data-slot='label']")]
        );
    }
}
