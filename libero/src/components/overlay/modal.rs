use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        FocusTrap, HtmlTag, Input, Overlay, States, Variables, common::base_props, layout::use_box,
        variables,
    },
    context::ModalContext,
    hooks::{use_css, use_modal_z_index},
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
        onclose: Option<EventHandler<()>>,
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
    // Forwarded through `use_callback`, not stored directly: a context
    // provider runs once, but rsx builds a fresh `EventHandler` every render,
    // so storing the prop would freeze descendants on the closure - and the
    // values it captured - from mount. `use_callback` swaps its inner closure
    // each render behind a handle stable enough to provide once.
    let onclose = props.onclose;
    let onclose = use_callback(move |()| {
        if let Some(onclose) = &onclose {
            onclose.call(());
        }
    });
    use_context_provider(|| ModalContext { onclose });

    let z_index = use_modal_z_index();
    let variables: Input<Variables> = variables()
        .with(MODAL_Z_INDEX_VAR, z_index.to_string())
        .into();

    // Deferred a microtask: closing synchronously from an event still
    // bubbling through the torn-down modal re-enters the same `EventHandler`
    // and panics with `AlreadyBorrowedMut`.
    let close = move || {
        spawn(async move {
            onclose.call(());
        });
    };

    // Static styling only, so a class rather than a component scope.
    let content_class = use_css(Some(&MODAL_CONTENT_SX), CssLayer::Framework);

    use_box()
        .framework_sx(&MODAL_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .variables(&variables)
        .prepare()
        .attr("data-lsx-scroll-lock", true)
        .event("onkeydown", move |event: Event<KeyboardData>| {
            if event.key() == Key::Escape {
                close();
            }
        })
        .render(
            HtmlTag::Div,
            props.attributes,
            vec![
                rsx! { Overlay { z_index: 0, onclick: move |_| close() } },
                rsx! {
                    div { class: content_class,
                        FocusTrap { {props.children} }
                    }
                },
            ],
        )
}
