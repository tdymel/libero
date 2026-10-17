use std::rc::Rc;
use std::sync::atomic::Ordering;

use dioxus::core::spawn_forever;
use serde_json::json;

use super::{Focused, NEXT_SLOT, PageState, eval, eval_with, page, runs_scripts};
use crate::platform::keyboard::{takes_arrows, takes_typing};
use crate::platform::{FocusStep, PlatformError, Read};

/// A focus move with no `relatedTarget` left for no element. A caret move resends
/// only when an input's edges changed; `selectionchange` targets the input or the document.
const ON_FOCUS: &str =
    "let last;
    const edges = (el) => {
        if (!(el instanceof HTMLInputElement) || el.selectionStart == null) return null;
        const { selectionStart: start, selectionEnd: end } = el;
        return [start === end && start === 0, start === end && end === el.value.length];
    };
    const send = (el) => {
        const state = el instanceof Element && el !== document.body
            ? [el.tagName, el.getAttribute('type'), el.isContentEditable === true,
               getComputedStyle(el).direction === 'rtl', edges(el)]
            : null;
        const key = JSON.stringify(state);
        if (key !== last) { last = key; dioxus.send(state); }
    };
    document.addEventListener('focusin', (event) => send(event.target));
    document.addEventListener('focusout', (event) => { if (!event.relatedTarget) send(null); });
    const caret = () => { if (document.activeElement instanceof HTMLInputElement) send(document.activeElement); };
    document.addEventListener('selectionchange', caret, { capture: true });
    document.addEventListener('keyup', caret, { capture: true });
    document.addEventListener('input', caret, { capture: true });
    send(document.activeElement);
    await new Promise(() => {});";

/// Starts mirroring the page's focus into Rust, once per dom. A key press goes
/// to the focused element, so its handler reads the mirror, as on Blitz.
pub(crate) fn watch_focus() {
    focus_page();
}

pub(super) fn focus_page() -> Option<Rc<PageState>> {
    if !runs_scripts() {
        return None;
    }
    let page = page()?;
    if !page.watching_focus.replace(true) {
        let script = eval(ON_FOCUS);
        let mirror = page.clone();
        spawn_forever(async move {
            let mut script = script;
            while let Ok(next) = script.recv::<Focused>().await {
                mirror.focused.replace(next);
            }
        });
    }
    Some(page)
}

fn focused() -> Focused {
    focus_page().and_then(|page| page.focused.borrow().clone())
}

pub(crate) fn typing_target() -> bool {
    focused()
        .is_some_and(|(tag, kind, editable, ..)| editable || takes_typing(&tag, kind.as_deref()))
}

pub(crate) fn focused_tag() -> Option<String> {
    focused().map(|(tag, ..)| tag)
}

pub(crate) fn arrow_target() -> bool {
    focused().is_some_and(|(tag, kind, ..)| takes_arrows(&tag, kind.as_deref()))
}

pub(crate) fn rtl_target() -> bool {
    focused().is_some_and(|(_, _, _, rtl, _)| rtl)
}

/// The mirror's caret, current once the page's `selectionchange` crossed the IPC.
pub(crate) fn caret_edges() -> Option<(bool, bool)> {
    focused().and_then(|(.., edges)| edges)
}

/// Queued, like [`ElementApi::focus`] here: no handle names the element.
pub(crate) fn focus_selector(selector: &str) -> Result<(), PlatformError> {
    if !runs_scripts() {
        return Err(PlatformError::Unsupported);
    }
    eval_with(json!(selector), "document.querySelector(data)?.focus();");
    Ok(())
}

/// Queued, like [`focus_selector`].
pub(crate) fn focus_closest_of_focused(focused: &str, ancestor: &str) -> Result<(), PlatformError> {
    if !runs_scripts() {
        return Err(PlatformError::Unsupported);
    }
    eval_with(
        json!([focused, ancestor]),
        "const active = document.activeElement;
        if (active?.matches(data[0])) active.closest(data[1])?.focus();",
    );
    Ok(())
}

/// Queued, as [`focus_selector`].
pub(crate) fn click_selector(selector: &str) -> Result<(), PlatformError> {
    if !runs_scripts() {
        return Err(PlatformError::Unsupported);
    }
    eval_with(json!(selector), "document.querySelector(data)?.click();");
    Ok(())
}

/// The tab stop next to `selector`'s match (the focused element for ""): positive `tabindex`
/// ascending, then 0 in DOM order (2720). With none that way, the match keeps focus.
const FOCUS_TAB_FROM: &str = "const [selector, backwards] = data;
    const from = selector ? document.querySelector(selector) : document.activeElement;
    if (!from) return;
    const candidates = document.querySelectorAll('a[href], area[href], button, input, select, \
        textarea, summary, iframe, [tabindex], [contenteditable]:not([contenteditable=\"false\"])');
    const stops = [...candidates].filter((el) => el === from || (el.tabIndex >= 0
        && !el.disabled && !el.closest('[inert]') && el.type !== 'hidden'
        && el.getClientRects().length > 0 && getComputedStyle(el).visibility !== 'hidden'));
    const rank = (el) => (el.tabIndex > 0 ? el.tabIndex : Infinity);
    const order = stops.map((el, i) => [el, i])
        .sort(([a, i], [b, j]) => rank(a) - rank(b) || i - j)
        .map(([el]) => el);
    (order[order.indexOf(from) + (backwards ? -1 : 1)] ?? from).focus();";

pub(crate) fn focus_tab_from(selector: &str, backwards: bool) -> Result<(), PlatformError> {
    if !runs_scripts() {
        return Err(PlatformError::Unsupported);
    }
    eval_with(json!([selector, backwards]), FOCUS_TAB_FROM);
    Ok(())
}

/// Queued, like [`focus_selector`]. A match that ignores `focus()` (hidden) is skipped.
pub(crate) fn focus_first_of(selectors: &[String]) -> Result<(), PlatformError> {
    if !runs_scripts() {
        return Err(PlatformError::Unsupported);
    }
    eval_with(
        json!(selectors),
        "for (const selector of data) {
            for (const el of document.querySelectorAll(selector)) {
                el.focus();
                if (document.activeElement === el) return;
            }
        }",
    );
    Ok(())
}

/// Queued, read where it runs: the page's focus, not a mirror that may lag.
pub(crate) fn focus_among(
    attr: &str,
    values: &[String],
    to: FocusStep,
) -> Result<(), PlatformError> {
    if !runs_scripts() {
        return Err(PlatformError::Unsupported);
    }
    let (kind, by, wrap) = match to {
        FocusStep::By { by, wrap } => ("by", by, wrap),
        FocusStep::First => ("first", 0, false),
        FocusStep::Last => ("last", 0, false),
    };
    eval_with(
        json!([attr, values, kind, by, wrap]),
        "const [attr, values, kind, by, wrap] = data;
        const items = [...document.querySelectorAll(`[${attr}]`)]
            .filter((el) => values.includes(el.getAttribute(attr)));
        const at = items.findIndex((el) => el.contains(document.activeElement));
        const last = items.length - 1;
        let next = kind === 'first' ? 0 : last;
        if (kind === 'by') {
            if (at < 0) return;
            next = wrap ? (at + by + items.length) % items.length
                : Math.min(Math.max(at + by, 0), last);
        }
        items[next]?.focus();",
    );
    Ok(())
}

/// The focused element's nearest `attr` value, read once the script answers.
pub(crate) fn focused_attribute(attr: &str) -> Read<Option<String>> {
    if !runs_scripts() {
        return Box::pin(std::future::ready(Err(PlatformError::Unsupported)));
    }
    let read = eval_with(
        json!(attr),
        "return document.activeElement?.closest(`[${data}]`)?.getAttribute(data) ?? null;",
    );
    Box::pin(async move {
        read.join::<Option<String>>()
            .await
            .map_err(|_| PlatformError::Unsupported)
    })
}

/// Whether focus is inside an element matching `boundary`, read once the script answers.
pub(crate) fn focus_lands_in(boundary: &str) -> Read<bool> {
    if !runs_scripts() {
        return Box::pin(std::future::ready(Err(PlatformError::Unsupported)));
    }
    let read = eval_with(
        json!(boundary),
        "return !!document.activeElement?.closest(data);",
    );
    Box::pin(async move {
        read.join::<bool>()
            .await
            .map_err(|_| PlatformError::Unsupported)
    })
}

/// Keeps the focused element in the page under the returned token, for [`focus_kept`].
pub(crate) fn keep_focused() -> Option<u64> {
    if !runs_scripts() {
        return None;
    }
    let token = NEXT_SLOT.fetch_add(1, Ordering::Relaxed);
    eval_with(
        json!(token),
        "(window.__lsxKept ??= new Map()).set(data, document.activeElement);",
    );
    Some(token)
}

/// Focuses the element [`keep_focused`] kept, once; nothing if it left the page.
pub(crate) fn focus_kept(token: u64) {
    eval_with(
        json!(token),
        "const el = window.__lsxKept?.get(data);
        window.__lsxKept?.delete(data);
        if (el?.isConnected && el !== document.body) el.focus();",
    );
}
