use dioxus::prelude::*;

use crate::{
    components::{Input, States, common::class_list},
    context::use_sx,
    hooks::ModalContext,
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
        // A flex item shrinks to its content by default, so without an
        // explicit width, `max-width`/`size` only caps rather than fills.
        .width("100%")
        .max_width(SizeCss::DIALOG_SIZE.value(crate::theme::Size::Md))
        .margin("md")
        .padding("lg")
        .border_radius("md")
        .box_shadow("0 12px 32px rgba(0, 0, 0, 0.25)")
});

/// `xs`/`sm`/`md`/`lg`/`xl` resolve through the dialog size scale (distinct
/// from the (much larger) breakpoint scale generic `max-width` normally
/// uses) - anything else (a raw length, percentage, CSS var) passes through.
fn dialog_size(value: &ThemeAwareValue) -> ThemeAwareValue {
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

#[derive(Props, Clone, PartialEq)]
pub struct DialogProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    #[props(default, into)]
    aria_label: Option<String>,
    #[props(default, into)]
    radius: Input<ThemeAwareValue>,
    #[props(default, into)]
    size: Input<ThemeAwareValue>,
    #[props(default, into)]
    z_index: Input<ThemeAwareValue>,
    /// Position within its container's cross axis - Modal centers its
    /// content by default, so `Start`/`End` pin the dialog to the
    /// top/bottom instead of the vertical center.
    #[props(default, into)]
    vertical: Input<DialogAlign>,
    /// Position within its container's main axis - `Start`/`End` pin the
    /// dialog to the left/right instead of the horizontal center.
    #[props(default, into)]
    horizontal: Input<DialogAlign>,
    children: Element,
}

/// Dialog surface: `role="dialog"` always, plus `aria-modal="true"` when
/// rendered inside a [`crate::components::Modal`] - detected via
/// [`crate::hooks::use_modal_context`]'s presence, not passed explicitly, so
/// a standalone (non-modal) `Dialog` correctly omits it.
#[component]
pub fn Dialog(props: DialogProps) -> Element {
    let is_modal = try_use_context::<ModalContext>().is_some();

    let framework_class = use_sx(&DIALOG_BASE_SX, crate::SxLayer::Framework);
    let dynamic_class = use_sx(&dialog_dynamic_sx(&props), crate::SxLayer::UserDynamic);
    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| use_sx(sx, crate::SxLayer::UserStatic));
    let class = class_list([props.class, framework_class, dynamic_class, static_class]);
    let data_state = props.states.as_ref().and_then(States::data_state);

    rsx! {
        div {
            role: "dialog",
            "aria-modal": is_modal.then_some("true"),
            "aria-label": props.aria_label.clone(),
            class: class,
            "data-state": data_state,
            ..props.attributes,
            {props.children}
        }
    }
}
