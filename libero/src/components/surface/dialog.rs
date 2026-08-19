use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, Variables, common::base_props, variables},
    context::ModalContext,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{CssVar, DIALOG_SIZE, Size, SizeCss},
};

const DIALOG_RADIUS_VAR: CssVar = CssVar::new("--lsx-dialog-radius");

static DIALOG_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().background("white")
        // `Modal`'s wrapper is pointer-events:none so backdrop clicks fall
        // through; the dialog itself needs them back.
        .pointer_events("auto")
        // A flex item shrinks to content, so `size` would only cap, not fill.
        .width("100%")
        .max_width(DIALOG_SIZE.overridable(Size::Md))
        .margin("md")
        .padding("lg")
        .border_radius(DIALOG_RADIUS_VAR.value_or(SizeCss::RADIUS.value(Size::Md)))
        .box_shadow("0 12px 32px rgba(0, 0, 0, 0.25)")
});

fn dialog_variables(props: &DialogProps) -> Variables {
    variables()
        .with(
            DIALOG_RADIUS_VAR,
            props.radius.resolve(Some(SizeCss::RADIUS)),
        )
        .with(
            DIALOG_SIZE.override_var(),
            props.size.resolve(Some(DIALOG_SIZE)),
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
        /// Layered onto Dialog's own - e.g. `Drawer`'s anchor/size vars.
        #[props(default, into)]
        variables: Input<Variables>,
        children: Element,
    }
}

/// Dialog surface: `role="dialog"`, plus `aria-modal="true"` when nested in a
/// [`crate::components::Modal`] (auto-detected). No positioning of its own -
/// anchor it with [`crate::components::Float`] or your own layout.
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
