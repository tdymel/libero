use dioxus::prelude::*;

use crate::{
    components::{HtmlTag, Input, common::base_props, layout::use_box},
    hooks::{ElementHandle, use_element, use_local_state},
    platform::ElementApi,
    sx::{StaticSx, sx},
};

/// `FocusTrap` wraps arbitrary consumer children, so the tab order has to
/// exclude what a browser would never focus. The `:not(..)` tail drops
/// `hidden`/`inert`/`aria-hidden` elements *and their descendants* - without
/// it, Tab strands focus on something invisible. Deliberately still matched:
/// a visually-hidden-but-focusable element, which is what
/// [`FocusTrapInitialFocus`] is.
const FOCUSABLE_SELECTOR: &str = concat!(
    ":is(a[href], button:not([disabled]), textarea:not([disabled]), ",
    "input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex=\"-1\"]))",
    ":not([hidden], [inert], [aria-hidden=\"true\"], ",
    "[hidden] *, [inert] *, [aria-hidden=\"true\"] *)"
);

static FOCUS_TRAP_SX: StaticSx = StaticSx::new(|| sx().display("contents"));

/// The sr-only recipe, but `fixed` rather than `absolute`. `FocusTrap` is
/// `display: contents` and so cannot host an absolute span; unhosted, focusing
/// it scrolled the document 609px. A fixed box scrolls nothing into view.
static INITIAL_FOCUS_SX: StaticSx = StaticSx::new(|| {
    sx().position("fixed")
        .width("1px")
        .height("1px")
        .padding("0")
        .margin("-1px")
        .overflow("hidden")
        .clip_path("inset(50%)")
        .white_space("nowrap")
        .border_width("0")
});

fn focus_first(root: &ElementHandle) {
    let target = root
        .query_selector("[data-autofocus]")
        .or_else(|_| root.query_selector(FOCUSABLE_SELECTOR));
    if let Ok(target) = target {
        let _ = target.focus();
    }
}

/// `false` when there was nothing to focus, so the caller can leave Tab to
/// the browser rather than swallowing it and stranding focus.
fn cycle_focus(root: &ElementHandle, backwards: bool) -> bool {
    let Ok(items) = root.query_selector_all(FOCUSABLE_SELECTOR) else {
        return false;
    };
    if items.is_empty() {
        return false;
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
    items[next].focus().is_ok()
}

base_props! {
    pub struct FocusTrapProps {
        children: Element,
    }
}

#[component]
pub fn FocusTrap(props: FocusTrapProps) -> Element {
    let root = use_element();

    use_box()
        .framework_sx(&FOCUS_TRAP_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        // Not `.element(&root)`: the trap also has to focus its first target
        // once mounted, and both have to happen in the one `onmounted`.
        .event("onmounted", move |event: Event<MountedData>| {
            (root.mount())(event);
            focus_first(&root);
        })
        .event("onkeydown", move |event: Event<KeyboardData>| {
            if event.key() == Key::Tab && cycle_focus(&root, event.modifiers().shift()) {
                event.prevent_default();
            }
        })
        .render(HtmlTag::Div, props.attributes, props.children)
}

/// A hidden placeholder that soaks up initial focus, then leaves the tab
/// order once blurred. Mantine's `FocusTrap.InitialFocus` equivalent.
#[component]
pub fn FocusTrapInitialFocus() -> Element {
    let used = use_local_state(|| false);
    let mark_used = used.clone();

    use_box()
        .framework_sx(&INITIAL_FOCUS_SX)
        .prepare()
        .attr("tabindex", if used.get() { "-1" } else { "0" })
        .attr("data-autofocus", true)
        .event("onblur", move |_: Event<FocusData>| mark_used.set(true))
        .render(HtmlTag::Span, Vec::new(), ())
}
