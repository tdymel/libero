use dioxus::prelude::*;

use crate::{
    components::{Input, States, common::class_list},
    sx::{StaticSx, Sx, sx},
};

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
    sx().margin("0")
        .flex_shrink("0")
        .border_width("0")
        .border_style("solid")
        .border_color("grey.3")
        .when(
            "vertical",
            sx().border_right("1px solid")
                .border_right_color("grey.3")
                .align_self("stretch"),
        )
        .when(
            "horizontal",
            sx().border_bottom("1px solid")
                .border_bottom_color("grey.3")
                .height("1px"),
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
                .margin("16px 0")
                .selector("&::before, &::after", sx().height("1px")),
        )
        .when(
            "vertical && label",
            sx().flex_direction("column")
                .align_self("stretch")
                .selector("&::before, &::after", sx().width("1px")),
        )
});

fn divider_dynamic_sx(label_position: LabelPosition) -> Sx {
    match label_position {
        LabelPosition::Start => sx()
            .selector("::before", sx().flex("0 0 10%"))
            .selector("::after", sx().flex("1")),
        LabelPosition::End => sx()
            .selector("::before", sx().flex("1"))
            .selector("::after", sx().flex("0 0 10%")),
        LabelPosition::Center => sx(),
    }
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
    children: Option<Element>,
}

#[component]
pub fn Divider(props: DividerProps) -> Element {
    let vertical = props.vertical.unwrap_or(false);
    let has_label = props.children.is_some();
    let label_position = props.label_position.as_ref().copied().unwrap_or_default();

    // Build states using the API
    let divider_states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("vertical", vertical)
        .with("horizontal", !vertical)
        .with("label", has_label);

    let framework_class = crate::context::use_sx(&DIVIDER_BASE_SX, crate::SxLayer::Framework);

    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| crate::context::use_sx(sx, crate::SxLayer::UserStatic));

    let dynamic_class = if has_label {
        crate::context::use_sx(
            &divider_dynamic_sx(label_position),
            crate::SxLayer::UserDynamic,
        )
    } else {
        None
    };

    let class = class_list([props.class, framework_class, dynamic_class, static_class]);
    let data_state = divider_states.data_state();

    rsx! {
        if has_label {
            div {
                class: class,
                "data-state": data_state,
                ..props.attributes,
                span {
                    class: crate::context::use_sx(
                        if vertical { &DIVIDER_LABEL_VERTICAL_SX } else { &DIVIDER_LABEL_HORIZONTAL_SX },
                        crate::SxLayer::Framework
                    ),
                    {props.children}
                }
            }
        } else {
            hr {
                class: class,
                "data-state": data_state,
                ..props.attributes,
            }
        }
    }
}
