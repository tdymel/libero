use dioxus::prelude::*;

use super::visually_hidden::VISUALLY_HIDDEN_FIXED_SX;
use crate::{
    components::{HtmlTag, Input, common::base_props, layout::use_box},
    hooks::{ElementHandle, use_element, use_local_state},
    platform::{ElementApi, key_taken},
    sx::{StaticSx, sx},
};

/// `FocusTrap` wraps arbitrary consumer children, so the tab order has to
/// exclude what a browser would never focus. The `:not(..)` tail drops
/// `hidden`/`inert`/`aria-hidden` elements *and their descendants* - without
/// it, Tab strands focus on something invisible - and anything with
/// `tabindex="-1"`, which a browser's Tab skips even on a button. Without that,
/// Tab walked every roving item: all six of a lightbox's thumbnails, where the
/// strip has one tab stop. `:disabled` too: a disabled button with
/// `tabindex="0"` (a calendar's Nav at `min`) matched the `[tabindex]` arm.
/// Deliberately still matched: a visually-hidden-but-focusable element, which
/// is what [`FocusTrapInitialFocus`] is.
pub(crate) const FOCUSABLE_SELECTOR: &str = concat!(
    ":is(a[href], button:not([disabled]), textarea:not([disabled]), ",
    "input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex=\"-1\"]), ",
    "summary, iframe, audio[controls], video[controls], ",
    "[contenteditable]:not([contenteditable=\"false\"]))",
    ":not([hidden], [inert], [aria-hidden=\"true\"], [tabindex=\"-1\"], :disabled, ",
    "[hidden] *, [inert] *, [aria-hidden=\"true\"] *)"
);

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

/// `false` when there was nothing to focus, so the caller can leave Tab to
/// the browser rather than swallowing it and stranding focus.
fn cycle_focus(root: &ElementHandle, backwards: bool) -> bool {
    let Ok(items) = root.query_selector_all(FOCUSABLE_SELECTOR) else {
        return false;
    };
    let index = items.iter().position(|item| item.is_focused());
    focus_next(&items, &tab_stops(root, &items), index, backwards)
}

/// Which of `items` are Tab stops: a native radio group is one, its checked
/// radio or, with none checked, its first. The live checked state is a
/// selector's (`:checked`), matched back to an item by the radio's `value`.
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

/// The stop after `index` that takes focus; from outside (`None`), the first
/// or, backwards, the last. A `display: none` match ignores `focus()`, so it
/// is passed over rather than stalling Tab on it. Items `stops` marks `false`
/// are passed over too.
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

/// A hidden placeholder that soaks up initial focus, then leaves the tab
/// order once blurred.
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

#[cfg(test)]
mod tests {
    use super::FOCUSABLE_SELECTOR;

    /// There is no DOM in a unit test, so this pins the one clause the
    /// browser pass measured: the exclusion list, not the `:is(..)` list, has
    /// to carry `tabindex="-1"`, or a button with it stays a Tab stop.
    #[test]
    fn a_tabindex_minus_one_element_is_never_a_tab_stop() {
        let (_, excluded) = FOCUSABLE_SELECTOR
            .split_once("):not(")
            .expect("an exclusion list after the :is(..)");

        assert!(
            excluded.contains(r#"[tabindex="-1"]"#),
            "{FOCUSABLE_SELECTOR}"
        );
    }

    /// A disabled button keeps no tab stop, whatever its `tabindex`.
    #[test]
    fn a_disabled_element_is_never_a_tab_stop() {
        let (_, excluded) = FOCUSABLE_SELECTOR
            .split_once("):not(")
            .expect("an exclusion list after the :is(..)");

        assert!(excluded.contains(":disabled"), "{FOCUSABLE_SELECTOR}");
    }
}
