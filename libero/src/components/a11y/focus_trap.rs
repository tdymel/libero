use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::{document, prelude::*};

use crate::{
    components::{Box, Input, States, common::base_props},
    sx::{StaticSx, Sx, sx},
};

use super::visually_hidden::VISUALLY_HIDDEN_SX;

const FOCUSABLE_SELECTOR: &str = "a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex=\"-1\"])";

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

static FOCUS_TRAP_SX: StaticSx = StaticSx::new(|| sx().display("contents"));

// `await` a macrotask first: calling document::eval synchronously from an
// effect/handler hangs the page (dioxus.close() reenters synchronously).
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

base_props! {
    pub struct FocusTrapProps {
        children: Element,
    }
}

#[component]
pub fn FocusTrap(props: FocusTrapProps) -> Element {
    let id = use_signal(|| format!("lsx-focus-trap-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed)));

    rsx! {
        Box {
            id: "{id}",
            class: props.class,
            sx: props.sx,
            states: props.states,
            framework_sx: &FOCUS_TRAP_SX,
            onkeydown: move |event: Event<KeyboardData>| {
                if event.key() == Key::Tab {
                    event.prevent_default();
                    cycle_focus(&id(), event.modifiers().shift());
                }
            },
            attributes: props.attributes,
            span {
                "aria-hidden": "true",
                style: "display:none",
                onmounted: move |_| focus_first(&id()),
            }
            {props.children}
        }
    }
}

/// A hidden placeholder that soaks up initial focus, then leaves the tab
/// order once blurred. Mantine's `FocusTrap.InitialFocus` equivalent.
#[component]
pub fn FocusTrapInitialFocus() -> Element {
    let mut used = use_signal(|| false);

    rsx! {
        Box {
            component: "span",
            framework_sx: &VISUALLY_HIDDEN_SX,
            tabindex: if used() { "-1" } else { "0" },
            "data-autofocus": true,
            onblur: move |_| used.set(true),
        }
    }
}
