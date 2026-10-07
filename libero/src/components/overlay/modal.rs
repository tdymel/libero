use std::{cell::Cell, rc::Rc};

use dioxus::prelude::*;

use super::use_modal::use_modal_z_index;
use crate::{
    CssLayer,
    components::{
        accessibility::FocusTrap,
        common::{FOCUSABLE_SELECTOR, HtmlTag, Input, Variables, base_props, variables},
        layout::use_box,
        overlay::Overlay,
    },
    context::ModalContext,
    hooks::{escape_closes, use_back, use_css, use_dismiss_layer, use_element, use_scroll_lock},
    platform::{ElementApi, KeyChord, key_taken, keyboard},
    sx::{StaticSx, sx},
    theme::CssVar,
};

const MODAL_Z_INDEX_VAR: CssVar = CssVar::new("--lsx-modal-z-index");

static MODAL_SX: StaticSx = StaticSx::new(|| {
    sx().position("fixed")
        .inset("0")
        .z_index(MODAL_Z_INDEX_VAR.value())
});

// On the `FocusTrap` as an `sx`, to beat its `display: contents`. Scrolls when the
// dialog outgrows the viewport; it takes pointer events, so its scrollbar works (2560).
static MODAL_CONTENT_SX: StaticSx = StaticSx::new(|| {
    sx().position("fixed")
        .inset("0")
        .z_index("1")
        .display("block")
        .overflow_y("auto")
});

// Grows with the dialog; its auto margins centre it, and drop to 0 rather than clip the top.
// No pointer events: only a `Dialog` opts back in, the rest falls through to the hit area.
static MODAL_FRAME_SX: StaticSx = StaticSx::new(|| {
    sx().position("relative")
        .isolation("isolate")
        .display("flex")
        .box_sizing("border-box")
        .min_height("100%")
        .padding("md")
        .pointer_events("none")
        .selector("& > [role='dialog']", sx().margin("auto"))
});

// The backdrop click target, behind the content: inside the scroller, so the wheel over
// it scrolls, and a press on the scrollbar never lands on it.
static MODAL_HIT_AREA_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .inset("0")
        .z_index("-1")
        .pointer_events("auto")
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
    // `None` on the WebView floor. The document hears only presses from outside.
    let layer = use_dismiss_layer();
    let root = use_element();
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

    // Plain divs, not `Box`: a `Box` re-renders and counts against the perf budgets (2594).
    let frame_class = use_css(Some(&MODAL_FRAME_SX), CssLayer::Framework);
    let hit_area_class = use_css(Some(&MODAL_HIT_AREA_SX), CssLayer::Framework);
    use_back(true, use_callback(move |()| close()));

    // Escape from outside the dialog: the focused control was removed and focus
    // fell to `<body>` (todo 1304). Recorded only; the effect closes.
    let stray_escape = use_signal(|| 0u64);
    use_hook(move || {
        Rc::new(keyboard().map(|api| {
            api.on_key_unfiltered(Box::new(move |chord: KeyChord| {
                let outside = root
                    .try_mounted()
                    .is_some_and(|mounted| !chord.within(&mounted, root.tag()));
                if chord.key != Key::Escape || chord.repeat || !outside || !layer.is_top() {
                    return false;
                }
                crate::utils::bump(stray_escape);
                true
            }))
        }))
    });
    let seen = use_hook(|| Rc::new(Cell::new(0u64)));
    use_effect(move || {
        let tick = stray_escape();
        if tick != seen.replace(tick) {
            close();
        }
    });

    let scroll_lock = use_scroll_lock(root, true);

    let mut attributes = props.attributes;
    attributes.extend(root.attributes());

    use_box()
        .framework_sx(&MODAL_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .variables(&variables)
        .prepare()
        .attr("data-lsx-scroll-lock", true)
        .event("onmounted", root.mount())
        .event("onkeydown", move |event: Event<KeyboardData>| {
            // Only the top layer answers. `escape_closes` also skips a press a popover
            // or field dropdown already took (default prevented), and held repeats.
            if escape_closes(&event) && layer.is_top() {
                close();
            }
            // A Tab the trap left alone found nothing to focus: stay put rather than
            // walk out to the page behind. A WebView queries nothing, so Tab stays native.
            if event.key() == Key::Tab
                && !key_taken(&event)
                && root.query_selector_all(FOCUSABLE_SELECTOR).is_ok()
            {
                event.prevent_default();
            }
        })
        .render(
            HtmlTag::Div,
            attributes,
            rsx! {
                {scroll_lock}
                Overlay { z_index: 0 }
                FocusTrap { sx: &MODAL_CONTENT_SX,
                    div { class: frame_class,
                        div { class: hit_area_class, onclick: move |_| close() }
                        {props.children}
                    }
                }
            },
        )
}
