use dioxus::prelude::*;

use crate::{
    components::{
        ActionIcon, Box, Input, Paper, Title, Variables,
        common::{CloseIcon, attr, base_props},
        surface::paper_sx,
        variables,
    },
    context::ModalContext,
    hooks::use_id,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{CssVar, DIALOG_SIZE, PAPER_RADIUS, Size, SizeCss},
};

const DIALOG_RADIUS_VAR: CssVar = CssVar::new("--lsx-dialog-radius");

// A dialog is a `Paper`, so its chrome comes from `paper_sx()` - background,
// border and the themed radius - and this adds only what makes it a dialog.
// Both overrides below are plain declarations chained on top, which is what
// `paper_sx()` leaves room for: `Paper` emits a `data-state` token only for a
// step a caller names, and `Dialog` names none.
static DIALOG_BASE_SX: StaticSx = StaticSx::new(|| {
    paper_sx()
        // `Modal`'s wrapper is pointer-events:none so backdrop clicks fall
        // through; the dialog itself needs them back.
        .pointer_events("auto")
        // A flex item shrinks to content, so `size` would only cap, not fill.
        .width("100%")
        .max_width(DIALOG_SIZE.overridable(Size::Md))
        .margin("md")
        .padding("lg")
        .border_radius(DIALOG_RADIUS_VAR.value_or(PAPER_RADIUS.value()))
        // A dialog floats above everything, where the surface default rests.
        .box_shadow(SizeCss::SHADOW.value(Size::Xl))
});

static DIALOG_HEADER_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("flex-start")
        .justify_content("space-between")
        .gap("sm")
        .margin_bottom("md")
});

// Pushes a lone close button to the right, where a title would have left it.
static DIALOG_HEADER_TITLE_SX: StaticSx = StaticSx::new(|| sx().margin("0").flex("1"));

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
        /// Heading, and the accessible name unless `aria_label` overrides it.
        #[props(default, into)]
        title: Option<String>,
        /// Defaults to on inside a `Modal`, which is the only place it has
        /// something to close.
        #[props(default)]
        close_button: Option<bool>,
        /// Accessible name for the close button.
        #[props(default, into)]
        close_label: Option<String>,
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
/// modal (auto-detected). Inside one it also names itself from `title` and
/// closes itself from its own button, so a modal opened with
/// [`crate::hooks::use_modal`] needs no closing wiring. No positioning of its
/// own - anchor it with [`crate::components::Float`] or your own layout.
#[component]
pub fn Dialog(props: DialogProps) -> Element {
    let modal = try_use_context::<ModalContext>();
    let is_modal = modal.is_some();
    let close_button = props.close_button.unwrap_or(is_modal);
    let title_id = use_id();
    let variables: Input<Variables> = dialog_variables(&props)
        .merge(props.variables.unwrap_or_default())
        .into();

    // Pushed onto the caller's own, not handed to `Paper` as props: the
    // component's attributes have to render after the caller's to win a
    // duplicate name, which is the order `BoxStyle::render` keeps.
    let mut attributes = props.attributes;
    attributes.push(attr("role", "dialog"));
    if is_modal {
        attributes.push(attr("aria-modal", "true"));
    }
    if props.aria_label.is_none() && props.title.is_some() {
        attributes.push(attr("aria-labelledby", title_id()));
    }
    if let Some(aria_label) = props.aria_label.clone() {
        attributes.push(attr("aria-label", aria_label));
    }

    rsx! {
        Paper {
            framework_sx: &DIALOG_BASE_SX,
            class: props.class,
            sx: props.sx,
            states: props.states,
            variables,
            attributes,
            if props.title.is_some() || close_button {
                Box { framework_sx: &DIALOG_HEADER_SX,
                    if let Some(title) = props.title.clone() {
                        Title { id: title_id(), size: "xl", sx: &DIALOG_HEADER_TITLE_SX, "{title}" }
                    }
                    if close_button {
                        ActionIcon {
                            variant: "transparent",
                            color: "gray",
                            size: "sm",
                            aria_label: props.close_label.clone().unwrap_or_else(|| "Close".to_string()),
                            onclick: move |_| {
                                if let Some(modal) = modal {
                                    modal.close();
                                }
                            },
                            CloseIcon {}
                        }
                    }
                }
            }
            {props.children}
        }
    }
}
