use dioxus::prelude::*;

use crate::{
    components::{Box, FocusTrap, Input, Overlay, States, common::base_props, variables},
    context::ModalContext,
    hooks::use_modal_z_index,
    sx::{StaticSx, Sx, sx},
    theme::CssVar,
};

const MODAL_Z_INDEX_VAR: CssVar = CssVar::new("--lsx-modal-z-index");

static MODAL_SX: StaticSx = StaticSx::new(|| {
    sx().position("fixed")
        .inset("0")
        .z_index(MODAL_Z_INDEX_VAR.value())
});

// Fixed + inset so it spans the viewport: `Dialog`'s `margin:auto` needs real
// free space on this flex container to push against. `pointer-events:none` so
// clicks in the empty area reach the `Overlay` below; the content restores it.
static MODAL_CONTENT_SX: StaticSx = StaticSx::new(|| {
    sx().position("fixed")
        .inset("0")
        .z_index("1")
        .display("flex")
        .align_items("center")
        .justify_content("center")
        .pointer_events("none")
});

base_props! {
    pub struct ModalProps {
        #[props(default)]
        onclose: EventHandler<()>,
        children: Element,
    }
}

/// A focus-trapped, dimmed layer that locks scroll and stacks above earlier
/// modals. Render it conditionally - Escape/backdrop only *request* a close
/// via `onclose`. Descendants close it via
/// [`crate::hooks::use_modal_context`].
///
/// No opinion on content - pass `role`/`aria-modal`/`aria-label` yourself.
#[component]
pub fn Modal(props: ModalProps) -> Element {
    use_context_provider(|| ModalContext {
        onclose: props.onclose,
    });

    let z_index = use_modal_z_index();
    let variables = variables().with(MODAL_Z_INDEX_VAR, z_index.to_string());

    // Deferred a microtask: closing synchronously from an event still
    // bubbling through the torn-down modal re-enters the same `EventHandler`
    // and panics with `AlreadyBorrowedMut`.
    let onclose = props.onclose;
    let close = move || {
        spawn(async move {
            onclose.call(());
        });
    };

    rsx! {
        Box {
            "data-lsx-scroll-lock": true,
            class: props.class,
            sx: props.sx,
            states: props.states,
            variables,
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
