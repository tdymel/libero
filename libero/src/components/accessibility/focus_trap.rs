use dioxus::prelude::*;

use super::visually_hidden::VISUALLY_HIDDEN_FIXED_SX;
use crate::{
    components::{
        common::{FOCUSABLE_SELECTOR, HtmlTag, Input, base_props},
        layout::use_box,
    },
    hooks::{ElementHandle, use_element, use_local_state},
    platform::{ElementApi, key_taken},
    sx::{StaticSx, sx},
};

static FOCUS_TRAP_SX: StaticSx = StaticSx::new(|| sx().display("contents"));

fn focus_first(root: &ElementHandle) {
    if let Ok(target) = root.query_selector("[data-autofocus]") {
        let _ = target.focus();
        return;
    }
    if let Ok(items) = root.query_selector_all(FOCUSABLE_SELECTOR)
        && focus_next(&items, &tab_stops(root, &items), None, false)
    {
        return;
    }
    // Nothing to focus: the modal dialog itself (APG), not the page behind.
    if let Ok(target) = root.query_selector("[aria-modal=\"true\"]") {
        let _ = target.focus();
    }
}

/// `false` when nothing took focus, so the caller leaves Tab to the browser.
fn cycle_focus(root: &ElementHandle, backwards: bool) -> bool {
    let Ok(items) = root.query_selector_all(FOCUSABLE_SELECTOR) else {
        return false;
    };
    let index = items.iter().position(|item| item.is_focused());
    focus_next(&items, &tab_stops(root, &items), index, backwards)
}

/// Which of `items` are Tab stops: a native radio group is one, its checked
/// radio (matched by `:checked` and `value`) or else its first.
fn tab_stops(root: &ElementHandle, items: &[Box<dyn ElementApi>]) -> Vec<bool> {
    let attr = |item: &dyn ElementApi, name| item.attribute(name).ok().flatten();
    let group = |item: &dyn ElementApi| {
        let radio = attr(item, "type").is_some_and(|ty| ty.eq_ignore_ascii_case("radio"));
        radio
            .then(|| attr(item, "name"))
            .flatten()
            .filter(|name| !name.is_empty())
    };
    let checked: Vec<(String, String)> = root
        .query_selector_all(&format!("{FOCUSABLE_SELECTOR}:checked"))
        .unwrap_or_default()
        .iter()
        .filter_map(|item| {
            let value = attr(item.as_ref(), "value").unwrap_or_else(|| "on".into());
            Some((group(item.as_ref())?, value))
        })
        .collect();
    let mut seen: Vec<String> = Vec::new();
    items
        .iter()
        .map(|item| {
            let Some(name) = group(item.as_ref()) else {
                return true;
            };
            if seen.contains(&name) {
                return false;
            }
            let value = attr(item.as_ref(), "value").unwrap_or_else(|| "on".into());
            let stop = match checked.iter().find(|(group, _)| *group == name) {
                Some((_, checked)) => *checked == value,
                None => true,
            };
            if stop {
                seen.push(name);
            }
            stop
        })
        .collect()
}

/// Focuses the stop after `index` (from `None`: the first, or last backwards).
/// Skips non-stops and a `display: none` match, which ignores `focus()`.
fn focus_next(
    items: &[Box<dyn ElementApi>],
    stops: &[bool],
    index: Option<usize>,
    backwards: bool,
) -> bool {
    let len = items.len();
    let mut first_ok = None;
    for step in 1..=len {
        let next = match (index, backwards) {
            (None, false) => step - 1,
            (None, true) => len - step,
            (Some(i), false) => (i + step) % len,
            (Some(i), true) => (i + len - step) % len,
        };
        if !stops.get(next).copied().unwrap_or(true) {
            continue;
        }
        if items[next].focus().is_err() {
            continue;
        }
        if items[next].is_focused() {
            return true;
        }
        first_ok.get_or_insert(next);
    }
    // A renderer that applies focus later cannot confirm it: the plain next stop.
    first_ok.is_some_and(|next| items[next].focus().is_ok())
}

base_props! {
    pub struct FocusTrapProps {
        children: Element,
    }
}

/// Keeps Tab and Shift+Tab cycling inside its children, and focuses the first one on mount.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::FocusTrap;
/// # fn app() -> Element {
/// rsx! {
///     FocusTrap {
///         input { placeholder: "Name" }
///         button { "Save" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/accessibility/focus-trap>
#[component]
pub fn FocusTrap(props: FocusTrapProps) -> Element {
    let root = use_element();

    use_box()
        .framework_sx(&FOCUS_TRAP_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        // Not `.element(&root)`: mount and first focus share the one `onmounted`.
        .event("onmounted", move |event: Event<MountedData>| {
            (root.mount())(event);
            focus_first(&root);
        })
        .event("onkeydown", move |event: Event<KeyboardData>| {
            // A nested trap below already moved focus for this press.
            if event.key() == Key::Tab
                && !key_taken(&event)
                && cycle_focus(&root, event.modifiers().shift())
            {
                event.prevent_default();
            }
        })
        .render(HtmlTag::Div, props.attributes, props.children)
}

/// A hidden placeholder inside a [`FocusTrap`] that takes initial focus, then leaves the tab order.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{FocusTrap, FocusTrapInitialFocus};
/// # fn app() -> Element {
/// rsx! {
///     FocusTrap {
///         FocusTrapInitialFocus {}
///         button { "Close" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/accessibility/focus-trap>
#[component]
pub fn FocusTrapInitialFocus() -> Element {
    let used = use_local_state(|| false);
    let mark_used = used.clone();

    use_box()
        .framework_sx(&VISUALLY_HIDDEN_FIXED_SX)
        .prepare()
        .attr("tabindex", if used.get() { "-1" } else { "0" })
        .attr("data-autofocus", true)
        .event("onblur", move |_: Event<FocusData>| mark_used.set(true))
        .render(HtmlTag::Span, Vec::new(), ())
}
