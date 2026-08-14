use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::{document, prelude::*};

use crate::{
    SxLayer,
    components::{Input, States, common::class_list},
    context::use_sx,
    sx::{StaticSx, Sx, sx},
};

use super::visually_hidden::VISUALLY_HIDDEN_SX;

// Mirrors the selector used by most vanilla-JS focus-trap implementations:
// anything natively focusable, or explicitly opted in via tabindex (but not
// opted out with tabindex="-1").
const FOCUSABLE_SELECTOR: &str = "a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex=\"-1\"])";

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

// `display: contents` keeps the wrapper out of layout entirely - it exists
// only so we have a DOM node to scope the id-based JS queries below to.
static FOCUS_TRAP_SX: StaticSx = StaticSx::new(|| sx().display("contents"));

// Both scripts below start with a real `await` on a macrotask boundary, not
// just a fire-and-forget `setTimeout(..., 0)`. Dioxus wraps every eval'd
// script in `(async function(){ <script>; dioxus.close(); })()` and calls it
// synchronously - so unless our own script actually suspends at an `await`,
// `dioxus.close()` (which signals back into the Dioxus/wasm runtime) fires
// synchronously too, on the same call stack as whatever triggered the eval
// (an effect, a keydown handler, ...). That reentrant signal is what was
// hanging the page - a bare `setTimeout` without `await` schedules work but
// doesn't stop `dioxus.close()` from firing immediately regardless. Actually
// awaiting the timeout defers the *entire* rest of the script, `dioxus.close()`
// included, to a genuinely fresh task.

fn focus_first(id: &str) {
    document::eval(&format!(
        r#"await new Promise(function(r) {{ setTimeout(r, 0); }});
        var root = document.getElementById("{id}");
        if (!root) return;
        var target = root.querySelector("[data-autofocus]");
        if (!target) target = root.querySelector({FOCUSABLE_SELECTOR:?});
        if (target) target.focus();"#
    ));
}

// Handles the actual Tab/Shift+Tab move ourselves (not just the wrap-around
// edges) so the whole thing stays synchronous on the Rust side: the keydown
// handler always calls `event.prevent_default()` immediately, then fires
// this off to do the real work, rather than awaiting a round trip to decide
// whether to prevent the browser's own default action (which - for a
// synchronous browser event like keydown - would already be too late).
fn cycle_focus(id: &str, backwards: bool) {
    document::eval(&format!(
        r#"await new Promise(function(r) {{ setTimeout(r, 0); }});
        var root = document.getElementById("{id}");
        if (!root) return;
        var items = Array.prototype.slice.call(root.querySelectorAll({FOCUSABLE_SELECTOR:?}));
        if (items.length === 0) return;
        var index = items.indexOf(document.activeElement);
        var next;
        if ({backwards}) {{
            next = index <= 0 ? items.length - 1 : index - 1;
        }} else {{
            next = index === -1 || index === items.length - 1 ? 0 : index + 1;
        }}
        items[next].focus();"#
    ));
}

#[derive(Props, Clone, PartialEq)]
pub struct FocusTrapProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    #[props(default = true)]
    active: bool,
    children: Element,
}

#[component]
pub fn FocusTrap(props: FocusTrapProps) -> Element {
    let id =
        use_signal(|| format!("lsx-focus-trap-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed)));

    let framework_class = use_sx(&FOCUS_TRAP_SX, SxLayer::Framework);
    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| use_sx(sx, SxLayer::UserStatic));
    let class = class_list([props.class, framework_class, static_class]);
    let data_state = props.states.as_ref().and_then(States::data_state);

    rsx! {
        div {
            id: "{id}",
            class: class,
            "data-state": data_state,
            onkeydown: move |event: Event<KeyboardData>| {
                if props.active && event.key() == Key::Tab {
                    event.prevent_default();
                    cycle_focus(&id(), event.modifiers().shift());
                }
            },
            ..props.attributes,
            // Fresh insertion each time `active` flips to `true` (Dioxus
            // removes/re-adds this node rather than patching it in place),
            // so `onmounted` naturally fires exactly once per activation -
            // no manual "did we already run this" bookkeeping needed.
            if props.active {
                span {
                    "aria-hidden": "true",
                    style: "display:none",
                    onmounted: move |_| focus_first(&id()),
                }
            }
            {props.children}
        }
    }
}

/// A visually-hidden, focusable placeholder that soaks up a [`FocusTrap`]'s
/// initial focus instead of the first real focusable descendant - useful
/// when you don't want anything inside the trap focused right away (e.g. a
/// modal that shouldn't auto-focus its first input). Once it loses focus, it
/// drops out of the tab order for good.
///
/// Named after Mantine's `FocusTrap.InitialFocus`, which Dioxus has no
/// equivalent namespaced-component syntax for.
#[component]
pub fn FocusTrapInitialFocus() -> Element {
    let mut used = use_signal(|| false);
    let framework_class = use_sx(&VISUALLY_HIDDEN_SX, SxLayer::Framework);

    rsx! {
        span {
            class: framework_class,
            tabindex: if used() { "-1" } else { "0" },
            "data-autofocus": true,
            onblur: move |_| used.set(true),
        }
    }
}
