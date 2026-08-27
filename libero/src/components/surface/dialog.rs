use dioxus::prelude::*;

use crate::{
    components::{
        ActionIcon, Box, HtmlTag, Input, Title, Variables, common::base_props, layout::use_box,
        variables,
    },
    context::ModalContext,
    hooks::use_id,
    sx::{StaticSx, ThemeAwareValue, sx},
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

fn close_icon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            width: "16px",
            height: "16px",
            path { d: "M18 6 6 18" }
            path { d: "m6 6 12 12" }
        }
    }
}

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

    use_box()
        .framework_sx(&DIALOG_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .variables(&variables)
        .prepare()
        .attr("role", "dialog")
        .attr("aria-modal", is_modal.then_some("true"))
        .attr(
            "aria-labelledby",
            (props.aria_label.is_none() && props.title.is_some()).then(&*title_id),
        )
        .attr("aria-label", props.aria_label.clone())
        .render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
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
                                {close_icon()}
                            }
                        }
                    }
                }
                {props.children}
            },
        )
}
