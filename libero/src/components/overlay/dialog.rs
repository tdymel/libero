use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, Variables, common::base_props, variables},
    context::ModalContext,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{Size, SizeCss},
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

const DIALOG_RADIUS_VAR: &str = "--lsx-dialog-radius";
const DIALOG_SIZE_VAR: &str = "--lsx-dialog-size-override";
const DIALOG_Z_INDEX_VAR: &str = "--lsx-dialog-z-index";

static DIALOG_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().background("white")
        // Modal's content wrapper is pointer-events:none so backdrop clicks
        // fall through it - restore interactivity for the dialog itself.
        .pointer_events("auto")
        // A flex item shrinks to its content by default, so without an
        // explicit width, `max-width`/`size` only caps rather than fills.
        .width("100%")
        .max_width(format!(
            "var({DIALOG_SIZE_VAR}, {})",
            SizeCss::DIALOG_SIZE.value(Size::Md)
        ))
        .margin("md")
        .padding("lg")
        .border_radius(format!(
            "var({DIALOG_RADIUS_VAR}, {})",
            SizeCss::RADIUS.value(Size::Md)
        ))
        .box_shadow("0 12px 32px rgba(0, 0, 0, 0.25)")
        .z_index(format!("var({DIALOG_Z_INDEX_VAR}, auto)"))
        .when("vertical-start", sx().margin_bottom("auto"))
        .when("vertical-end", sx().margin_top("auto"))
        .when("horizontal-start", sx().margin_right("auto"))
        .when("horizontal-end", sx().margin_left("auto"))
});

/// `xs`-`xl` resolve through the dialog size scale, not the (much larger)
/// breakpoint scale `max-width` normally uses; anything else passes through.
pub(crate) fn dialog_size(value: &ThemeAwareValue) -> ThemeAwareValue {
    match value {
        ThemeAwareValue::Size(size) => SizeCss::DIALOG_SIZE.value(*size).into(),
        other => other.clone(),
    }
}

fn dialog_variables(props: &DialogProps) -> Variables {
    variables()
        .with(
            DIALOG_RADIUS_VAR,
            props.radius.as_ref().and_then(ThemeAwareValue::radius),
        )
        .with(
            DIALOG_SIZE_VAR,
            props.size.as_ref().map(dialog_size).and_then(|v| v.raw()),
        )
        .with(
            DIALOG_Z_INDEX_VAR,
            props.z_index.as_ref().and_then(ThemeAwareValue::raw),
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
        /// Forwarded alongside Dialog's own - e.g. `Drawer` layers its
        /// own anchor/size variables onto Dialog's rendered surface.
        #[props(default, into)]
        variables: Input<Variables>,
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
    let vertical = props.vertical.as_ref().copied().unwrap_or_default();
    let horizontal = props.horizontal.as_ref().copied().unwrap_or_default();
    let variables = dialog_variables(&props).merge(props.variables.unwrap_or_default());

    let states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("vertical-start", vertical == DialogAlign::Start)
        .with("vertical-end", vertical == DialogAlign::End)
        .with("horizontal-start", horizontal == DialogAlign::Start)
        .with("horizontal-end", horizontal == DialogAlign::End);

    rsx! {
        Box {
            role: "dialog",
            "aria-modal": is_modal.then_some("true"),
            "aria-label": props.aria_label.clone(),
            class: props.class,
            sx: props.sx,
            states,
            variables,
            framework_sx: &DIALOG_BASE_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
