use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, Variables, common::base_props, variables},
    context::ModalContext,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{Size, SizeCss},
};

const DIALOG_RADIUS_VAR: &str = "--lsx-dialog-radius";
const DIALOG_SIZE_VAR: &str = "--lsx-dialog-size-override";

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
}

base_props! {
    pub struct DialogProps {
        #[props(default, into)]
        aria_label: Option<String>,
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        /// Forwarded alongside Dialog's own - e.g. `Drawer` layers its
        /// own anchor/size variables onto Dialog's rendered surface.
        #[props(default, into)]
        variables: Input<Variables>,
        children: Element,
    }
}

/// Dialog surface: `role="dialog"`, plus `aria-modal="true"` when nested in
/// a [`crate::components::Modal`] (auto-detected, not passed explicitly).
/// Carries no positioning of its own - anchor it with [`crate::components::Float`]
/// or your own layout.
#[component]
pub fn Dialog(props: DialogProps) -> Element {
    let is_modal = try_use_context::<ModalContext>().is_some();
    let variables = dialog_variables(&props).merge(props.variables.unwrap_or_default());

    rsx! {
        Box {
            role: "dialog",
            "aria-modal": is_modal.then_some("true"),
            "aria-label": props.aria_label.clone(),
            class: props.class,
            sx: props.sx,
            states: props.states,
            variables,
            framework_sx: &DIALOG_BASE_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
