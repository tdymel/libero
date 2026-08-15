use dioxus::prelude::*;

use crate::{
    components::{Box, FocusTrap, Input, Overlay, States, common::class_list},
    context::use_sx,
    hooks::{ModalContext, use_modal_z_index},
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

/// A focus-trapped, dimmed layer on top of the page. Locks page scroll and
/// stacks above earlier-opened modals while it exists. Callers control
/// whether it exists by conditionally rendering it - closing (Escape,
/// clicking the overlay) is requested via `onclose`, not by the modal
/// removing itself. Descendants can request a close of their own via
/// [`crate::hooks::use_modal_context`].
///
/// `Modal` is just the mechanism - it has no opinion on what's inside, so it
/// doesn't set `role`/`aria-modal`/`aria-label` itself. Pass those (e.g.
/// `role: "dialog"`, `"aria-modal": "true"`, `aria_label: "..."`) at the call
/// site to match what the content actually is.
#[component]
pub fn Modal(props: ModalProps) -> Element {
    use_context_provider(|| ModalContext {
        onclose: props.onclose,
    });

    let z_index = use_modal_z_index();

    let dynamic_class = use_sx(&sx().z_index(z_index), crate::SxLayer::UserDynamic);
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
