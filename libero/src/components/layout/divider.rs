use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, common::class_list},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{ColorShade, ColorValue, DividerDefaults},
};

fn divider_color_value(value: &ThemeAwareValue) -> ThemeAwareValue {
    match value {
        ThemeAwareValue::Color(color) => {
            ThemeAwareValue::ColorValue(ColorValue::Shade(*color, ColorShade::S3))
        }
        other => other.clone(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelPosition {
    Start,
    Center,
    End,
}

impl Default for LabelPosition {
    fn default() -> Self {
        Self::Center
    }
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

static DIVIDER_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().flex_shrink("0")
        .border_width("0")
        .border_style("solid")
        .border_color("grey.3")
        .when(
            "vertical",
            sx().border_right("1px solid")
                .border_right_color("grey.3")
                .align_self("stretch")
                .and(DividerDefaults::vertical_sx()),
        )
        .when(
            "horizontal",
            sx().border_bottom("1px solid")
                .border_bottom_color("grey.3")
                .height("1px")
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
                    sx().content("\"\"").flex("1").background("grey.3"),
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
});

fn divider_dynamic_sx(
    label_position: LabelPosition,
    has_label: bool,
    vertical: bool,
    spacing: Option<&ThemeAwareValue>,
    color: Option<&ThemeAwareValue>,
) -> Sx {
    let position_sx = if has_label {
        match label_position {
            LabelPosition::Start => sx()
                .selector("::before", sx().flex("0 0 10%"))
                .selector("::after", sx().flex("1")),
            LabelPosition::End => sx()
                .selector("::before", sx().flex("1"))
                .selector("::after", sx().flex("0 0 10%")),
            LabelPosition::Center => sx(),
        }
    } else {
        sx()
    };

    let position_sx = position_sx.apply_if(spacing, |sx, spacing| {
        if vertical {
            sx.margin_left(spacing.clone())
                .margin_right(spacing.clone())
        } else {
            sx.margin_top(spacing.clone())
                .margin_bottom(spacing.clone())
        }
    });

    position_sx.apply_if(color, |base, color| {
        base.border_color(color.clone())
            .border_right_color(color.clone())
            .border_bottom_color(color.clone())
            .selector("&::before, &::after", sx().background(color.clone()))
    })
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

#[derive(Props, Clone, PartialEq)]
pub struct DividerProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    #[props(default)]
    vertical: Option<bool>,
    #[props(default, into)]
    label_position: Input<LabelPosition>,
    #[props(default, into)]
    spacing: Input<ThemeAwareValue>,
    #[props(default, into)]
    color: Input<ThemeAwareValue>,
    children: Option<Element>,
}

#[component]
pub fn Divider(props: DividerProps) -> Element {
    let vertical = props.vertical.unwrap_or(false);
    let has_label = props.children.is_some();
    let label_position = props.label_position.as_ref().copied().unwrap_or_default();

    let divider_states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("vertical", vertical)
        .with("horizontal", !vertical)
        .with("label", has_label);

    let color = props.color.as_ref().map(divider_color_value);

    let dynamic_class = crate::hooks::use_css(
        &divider_dynamic_sx(
            label_position,
            has_label,
            vertical,
            props.spacing.as_ref(),
            color.as_ref(),
        ),
        crate::CssLayer::UserDynamic,
    );

    let class = class_list([props.class, dynamic_class]);
    let data_state = divider_states.data_state();
    let aria_orientation = vertical.then_some("vertical");

    rsx! {
        Box {
            class: class,
            sx: props.sx,
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
