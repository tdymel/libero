use dioxus::prelude::*;

use crate::{
    components::{Box, Input, Orientation, States, Variables, common::base_props, variables},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{ColorCss, ColorShade, ColorValue, CssVar, DividerDefaults, SizeCss},
};

fn divider_color_value(value: &ThemeAwareValue) -> ThemeAwareValue {
    match value {
        ThemeAwareValue::Color(color) => {
            ThemeAwareValue::ColorValue(ColorValue::Shade(*color, ColorShade::S3))
        }
        other => other.clone(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LabelPosition {
    Start,
    #[default]
    Center,
    End,
}

impl From<&str> for LabelPosition {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "start" => Self::Start,
            "end" => Self::End,
            "center" => Self::Center,
            _ => Self::Center,
        }
    }
}

impl From<String> for LabelPosition {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&str> for Input<LabelPosition> {
    fn from(value: &str) -> Self {
        Input::Value(LabelPosition::from(value))
    }
}

impl From<String> for Input<LabelPosition> {
    fn from(value: String) -> Self {
        Input::Value(LabelPosition::from(value))
    }
}

const DIVIDER_COLOR_VAR: CssVar = CssVar::new("--lsx-divider-color");
const DIVIDER_SPACING_VAR: CssVar = CssVar::new("--lsx-divider-spacing");

/// The caller's `color` if they set one, else the theme's grey-4.
fn divider_color() -> String {
    DIVIDER_COLOR_VAR.value_or(ColorCss::GREY.value(ColorShade::S4))
}

/// The caller's `spacing` if they set one, else none.
fn divider_spacing() -> String {
    DIVIDER_SPACING_VAR.value_or(0)
}

static DIVIDER_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().flex_shrink("0")
        .border_width("0")
        .border_style("solid")
        .border_color(divider_color())
        .when(
            "vertical",
            sx().border_right(format!("1px solid {}", divider_color()))
                .align_self("stretch")
                .margin_left(divider_spacing())
                .margin_right(divider_spacing())
                .and(DividerDefaults::vertical_sx()),
        )
        .when(
            "horizontal",
            sx().border_bottom(format!("1px solid {}", divider_color()))
                .height("1px")
                .margin_top(divider_spacing())
                .margin_bottom(divider_spacing())
                .and(DividerDefaults::horizontal_sx()),
        )
        .when(
            "label",
            sx().display("flex")
                .align_items("center")
                .border("0")
                .color("grey.9")
                .selector(
                    "&::before, &::after",
                    sx().content("\"\"").flex("1").background(divider_color()),
                ),
        )
        .when(
            "horizontal && label",
            // "horizontal" is unconditional (see below), so it also matches here;
            // these two properties out-specificity and override its border/height.
            sx().border_bottom("0")
                .height("auto")
                .width("100%")
                .selector("&::before, &::after", sx().height("1px")),
        )
        .when(
            "vertical && label",
            sx().flex_direction("column")
                .align_self("stretch")
                .selector("&::before, &::after", sx().width("1px")),
        )
        // Only meaningful together with "label" - out-specificities the
        // "label" block's own even-flex ::before/::after above.
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

base_props! {
    pub struct DividerProps {
        /// `"horizontal"` (the default) or `"vertical"`.
        #[props(default, into)]
        orientation: Input<Orientation>,
        #[props(default, into)]
        label_position: Input<LabelPosition>,
        #[props(default, into)]
        spacing: Input<ThemeAwareValue>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        children: Option<Element>,
    }
}

#[component]
pub fn Divider(props: DividerProps) -> Element {
    let orientation = props.orientation.copied_or(Orientation::Horizontal);
    let vertical = orientation == Orientation::Vertical;
    let has_label = props.children.is_some();
    let label_position = props.label_position.copied_or_default();

    let divider_states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("vertical", vertical)
        .with("horizontal", !vertical)
        .with("label", has_label)
        .with("label-start", label_position == LabelPosition::Start)
        .with("label-end", label_position == LabelPosition::End);

    let variables = divider_variables(props.color.as_ref(), props.spacing.as_ref());
    let data_state = divider_states.data_state();
    let aria_orientation = vertical.then_some("vertical");

    rsx! {
        Box {
            class: props.class,
            sx: props.sx,
            variables,
            framework_sx: &DIVIDER_BASE_SX,
            role: "separator",
            "aria-orientation": aria_orientation,
            "data-state": data_state,
            attributes: props.attributes,
            if has_label {
                Box {
                    component: "span",
                    framework_sx: if vertical { &DIVIDER_LABEL_VERTICAL_SX } else { &DIVIDER_LABEL_HORIZONTAL_SX },
                    {props.children}
                }
            }
        }
    }
}
