use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, common::base_props, common::dom_api},
    sx::{StaticSx, Sx, sx},
};

use super::visually_hidden::VISUALLY_HIDDEN_SX;

const FOCUSABLE_SELECTOR: &str = "a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex=\"-1\"])";

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

static FOCUS_TRAP_SX: StaticSx = StaticSx::new(|| sx().display("contents"));

fn focus_first(id: &str) {
    let Ok(root) = dom_api().query_selector(&format!("#{id}")) else {
        return;
    };
    let target = root
        .query_selector("[data-autofocus]")
        .or_else(|_| root.query_selector(FOCUSABLE_SELECTOR));
    if let Ok(target) = target {
        let _ = target.focus();
    }
}

fn cycle_focus(id: &str, backwards: bool) {
    let Ok(root) = dom_api().query_selector(&format!("#{id}")) else {
        return;
    };
    let Ok(items) = root.query_selector_all(FOCUSABLE_SELECTOR) else {
        return;
    };
    if items.is_empty() {
        return;
    }

    let index = items.iter().position(|item| item.is_focused());
    let next = match index {
        None => 0,
        Some(i) if backwards => {
            if i == 0 {
                items.len() - 1
            } else {
                i - 1
            }
        }
        Some(i) => {
            if i + 1 == items.len() {
                0
            } else {
                i + 1
            }
        }
    };
    let _ = items[next].focus();
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
