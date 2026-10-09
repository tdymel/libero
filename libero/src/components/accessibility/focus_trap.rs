use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use dioxus::core::AttributeValue;
use dioxus::prelude::*;

use super::visually_hidden::VISUALLY_HIDDEN_FIXED_SX;
use crate::{
    components::{
        common::{FOCUSABLE_SELECTOR, HtmlTag, Input, base_props},
        layout::use_box,
    },
    hooks::{ElementHandle, use_element, use_focus_return, use_local_state},
    platform::{
        ElementApi, KeyChord, OBSERVE_ATTR, document, focus_first_of, key_taken, keyboard,
        next_task, when_free, when_laid_out,
    },
    sx::{StaticSx, sx},
};

static FOCUS_TRAP_SX: StaticSx = StaticSx::new(|| sx().display("contents"));

fn focus_first(root: &ElementHandle, tag: Option<&str>) {
    // A WebView queries nothing: the page focuses inside the root's tag, in the same order.
    if root.query_selector_all(FOCUSABLE_SELECTOR).is_err()
        && let Some(tag) = tag
    {
        let within = format!("[{OBSERVE_ATTR}='{tag}']");
        let _ = focus_first_of(&[
            format!("{within} [data-autofocus]"),
            format!("{within} :is({FOCUSABLE_SELECTOR})"),
            format!("{within} [aria-modal=\"true\"]"),
            format!("{within}[tabindex]"),
        ]);
        return;
    }
    if let Ok(mut items) = root.query_selector_all(FOCUSABLE_SELECTOR) {
        let mut stops = tab_stops(root, &items);
        let mut order = tab_order(&items);
        // An autofocus target leads; one that takes no focus falls through to the first stop.
        if let Ok(target) = root.query_selector("[data-autofocus]") {
            items.insert(0, target);
            stops.insert(0, true);
            order = std::iter::once(0)
                .chain(order.into_iter().map(|at| at + 1))
                .collect();
        }
        if focus_next(&items, &stops, &order, None, false) {
            return;
        }
    }
    // Nothing to focus: the modal dialog itself (APG), not the page behind; without one, a
    // trap given a `tabindex` takes focus itself (`Modal`'s content box, 2559).
    match root.query_selector("[aria-modal=\"true\"]") {
        Ok(target) => {
            let _ = target.focus();
        }
        Err(_) if root.attribute("tabindex").ok().flatten().is_some() => {
            let _ = root.focus();
        }
        Err(_) => {}
    }
}

/// `false` when nothing took focus, so the caller leaves Tab to the browser.
fn cycle_focus(root: &ElementHandle, backwards: bool) -> bool {
    let Ok(items) = root.query_selector_all(FOCUSABLE_SELECTOR) else {
        return false;
    };
    let index = items.iter().position(|item| item.is_focused());
    let order = tab_order(&items);
    focus_next(&items, &tab_stops(root, &items), &order, index, backwards)
}

/// Focus fell to the document: on `<body>` (`<html>` on Blitz) or on a removed element.
/// A platform that cannot say, or a window blur, which keeps the element, is not a fall.
fn focus_fell() -> bool {
    document()
        .and_then(|document| document.active_element())
        .is_some_and(|active| {
            !active.is_connected()
                || active.query_selector("body").is_ok()
                || active.query_selector(":scope:is(body) > *").is_ok()
        })
}

thread_local! {
    /// Mounted traps in mount order, each with whether focus was last inside it (2297).
    static TRAPS: RefCell<Vec<(u64, Rc<Cell<bool>>)>> = const { RefCell::new(Vec::new()) };
    static NEXT_TRAP: Cell<u64> = const { Cell::new(0) };
}

/// Whether `id` is the innermost trap focus was last inside: the one a fallen Tab returns to.
fn holds_last(id: u64) -> bool {
    TRAPS.with_borrow(|traps| {
        traps
            .iter()
            .rev()
            .find(|(_, held)| held.get())
            .is_some_and(|(top, _)| *top == id)
    })
}

/// A trap's place in [`TRAPS`], dropped on unmount.
struct Registered(u64);

impl Drop for Registered {
    fn drop(&mut self) {
        TRAPS.with_borrow_mut(|traps| traps.retain(|(id, _)| *id != self.0));
    }
}

/// Which of `items` are Tab stops: a native radio group is one, its checked
/// radio (matched by `:checked` and `value`) or else its first.
pub(crate) fn tab_stops(root: &ElementHandle, items: &[Box<dyn ElementApi>]) -> Vec<bool> {
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

/// Item indices in the browser's Tab order: positive `tabindex` ascending, then the rest in DOM order.
pub(crate) fn tab_order(items: &[Box<dyn ElementApi>]) -> Vec<usize> {
    let tabindex = |item: &dyn ElementApi| {
        let value = item.attribute("tabindex").ok().flatten();
        value.and_then(|text| text.trim().parse::<i32>().ok())
    };
    let indices: Vec<Option<i32>> = items.iter().map(|item| tabindex(item.as_ref())).collect();
    order_by_tabindex(&indices)
}

fn order_by_tabindex(tabindexes: &[Option<i32>]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..tabindexes.len()).collect();
    // Stable: equal `tabindex` and the unordered keep their DOM order.
    order.sort_by_key(|at| match tabindexes[*at] {
        Some(index) if index > 0 => (0, index),
        _ => (1, 0),
    });
    order
}

/// `root`'s first Tab stop in Tab order that takes focus (the last, `last`); whether one did.
pub(crate) fn focus_edge(root: &ElementHandle, last: bool) -> bool {
    let Ok(items) = root.query_selector_all(FOCUSABLE_SELECTOR) else {
        return false;
    };
    let stops = tab_stops(root, &items);
    let order = tab_order(&items);
    match last {
        true => focus_first_stop(&items, &stops, order.into_iter().rev()),
        false => focus_first_stop(&items, &stops, order.into_iter()),
    }
}

/// Focuses the first stop of `order` that takes it: a `display: none` one ignores `focus()`.
pub(crate) fn focus_first_stop(
    items: &[Box<dyn ElementApi>],
    stops: &[bool],
    order: impl Iterator<Item = usize>,
) -> bool {
    order
        .filter(|at| stops.get(*at).copied().unwrap_or(true))
        .any(|at| items[at].focus().is_ok() && items[at].is_focused())
}

/// Focuses the stop after item `index` in `order` (from `None`: the first, or last backwards).
/// Skips non-stops and a `display: none` match, which ignores `focus()`.
fn focus_next(
    items: &[Box<dyn ElementApi>],
    stops: &[bool],
    order: &[usize],
    index: Option<usize>,
    backwards: bool,
) -> bool {
    let len = order.len();
    let index = index.and_then(|index| order.iter().position(|at| *at == index));
    let mut first_ok = None;
    for step in 1..=len {
        let next = order[match (index, backwards) {
            (None, false) => step - 1,
            (None, true) => len - step,
            (Some(i), false) => (i + step) % len,
            (Some(i), true) => (i + len - step) % len,
        }];
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
        /// On unmount, hands focus back to whatever held it when the trap mounted.
        #[props(default)]
        restore_focus: bool,
    }
}

/// Keeps Tab and Shift+Tab cycling inside its children, and focuses the first one on mount,
/// both in the browser's Tab order: a positive `tabindex` first.
/// It has no Escape, and restores focus on unmount only with `restore_focus`.
///
/// With nothing focusable inside it focuses an `aria-modal="true"` child, or else
/// itself when given a `tabindex` and a box (not its default `display: contents`).
///
/// A native radio group is one stop, grouped by `name` only and its checked radio found by
/// `value`: two forms sharing a radio name, or radios sharing or lacking a `value`, mislead it.
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
    // A tag the caller already spread wins: one element carries one (1255).
    let theirs = props
        .attributes
        .iter()
        .find_map(|attribute| match &attribute.value {
            AttributeValue::Text(tag) if attribute.name == OBSERVE_ATTR => Some(tag.clone()),
            _ => None,
        });
    let tag = theirs
        .clone()
        .or_else(|| root.tag().map(|tag| tag.to_string()));
    let mut attributes = props.attributes;
    if theirs.is_none() {
        attributes.extend(root.attributes());
    }

    let back = use_focus_return();
    let mut restore = use_hook(|| CopyValue::new(false));
    restore.set(props.restore_focus);
    use_drop(move || {
        if restore.try_peek().is_ok_and(|restore| *restore) {
            back.restore_detached();
        }
    });

    // Focus gone without a Tab (a removed control, a click outside): refocused when it
    // fell to the document, and a Tab from there comes back in (2297).
    let held = use_hook(|| Rc::new(Cell::new(false)));
    let last = use_hook(|| Rc::new(RefCell::new(None::<Box<dyn ElementApi>>)));
    let id = use_hook(|| {
        let id = NEXT_TRAP.replace(NEXT_TRAP.get() + 1);
        TRAPS.with_borrow_mut(|traps| traps.push((id, held.clone())));
        Rc::new(Registered(id))
    })
    .0;
    use_hook(move || {
        Rc::new(keyboard().map(|api| {
            api.on_key_unfiltered(Box::new(move |chord: KeyChord| {
                chord.key == Key::Tab
                    && holds_last(id)
                    && focus_fell()
                    && root
                        .query_selector_all(FOCUSABLE_SELECTOR)
                        .is_ok_and(|items| {
                            let (stops, order) = (tab_stops(&root, &items), tab_order(&items));
                            focus_next(&items, &stops, &order, None, chord.modifiers.shift())
                        })
            }))
        }))
    });
    let onfocusin = {
        let (held, last) = (held.clone(), last.clone());
        move |_: Event<FocusData>| {
            held.set(true);
            if let Ok(focused) = root.query_selector(":focus") {
                *last.borrow_mut() = Some(focused);
            }
        }
    };
    let onfocusout = move |_: Event<FocusData>| {
        let (held, last) = (held.clone(), last.clone());
        // The web lands focus after `focusout`: asked a task later, once the document is free.
        spawn(async move {
            next_task().await;
            when_free(move || {
                if root.query_selector(":focus").is_ok() {
                    return;
                }
                // Moved on to an element outside, or the window blurred: not ours to take back.
                if !focus_fell() {
                    held.set(false);
                    return;
                }
                // Taken out: its `focus()` runs `onfocusin`, which stores it again.
                let previous = last.borrow_mut().take();
                let back = previous.is_some_and(|last| {
                    last.is_connected() && last.focus().is_ok() && last.is_focused()
                });
                if !back {
                    focus_first(&root, None);
                }
            });
        });
    };

    use_box()
        .framework_sx(&FOCUS_TRAP_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        // Not `.element(&root)`: mount and first focus share the one `onmounted`.
        .event("onmounted", move |event: Event<MountedData>| {
            (root.mount())(event);
            if *restore.peek() {
                back.remember_focused();
            }
            // Blitz has no styles before layout, so a `display: none` target read as focusable (2370).
            let tag = tag.clone();
            when_laid_out(move || focus_first(&root, tag.as_deref()));
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
        .event("onfocusin", onfocusin)
        .event("onfocusout", onfocusout)
        .render(HtmlTag::Div, attributes, props.children)
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
        .attr("data-autofocus", !used.get())
        .event("onblur", move |_: Event<FocusData>| mark_used.set(true))
        .render(HtmlTag::Span, Vec::new(), ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_tabindex_comes_first_in_ascending_order() {
        let tabindexes = [None, Some(0), Some(3), Some(1), Some(3), None];
        assert_eq!(order_by_tabindex(&tabindexes), [3, 2, 4, 0, 1, 5]);
    }

    #[test]
    fn without_a_positive_tabindex_the_dom_order_stays() {
        assert_eq!(order_by_tabindex(&[None, Some(0), None]), [0, 1, 2]);
    }
}
