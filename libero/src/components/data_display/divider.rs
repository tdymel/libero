use dioxus::prelude::*;

use crate::{
    components::{Input, States, common::class_list},
    sx::{StaticSx, Sx, sx},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelPosition {
    Left,
    Center,
    Right,
}

impl Default for LabelPosition {
    fn default() -> Self {
        Self::Center
    }
}

impl From<&str> for LabelPosition {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "left" => Self::Left,
            "right" => Self::Right,
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

static DIVIDER_HORIZONTAL_NO_LABEL_SX: StaticSx = StaticSx::new(|| {
    sx().margin("0")
        .flex_shrink("0")
        .border_width("0")
        .border_style("solid")
        .border_color("#e0e0e0")
        .with("border-bottom-width", "1px")
});

static DIVIDER_VERTICAL_NO_LABEL_SX: StaticSx = StaticSx::new(|| {
    sx().margin("0")
        .flex_shrink("0")
        .height("100%")
        .border_width("0")
        .border_style("solid")
        .border_color("#e0e0e0")
        .with("border-right-width", "1px")
});

static DIVIDER_HORIZONTAL_WITH_LABEL_SX: StaticSx = StaticSx::new(|| {
    let line_sx = sx()
        .content("\"\"")
        .flex("1")
        .height("1px")
        .with("background", "#dcdcdc");

    sx().display("flex")
        .align_items("center")
        .width("100%")
        .with("color", "#6b7280")
        .margin("16px 0")
        .selector("::before", line_sx.clone())
        .selector("::after", line_sx)
});

static DIVIDER_VERTICAL_WITH_LABEL_SX: StaticSx = StaticSx::new(|| {
    let line_sx = sx()
        .content("\"\"")
        .flex("1")
        .width("1px")
        .with("background", "#dcdcdc");

    sx().display("flex")
        .flex_direction("column")
        .align_items("center")
        .height("100%")
        .with("color", "#6b7280")
        .margin("0 16px")
        .selector("::before", line_sx.clone())
        .selector("::after", line_sx)
});

fn divider_dynamic_sx(vertical: bool, label_position: LabelPosition) -> Sx {
    if vertical {
        match label_position {
            LabelPosition::Left => sx().selector("::before", sx().flex("0").with("height", "16px")),
            LabelPosition::Right => sx().selector("::after", sx().flex("0").with("height", "16px")),
            LabelPosition::Center => sx(),
        }
    } else {
        match label_position {
            LabelPosition::Left => sx().selector("::before", sx().flex("0").with("width", "16px")),
            LabelPosition::Right => sx().selector("::after", sx().flex("0").with("width", "16px")),
            LabelPosition::Center => sx(),
        }
    }
}

static DIVIDER_LABEL_SX: StaticSx =
    StaticSx::new(|| sx().padding("0 12px").with("white-space", "nowrap"));

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
    children: Option<Element>,
}

#[component]
pub fn Divider(props: DividerProps) -> Element {
    let vertical = props.vertical.unwrap_or(false);
    let has_label = props.children.is_some();
    let label_position = props.label_position.as_ref().copied().unwrap_or_default();

    let divider_base_sx = match (vertical, has_label) {
        (false, false) => &DIVIDER_HORIZONTAL_NO_LABEL_SX,
        (true, false) => &DIVIDER_VERTICAL_NO_LABEL_SX,
        (false, true) => &DIVIDER_HORIZONTAL_WITH_LABEL_SX,
        (true, true) => &DIVIDER_VERTICAL_WITH_LABEL_SX,
    };

    let framework_class = crate::context::use_sx(divider_base_sx, crate::SxLayer::Framework);

    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| crate::context::use_sx(sx, crate::SxLayer::UserStatic));

    let dynamic_class = if has_label {
        crate::context::use_sx(
            &divider_dynamic_sx(vertical, label_position),
            crate::SxLayer::UserDynamic,
        )
    } else {
        None
    };

    let class = class_list([props.class, framework_class, dynamic_class, static_class]);
    let data_state = props.states.as_ref().and_then(States::data_state);

    if vertical {
        rsx! {
            div {
                class: class,
                "data-state": data_state,
                ..props.attributes,
                if has_label {
                    span {
                        class: crate::context::use_sx(&DIVIDER_LABEL_SX, crate::SxLayer::Framework),
                        {props.children}
                    }
                }
            }
        }
    } else if has_label {
        rsx! {
            div {
                class: class,
                "data-state": data_state,
                ..props.attributes,
                span {
                    class: crate::context::use_sx(&DIVIDER_LABEL_SX, crate::SxLayer::Framework),
                    {props.children}
                }
            }
        }
    } else {
        rsx! {
            hr {
                class: class,
                "data-state": data_state,
                ..props.attributes,
            }
        }
    }
}
