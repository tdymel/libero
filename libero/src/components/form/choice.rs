//! What `Checkbox`, `Switch` and `Radio` share: a hidden input, the mark that carries
//! the click, and the focus ring around the mark.

use dioxus::prelude::*;

use crate::{
    components::{
        accessibility::{VISUALLY_HIDDEN_SX, hidden_input_centred_sx},
        common::{HtmlTag, Input, States, focus_ring_sx, ring_overlay, ring_overlay_sx},
        layout::{BoxStyle, use_box},
    },
    hooks::ElementHandle,
    platform::ElementApi,
    sx::{StaticSx, Sx, sx},
};

/// The control's rules past `vars`, the input's theme variables. Each input adds its
/// disabled look and its checked states.
pub(super) fn control_sx(vars: Sx) -> Sx {
    vars.display("inline-flex")
        .align_items("center")
        // The visually hidden input is absolutely positioned; without this it
        // escapes to the nearest positioned ancestor.
        .position("relative")
        .selector("& > input", hidden_input_centred_sx())
        // On the control rather than the mark, so `disabled` reaches it. Under `:hover`:
        // WebView spatial navigation makes a `pointer` element a Tab stop (3156).
        .selector("&:hover", sx().cursor("pointer"))
        // The ring hugs the mark, drawn by the overlay: the focus is on the input beside it.
        .selector("& > [data-ring]", ring_overlay_sx())
        .selector("& > input:focus-visible ~ [data-ring]", focus_ring_sx())
        // A card rings itself: the control stops being the overlay's
        // containing block, so the same overlay covers the card.
        .when("card", sx().position("static"))
}

/// A blank name names nothing: it neither renders nor counts as one.
pub(super) fn aria_label(label: &Option<String>) -> Option<String> {
    label.clone().filter(|label| !label.trim().is_empty())
}

/// The control's box, with its states and colour variables, and the hidden input's.
pub(super) fn use_boxes(
    control_sx: &'static StaticSx,
    states: &Input<States>,
    style: String,
) -> (BoxStyle, BoxStyle) {
    let control = use_box()
        .framework_sx(control_sx)
        .states(states)
        .style(Some(style).filter(|style| !style.is_empty()))
        .prepare();
    let input = use_box()
        .framework_sx(&VISUALLY_HIDDEN_SX)
        .focus_ring(false)
        .prepare();
    (control, input)
}

/// The mark's click. A card takes the click itself; otherwise this focuses the input
/// as a native click would, so its blur shows the rules.
pub(super) fn press(
    card: bool,
    disabled: bool,
    element: ElementHandle,
    act: impl Fn() + 'static,
) -> impl Fn() + 'static {
    move || {
        if !card {
            act();
            if !disabled {
                let _ = element.focus();
            }
        }
    }
}

/// The control slot: the hidden input, then `mark`, then the ring.
pub(super) fn control(
    control: BoxStyle,
    slot: &'static str,
    input: Element,
    mark: Element,
) -> Element {
    control.attr("data-slot", slot).render(
        HtmlTag::Span,
        Vec::new(),
        vec![input, mark, ring_overlay()],
    )
}
