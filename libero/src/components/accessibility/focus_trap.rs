use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use dioxus::core::AttributeValue;
use dioxus::prelude::*;

use super::{
    tab_stops::{focus_first_stop, tab_order},
    visually_hidden::VISUALLY_HIDDEN_FIXED_SX,
};
use crate::{
    components::{
        common::{FOCUSABLE_SELECTOR, HtmlTag, Input, base_props},
        layout::use_box,
    },
    hooks::{ElementHandle, use_element, use_focus_return, use_local_state},
    platform::{
        ElementApi, KeyChord, OBSERVE_ATTR, document, focus_first_of, is_tab, key_taken, keyboard,
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
        let mut order = tab_order(&items);
        // An autofocus target leads; one that takes no focus falls through to the first stop.
        if let Ok(target) = root.query_selector("[data-autofocus]") {
            items.insert(0, target);
            order = std::iter::once(0)
                .chain(order.into_iter().map(|at| at + 1))
                .collect();
        }
        if focus_next(root, &items, &order, None, false) {
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
    // No stop to move to: focus on the trap or its modal box stays there (APG).
    focus_next(root, &items, &order, index, backwards) || holds_focus(root)
}

/// Controls a script may focus out of Tab order (a roving toolbar's `tabindex="-1"` button).
const CONTROLS: &str = "a[href], button, input, select, textarea, summary, iframe, audio, video";

fn holds_focus(root: &ElementHandle) -> bool {
    root.is_focused() || root.query_selector(":focus").is_ok()
}

/// Focus fell out of the trap: to the document (`<body>`, `<html>` on Blitz, a removed element) or
/// onto a container that is no Tab stop and no widget, as a clicked `<main tabindex="-1">` (4662).
/// A platform that cannot say, or a window blur, which keeps the element, is not a fall.
fn focus_fell(root: &ElementHandle) -> bool {
    !holds_focus(root)
        && document()
            .and_then(|document| document.active_element())
            .is_some_and(|active| {
                !active.is_connected()
                    || active.query_selector("body").is_ok()
                    || active.query_selector(":scope:is(body) > *").is_ok()
                    || active
                        .matches(&format!(
                            ":not({FOCUSABLE_SELECTOR}, {CONTROLS}, [role], [contenteditable])"
                        ))
                        .unwrap_or(false)
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

/// Focuses the stop after item `index` in `order` (from `None`: the first, or last backwards),
/// wrapping round; the walk is [`focus_first_stop`]'s.
fn focus_next(
    root: &dyn ElementApi,
    items: &[Box<dyn ElementApi>],
    order: &[usize],
    index: Option<usize>,
    backwards: bool,
) -> bool {
    let index = index.and_then(|index| order.iter().position(|at| *at == index));
    let split = match (index, backwards) {
        (Some(at), false) => at + 1,
        (Some(at), true) => at,
        (None, _) => 0,
    };
    let (head, tail) = order.split_at(split);
    let walk: Vec<usize> = match backwards {
        true => head
            .iter()
            .rev()
            .chain(tail.iter().rev())
            .copied()
            .collect(),
        false => tail.iter().chain(head).copied().collect(),
    };
    focus_first_stop(root, items, walk.into_iter(), backwards)
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
/// itself when given a `tabindex` and a box (not its default `display: contents`), and Tab
/// keeps focus there.
///
/// A native radio group, the radios sharing a `name` and a form, is one stop: its checked
/// radio, or else its first.
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
                chord.key == Key::Named(NamedKey::Tab)
                    && holds_last(id)
                    && focus_fell(&root)
                    && root
                        .query_selector_all(FOCUSABLE_SELECTOR)
                        .is_ok_and(|items| {
                            let order = tab_order(&items);
                            focus_next(&root, &items, &order, None, chord.modifiers.shift())
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
                if !focus_fell(&root) {
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
            if is_tab(&event) && !key_taken(&event) && cycle_focus(&root, event.modifiers().shift())
            {
                event.prevent_default();
            }
        })
        .event("onfocusin", onfocusin)
        .event("onfocusout", onfocusout)
        .render(HtmlTag::Div, attributes, props.children)
}

/// A hidden placeholder inside a [`FocusTrap`] that takes initial focus. Never a Tab stop.
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
        // Script-focusable, never a Tab stop: an unnamed stop would be announced as nothing.
        .attr("tabindex", "-1")
        .attr("data-autofocus", !used.get())
        .event("onblur", move |_: Event<FocusData>| mark_used.set(true))
        .render(HtmlTag::Span, Vec::new(), ())
}
