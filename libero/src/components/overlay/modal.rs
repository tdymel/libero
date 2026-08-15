use dioxus::prelude::*;

use crate::{
    components::{Box, FocusTrap, Input, Overlay, States, common::class_list},
    context::ModalContext,
    hooks::{use_css, use_modal_z_index},
    sx::{StaticSx, Sx, sx},
};

static MODAL_SX: StaticSx = StaticSx::new(|| sx().position("fixed").inset("0"));

// Fixed + inset (not just relative) so it spans the full viewport itself:
// Dialog's start/end margin:auto trick needs real free space on this flex
// container to push against, not just Modal's root. pointer-events:none so
// clicks in its empty area (outside the actual content) fall through to
// Overlay beneath instead of being swallowed by this full-viewport div;
// the content itself restores pointer-events:auto.
static MODAL_CONTENT_SX: StaticSx = StaticSx::new(|| {
    sx().position("fixed")
        .inset("0")
        .z_index("1")
        .display("flex")
        .align_items("center")
        .justify_content("center")
        .pointer_events("none")
});

#[derive(Props, Clone, PartialEq)]
pub struct ModalProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    #[props(default)]
    onclose: EventHandler<()>,
    children: Element,
}

/// A focus-trapped, dimmed layer that locks scroll and stacks above earlier
/// modals. You control its existence by conditionally rendering it; closing
/// (Escape/backdrop) is requested via `onclose`, not self-removal. Descendants
/// close it via [`crate::hooks::use_modal_context`].
///
/// No opinion on content - pass `role`/`aria-modal`/`aria-label` yourself.
#[component]
pub fn Modal(props: ModalProps) -> Element {
    use_context_provider(|| ModalContext {
        onclose: props.onclose,
    });

    let z_index = use_modal_z_index();

    let dynamic_class = use_css(&sx().z_index(z_index), crate::CssLayer::UserDynamic);
    let class = class_list([props.class, dynamic_class]);

    // Deferred to the next microtask: closing synchronously (from an event
    // still bubbling through the modal being torn down) can re-enter the
    // same `EventHandler` and panic with `AlreadyBorrowedMut`.
    let onclose = props.onclose;
    let close = move || {
        spawn(async move {
            onclose.call(());
        });
    };

    rsx! {
        Box {
            "data-lsx-scroll-lock": true,
            class: class,
            sx: props.sx,
            states: props.states,
            framework_sx: &MODAL_SX,
            onkeydown: move |event: Event<KeyboardData>| {
                if event.key() == Key::Escape {
                    close();
                }
            },
            attributes: props.attributes,
            Overlay { z_index: 0, onclick: move |_| close() }
            Box {
                framework_sx: &MODAL_CONTENT_SX,
                FocusTrap { {props.children} }
            }
        }
    }
}
