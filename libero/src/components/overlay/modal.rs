use std::rc::Rc;

use dioxus::prelude::*;

use super::use_modal::use_modal_z_index;
use crate::{
    components::{
        accessibility::FocusTrap,
        common::{HtmlTag, Input, Variables, base_props, variables},
        layout::use_box,
        overlay::Overlay,
    },
    context::ModalContext,
    hooks::{escape_closes, use_back, use_dismiss_layer},
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

// On the `FocusTrap` as an `sx`, to beat its `display: contents`. No pointer
// events, so clicks in the empty area reach the `Overlay`.
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
    pub(crate) struct ModalProps {
        #[props(default)]
        onclose: Option<EventHandler<()>>,
        children: Element,
    }
}

/// A focus-trapped, dimmed, scroll-locking layer. Crate-only: `use_modal` adds
/// focus return. Escape and backdrop only request a close via `onclose`.
#[component]
pub(crate) fn Modal(props: ModalProps) -> Element {
    // Through `use_callback`: the context is provided once, and the raw prop
    // would freeze descendants on the first render's closure.
    let onclose = props.onclose;
    let onclose = use_callback(move |()| {
        if let Some(onclose) = &onclose {
            onclose.call(());
        }
    });
    use_context_provider(|| ModalContext {
        onclose: Some(onclose),
    });

    let z_index = use_modal_z_index();
    // Escape is a subtree `onkeydown`, not `KeyboardApi`: `keyboard()` is
    // `None` on the WebView floor.
    let layer = use_dismiss_layer();
    // The guard pops the layer on unmount. `Rc`: `use_hook` clones, the guard is not `Clone`.
    use_hook(move || Rc::new(layer.push()));
    let variables: Input<Variables> = variables()
        .with(MODAL_Z_INDEX_VAR, z_index.to_string())
        .into();

    // Deferred: closing synchronously mid-bubble re-enters the same
    // `EventHandler` and panics with `AlreadyBorrowedMut`.
    let close = move || {
        spawn(async move {
            onclose.call(());
        });
    };

    // Stable identity, so `Overlay`'s props memoize.
    let on_backdrop_click = use_callback(move |_: MouseEvent| close());
    use_back(true, use_callback(move |()| close()));

    use_box()
        .framework_sx(&MODAL_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .variables(&variables)
        .prepare()
        .attr("data-lsx-scroll-lock", true)
        .event("onkeydown", move |event: Event<KeyboardData>| {
            // Only the top layer answers. `escape_closes` also skips a press a popover
            // or field dropdown already took (default prevented), and held repeats.
            if escape_closes(&event) && layer.is_top() {
                close();
            }
            // A Tab the trap left alone found nothing to focus: stay put
            // rather than walk out to the page behind.
            if event.key() == Key::Tab && !key_taken(&event) {
                event.prevent_default();
            }
        })
        .render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                // The scroll lock, mounted with the modal. Not `body:has(..)`:
                // `:has()` never matches natively.
                style { dangerous_inner_html: SCROLL_LOCK_CSS }
                Overlay { z_index: 0, onclick: on_backdrop_click }
                FocusTrap { sx: &MODAL_CONTENT_SX, {props.children} }
            },
        )
}
