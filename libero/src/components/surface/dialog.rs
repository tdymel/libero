use dioxus::prelude::*;

use crate::{
    components::{
        ActionIcon, Box, HtmlTag, Input, Title, Variables,
        common::{CloseIcon, attr, base_props, names_itself, use_name_warning},
        layout::use_box,
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
        // Not `space-between`: that puts a lone child at the start, so a close
        // button without a title sat top-left.
        .justify_content("flex-end")
        .gap("sm")
        .margin_bottom("md")
});

// Takes the free space, so the title starts at the left and the close button
// stays at the right.
static DIALOG_HEADER_TITLE_SX: StaticSx = StaticSx::new(|| sx().margin("0").flex("1"));

fn dialog_variables(props: &DialogProps) -> Variables {
    variables()
        .with(
            DIALOG_RADIUS_VAR,
            props
                .radius
                .as_ref()
                .map(|&radius| SizeCss::RADIUS.value(radius)),
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
        radius: Input<Size>,
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
    use_name_warning(
        props.aria_label.is_some() || props.title.is_some() || names_itself(&props.attributes),
        "Dialog: no `title`, `aria_label` or `aria-labelledby`, so it is announced as just \
         \"dialog\".",
    );
    let variables: Input<Variables> = dialog_variables(&props)
        .merge(props.variables.unwrap_or_default())
        .into();

    // Pushed onto the caller's own: the component's attributes have to render
    // after the caller's to win a duplicate name.
    let mut attributes = props.attributes;
    attributes.push(attr("role", "dialog"));
    if is_modal {
        attributes.push(attr("aria-modal", "true"));
        // Focusable by script and by a click on its text, so focus and the
        // modal's keys stay inside when nothing in it takes focus (APG).
        attributes.push(attr("tabindex", "-1"));
    }
    if props.aria_label.is_none() && props.title.is_some() {
        attributes.push(attr("aria-labelledby", title_id()));
    }
    if let Some(aria_label) = props.aria_label.clone() {
        attributes.push(attr("aria-label", aria_label));
    }

    let mut children = Vec::with_capacity(2);
    if props.title.is_some() || close_button {
        children.push(rsx! {
            DialogHeader {
                title: props.title,
                title_id,
                close_button,
                close_label: props.close_label,
            }
        });
    }
    children.push(props.children);

    // `Paper`'s body without its scope: a `Paper` taking `children` would
    // re-render on every render of the dialog. Dialog sets no radius or shadow
    // step, so `Paper` would pass `states` through unchanged.
    use_box()
        .framework_sx(&DIALOG_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .variables(&variables)
        .prepare()
        .render(HtmlTag::Div, attributes, children)
}

/// The title and close button, in a scope of their own: its props compare
/// equal, so a dialog re-rendered for its `children` skips the header.
#[component]
fn DialogHeader(
    title: Option<String>,
    title_id: Signal<String>,
    close_button: bool,
    close_label: Option<String>,
) -> Element {
    let modal = try_use_context::<ModalContext>();
    let close = use_callback(move |_: MouseEvent| {
        if let Some(modal) = modal {
            modal.close();
        }
    });

    rsx! {
        Box { framework_sx: &DIALOG_HEADER_SX,
            if let Some(title) = title {
                Title { id: title_id(), size: "xl", sx: &DIALOG_HEADER_TITLE_SX, "{title}" }
            }
            if close_button {
                ActionIcon {
                    variant: "standard",
                    color: "muted",
                    size: "sm",
                    aria_label: close_label.unwrap_or_else(|| "Close".to_string()),
                    onclick: close,
                    CloseIcon {}
                }
            }
        }
    }
}
