use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        FocusTrap, HtmlTag, Input, Overlay, Variables, common::base_props, layout::use_box,
        variables,
    },
    context::ModalContext,
    hooks::{use_dismiss_layer, use_modal_z_index},
    platform::key_taken,
    sx::{StaticSx, sx},
    theme::CssVar,
};

const MODAL_Z_INDEX_VAR: CssVar = CssVar::new("--lsx-modal-z-index");

const SCROLL_LOCK_CSS: &str = "body { overflow: hidden; }";

static MODAL_SX: StaticSx = StaticSx::new(|| {
    sx().position("fixed")
        .inset("0")
        .z_index(MODAL_Z_INDEX_VAR.value())
});

// Fixed + inset so it spans the viewport: `Dialog`'s `margin:auto` needs real
// free space on this flex container to push against. `pointer-events:none` so
// clicks in the empty area reach the `Overlay` below; the content restores it.
// Goes on the `FocusTrap` itself rather than a wrapper - as an `sx` it lands in
// `lsx-user-static`, which beats the trap's own `display:contents`.
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
    // A `Modal` is only ever rendered while it is open, so mounting *is*
    // opening. Its Escape stays a bubbled subtree `onkeydown` rather than
    // moving to `KeyboardApi`: `platform::keyboard()` is `None` on every
    // backend but the web, so the capability would regress Escape natively.
    // Only the arbitration is shared.
    let layer = use_dismiss_layer();
    // The guard lives in this component's hook state, so the layer comes off
    // the stack when the modal unmounts, whatever path got it there. `Rc`
    // because `use_hook` clones what it stores on every render, and the guard
    // is deliberately not `Clone` - there is one of it, and it pops when it
    // dies.
    use_hook(move || Rc::new(layer.push()));
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

    // Stable identity, so `Overlay`'s props memoize - a closure here would be
    // a fresh listener attribute every render and re-render it.
    let on_backdrop_click = use_callback(move |_: MouseEvent| close());

    use_box()
        .framework_sx(&MODAL_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .variables(&variables)
        .prepare()
        .attr("data-lsx-scroll-lock", true)
        .event("onkeydown", move |event: Event<KeyboardData>| {
            // Only the top layer answers: a popover open inside this modal
            // hears the same press on its own box, and without the guard both
            // would close.
            //
            // The stack cannot see everything that takes Escape. On the web a
            // popover hears it at the document, and dioxus-web delivers this
            // handler only after that popover has closed and left the stack,
            // so `is_top()` alone answers yes. And a field dropdown - Select,
            // Cascader, the date and colour fields - is never on the stack at
            // all: it closes its list from a handler inside this modal and
            // lets the press bubble on. Both prevent the press's default when
            // they take it, and `key_taken` reads that on every backend.
            if event.key() == Key::Escape && layer.is_top() && !key_taken(&event) {
                close();
            }
        })
        .render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                // The scroll lock: in the document exactly while a modal is.
                // Not `body:has(..)` - `:has()` never matches natively.
                style { dangerous_inner_html: SCROLL_LOCK_CSS }
                Overlay { z_index: 0, onclick: on_backdrop_click }
                FocusTrap { sx: &MODAL_CONTENT_SX, {props.children} }
            },
        )
}
