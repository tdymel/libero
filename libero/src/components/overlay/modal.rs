use std::{cell::Cell, rc::Rc};

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
    hooks::{escape_closes, use_back, use_dismiss_layer, use_element},
    platform::{ElementApi, KeyChord, document, key_taken, keyboard, root_padding_right},
    sx::{StaticSx, sx},
    theme::CssVar,
};

const MODAL_Z_INDEX_VAR: CssVar = CssVar::new("--lsx-modal-z-index");

/// The scroll lock. A classic scrollbar's width is added to `:root`'s own `padding`, so the page
/// does not shift (todo 1305); not `scrollbar-gutter`, which no backdrop covers (todo 1316).
fn scroll_lock_css(gutter: f64, padding: f64) -> String {
    if gutter > 0.0 {
        let padding = padding + gutter;
        format!("html {{ padding-right: {padding}px !important; }} body {{ overflow: hidden; }}")
    } else {
        "body { overflow: hidden; }".to_string()
    }
}

/// The classic scrollbar's width: the viewport less the fixed, full-bleed root.
/// 0 for an overlay scrollbar, or a root not laid out yet.
fn scrollbar_gutter(viewport: Option<f64>, root: Option<f64>) -> f64 {
    match (viewport, root) {
        (Some(viewport), Some(root)) if root > 0.0 && (0.0..=64.0).contains(&(viewport - root)) => {
            viewport - root
        }
        _ => 0.0,
    }
}

static MODAL_SX: StaticSx = StaticSx::new(|| {
    sx().position("fixed")
        .inset("0")
        .z_index(MODAL_Z_INDEX_VAR.value())
});

// On the `FocusTrap` as an `sx`, to beat its `display: contents`. No pointer
// events, so clicks in the empty area reach the `Overlay`. Scrolls when the dialog
// outgrows the viewport; its auto margins centre it, and drop to 0 rather than clip the top.
static MODAL_CONTENT_SX: StaticSx = StaticSx::new(|| {
    sx().position("fixed")
        .inset("0")
        .z_index("1")
        .display("flex")
        .padding("md")
        .overflow_y("auto")
        .pointer_events("none")
        .selector("& > [role='dialog']", sx().margin("auto"))
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

    // Stable identity, so `Overlay`'s props memoize.
    let on_backdrop_click = use_callback(move |_: MouseEvent| close());
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
                let mut tick = stray_escape;
                let next = tick.peek().wrapping_add(1);
                tick.set(next);
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

    // Measured before the lock mounts: the scrollbar is gone after.
    let mut lock = use_signal(|| None::<(f64, f64)>);
    use_effect(move || {
        if !root.is_mounted() || lock.peek().is_some() {
            return;
        }
        let size = root.dimensions();
        let viewport = document().map(|document| document.viewport());
        let padding = root_padding_right().unwrap_or(0.0);
        spawn(async move {
            let width = size.await.ok().map(|size| size.width);
            let screen = match viewport {
                Some(read) => read.await.ok().map(|viewport| viewport.width),
                None => None,
            };
            lock.set(Some((scrollbar_gutter(screen, width), padding)));
        });
    });

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
            // A Tab the trap left alone found nothing to focus: stay put
            // rather than walk out to the page behind.
            if event.key() == Key::Tab && !key_taken(&event) {
                event.prevent_default();
            }
        })
        .render(
            HtmlTag::Div,
            attributes,
            rsx! {
                // The scroll lock, mounted with the modal. Not `body:has(..)`:
                // `:has()` never matches natively.
                if let Some((gutter, padding)) = lock() {
                    style { dangerous_inner_html: scroll_lock_css(gutter, padding) }
                }
                Overlay { z_index: 0, onclick: on_backdrop_click }
                FocusTrap { sx: &MODAL_CONTENT_SX, {props.children} }
            },
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_classic_scrollbar_keeps_its_gutter_under_the_lock() {
        assert_eq!(scrollbar_gutter(Some(1280.0), Some(1265.0)), 15.0);
        // On top of the page's own padding, which it must not replace.
        assert!(scroll_lock_css(15.0, 8.0).contains("html { padding-right: 23px !important; }"));
    }

    #[test]
    fn no_gutter_without_a_measured_classic_scrollbar() {
        // Overlay scrollbars, an unlaid-out root, no document, a nonsense reading.
        for (viewport, root) in [
            (Some(1280.0), Some(1280.0)),
            (Some(1280.0), Some(0.0)),
            (None, Some(1265.0)),
            (Some(1280.0), None),
            (Some(1280.0), Some(900.0)),
        ] {
            assert_eq!(
                scrollbar_gutter(viewport, root),
                0.0,
                "{viewport:?} {root:?}"
            );
        }
        assert_eq!(scroll_lock_css(0.0, 8.0), "body { overflow: hidden; }");
    }
}
