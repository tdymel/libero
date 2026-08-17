use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, common::base_props},
    context::ModalContext,
    hooks::use_css,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::SizeCss,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogAlign {
    Start,
    Center,
    End,
}

impl Default for DialogAlign {
    fn default() -> Self {
        Self::Center
    }
}

impl DialogAlign {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
        }
    }
}

impl From<&str> for DialogAlign {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "start" => Self::Start,
            "end" => Self::End,
            _ => Self::Center,
        }
    }
}

impl From<String> for DialogAlign {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&str> for Input<DialogAlign> {
    fn from(value: &str) -> Self {
        Input::Value(DialogAlign::from(value))
    }
}

impl From<String> for Input<DialogAlign> {
    fn from(value: String) -> Self {
        Input::Value(DialogAlign::from(value))
    }
}

static DIALOG_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().background("white")
        // Modal's content wrapper is pointer-events:none so backdrop clicks
        // fall through it - restore interactivity for the dialog itself.
        .pointer_events("auto")
        // A flex item shrinks to its content by default, so without an
        // explicit width, `max-width`/`size` only caps rather than fills.
        .width("100%")
        .max_width(SizeCss::DIALOG_SIZE.value(crate::theme::Size::Md))
        .margin("md")
        .padding("lg")
        .border_radius("md")
        .box_shadow("0 12px 32px rgba(0, 0, 0, 0.25)")
});

/// `xs`-`xl` resolve through the dialog size scale, not the (much larger)
/// breakpoint scale `max-width` normally uses; anything else passes through.
pub(crate) fn dialog_size(value: &ThemeAwareValue) -> ThemeAwareValue {
    match value {
        ThemeAwareValue::Size(size) => SizeCss::DIALOG_SIZE.value(*size).into(),
        other => other.clone(),
    }
}

fn dialog_dynamic_sx(props: &DialogProps) -> Sx {
    let vertical = props.vertical.as_ref().copied().unwrap_or_default();
    let horizontal = props.horizontal.as_ref().copied().unwrap_or_default();

    sx().apply_if(props.radius.as_ref(), |sx, radius| {
        sx.border_radius(radius.clone())
    })
    .apply_if(props.size.as_ref().map(dialog_size), |sx, size| {
        sx.max_width(size)
    })
    .apply_if(props.z_index.as_ref(), |sx, z_index| {
        sx.z_index(z_index.clone())
    })
    .apply_if(
        match vertical {
            DialogAlign::Start => Some("auto"),
            _ => None,
        },
        |sx, auto| sx.margin_bottom(auto),
    )
    .apply_if(
        match vertical {
            DialogAlign::End => Some("auto"),
            _ => None,
        },
        |sx, auto| sx.margin_top(auto),
    )
    .apply_if(
        match horizontal {
            DialogAlign::Start => Some("auto"),
            _ => None,
        },
        |sx, auto| sx.margin_right(auto),
    )
    .apply_if(
        match horizontal {
            DialogAlign::End => Some("auto"),
            _ => None,
        },
        |sx, auto| sx.margin_left(auto),
    )
}

base_props! {
    pub struct DialogProps {
        #[props(default, into)]
        aria_label: Option<String>,
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        z_index: Input<ThemeAwareValue>,
        /// Cross-axis position - `Start`/`End` pin to top/bottom instead of
        /// Modal's default vertical center.
        #[props(default, into)]
        vertical: Input<DialogAlign>,
        /// Main-axis position - `Start`/`End` pin to left/right instead of
        /// the default horizontal center.
        #[props(default, into)]
        horizontal: Input<DialogAlign>,
        children: Element,
    }
}

/// Dialog surface: `role="dialog"`, plus `aria-modal="true"` when nested in
/// a [`crate::components::Modal`] (auto-detected, not passed explicitly).
#[component]
pub fn Dialog(props: DialogProps) -> Element {
    let is_modal = try_use_context::<ModalContext>().is_some();

    let dynamic_class = use_css(&dialog_dynamic_sx(&props), crate::CssLayer::UserDynamic);
    let class = props.class.unwrap_or_default().with(dynamic_class);

    rsx! {
        Box {
            role: "dialog",
            "aria-modal": is_modal.then_some("true"),
            "aria-label": props.aria_label.clone(),
            class: class,
            sx: props.sx,
            states: props.states,
            framework_sx: &DIALOG_BASE_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
