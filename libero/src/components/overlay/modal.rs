use dioxus::prelude::*;

use crate::{
    components::{FocusTrap, Input, Overlay, States, common::class_list},
    context::use_sx,
    hooks::{ModalContext, use_modal_z_index},
    sx::{StaticSx, Sx, sx},
};

static MODAL_SX: StaticSx = StaticSx::new(|| {
    sx().position("fixed")
        .inset("0")
        .display("flex")
        .align_items("center")
        .justify_content("center")
});

static MODAL_CONTENT_SX: StaticSx = StaticSx::new(|| sx().position("relative").z_index("1"));

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
    #[props(default, into)]
    aria_label: Option<String>,
    #[props(default)]
    onclose: EventHandler<()>,
    children: Element,
}

/// A focus-trapped, dimmed dialog on top of the page. Locks page scroll and
/// stacks above earlier-opened modals while it exists. Callers control
/// whether it exists by conditionally rendering it - closing (Escape,
/// clicking the overlay) is requested via `onclose`, not by the modal
/// removing itself. Descendants can request a close of their own via
/// [`crate::hooks::use_modal_context`].
#[component]
pub fn Modal(props: ModalProps) -> Element {
    use_context_provider(|| ModalContext {
        onclose: props.onclose,
    });

    let z_index = use_modal_z_index();

    let framework_class = use_sx(&MODAL_SX, crate::SxLayer::Framework);
    let dynamic_class = use_sx(&sx().z_index(z_index), crate::SxLayer::UserDynamic);
    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| use_sx(sx, crate::SxLayer::UserStatic));
    let class = class_list([props.class, framework_class, dynamic_class, static_class]);
    let content_class = use_sx(&MODAL_CONTENT_SX, crate::SxLayer::Framework);

    let data_state = props.states.as_ref().and_then(States::data_state);

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
        div {
            role: "dialog",
            "aria-modal": "true",
            "aria-label": props.aria_label.clone(),
            "data-lsx-scroll-lock": true,
            class: class,
            "data-state": data_state,
            onkeydown: move |event: Event<KeyboardData>| {
                if event.key() == Key::Escape {
                    close();
                }
            },
            ..props.attributes,
            Overlay { z_index: 0, onclick: move |_| close() }
            div {
                class: content_class,
                FocusTrap { {props.children} }
            }
        }
    }
}
