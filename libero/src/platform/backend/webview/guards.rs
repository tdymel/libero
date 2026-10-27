use dioxus::prelude::{Event, PointerData, spawn};
use serde_json::json;

use super::{Slot, eval, eval_with, page, runs_scripts};
use crate::platform::PlatformError;

/// Wraps the interpreter's attribute write. Each `input` records the text it sent;
/// a `value` write matching an older record is the render of a stale keystroke and
/// is dropped. Any other is the app's own rewrite: it lands once every input sent
/// is answered, or after 200 ms without input (the newest rewrite wins), so a
/// rewriting app sees every keystroke. Records expire after 2 s.
const GUARD_TYPED: &str = "const interpreter = window.interpreter;
    if (!interpreter || interpreter.lsxTyped) return;
    const typed = interpreter.lsxTyped = new WeakMap();
    const write = interpreter.setAttributeInner.bind(interpreter);
    const live = (state) => {
        const now = performance.now();
        return state.records.filter(([, at]) => now - at < 2000);
    };
    const settle = (node, state) => {
        clearTimeout(state.timer);
        state.timer = setTimeout(() => {
            const held = state.stash;
            state.stash = null;
            state.inflight = 0;
            if (held && node.isConnected) write(node, ...held);
        }, 200);
    };
    window.addEventListener('input', (event) => {
        const node = event.target;
        if (!(node instanceof HTMLInputElement || node instanceof HTMLTextAreaElement)) return;
        const state = typed.get(node) ?? { records: [], inflight: 0, stash: null, timer: 0 };
        state.records = [...live(state), [node.value, performance.now()]];
        state.inflight += 1;
        typed.set(node, state);
        settle(node, state);
    }, { capture: true });
    interpreter.setAttributeInner = (node, field, value, ns) => {
        const state = field === 'value' && !ns ? typed.get(node) : undefined;
        if (state) {
            const records = live(state);
            const index = records.findIndex(([text]) => text === value);
            if (index !== -1) {
                state.records = records.slice(index + 1);
                state.inflight = state.records.length;
            } else {
                state.inflight = Math.max(0, state.inflight - 1);
            }
            if (state.inflight > 0) {
                if (index === -1) state.stash = [field, value, ns];
                return;
            }
            clearTimeout(state.timer);
            typed.delete(node);
        }
        write(node, field, value, ns);
    };";

/// Keeps a controlled field's older renders from overwriting faster typing: an
/// `input` crosses the IPC and its render comes back after the next keystroke (1026).
pub(crate) fn guard_typed_values() {
    if !runs_scripts() {
        return;
    }
    if let Some(page) = page()
        && !page.guarding_values.replace(true)
    {
        eval(GUARD_TYPED);
    }
}

/// Queued like [`WebViewDocument::set_root_attribute`](super::WebViewDocument), so an edit dioxus sends
/// later still lands over it.
pub(crate) fn set_value_by_id(id: &str, value: &str) -> Result<(), PlatformError> {
    if !runs_scripts() {
        return Err(PlatformError::Unsupported);
    }
    eval_with(
        json!([id, value]),
        "const [id, value] = data;
        const el = document.getElementById(id);
        if (el && el.value !== value) el.value = value;",
    );
    Ok(())
}

/// The page's first `<form>` matching `data` resets, then each `data-controlled` control gets
/// its value back, as the web's `ElementApi::reset`. Its own `reset` event stays in the page.
const RESET_FORM: &str = "const form = document.querySelector(data);
    if (!(form instanceof HTMLFormElement)) return;
    const saved = [...form.querySelectorAll('[data-controlled]')]
        .map((el) => [el, el.value, el.checked, el.selectedIndex]);
    const own = (event) => { if (event.target === form) event.stopImmediatePropagation(); };
    window.addEventListener('reset', own, { capture: true });
    try { form.reset(); } finally { window.removeEventListener('reset', own, { capture: true }); }
    for (const [el, value, checked, index] of saved) {
        if (el instanceof HTMLSelectElement) el.selectedIndex = index;
        else if (el instanceof HTMLInputElement) { el.value = value; el.checked = checked; }
        else if (el instanceof HTMLTextAreaElement) el.value = value;
    }";

/// Queued, so it runs past the reset event that asked for it, where `form.reset()` would be
/// ignored, and before the edits of the render it causes.
pub(crate) fn reset_selector(selector: &str) -> Result<(), PlatformError> {
    if !runs_scripts() {
        return Err(PlatformError::Unsupported);
    }
    eval_with(json!(selector), RESET_FORM);
    Ok(())
}

/// Records each pointer's pressed element for [`follow_pointer`]. A followed release
/// 4px or more from its press swallows its click: the web's capture on the drag root
/// sends that click to the root, never to the pressed handle.
const RECORD_PRESSES: &str = "const pressed = window.__lsxPressed = new Map();
    window.addEventListener('pointerdown', (event) => {
        pressed.set(event.pointerId, [event.target, event.clientX, event.clientY]);
    }, { capture: true });
    const release = (event) => {
        const [, x, y, followed] = pressed.get(event.pointerId) ?? [];
        pressed.delete(event.pointerId);
        if (event.type !== 'pointerup' || !followed
            || Math.hypot(event.clientX - x, event.clientY - y) < 4) return;
        const swallow = (click) => { click.stopPropagation(); click.preventDefault(); };
        window.addEventListener('click', swallow, { capture: true, once: true });
        setTimeout(() => window.removeEventListener('click', swallow, { capture: true }));
    };
    window.addEventListener('pointerup', release);
    window.addEventListener('pointercancel', release);";

/// WebKitGTK scrolls the nearest scroller on an arrow the caret cannot follow past a
/// field's edge; browsers do not (1178). Bubble phase, after every handler's say. `data`:
/// whether Up/Down are guarded too, see [`guard_defaults`].
const ARROW_EDGES: &str = "const vertical = data;
    window.addEventListener('keydown', (event) => {
        if (event.defaultPrevented || event.isComposing || event.altKey || event.ctrlKey
            || event.metaKey || event.shiftKey) return;
        const el = event.target;
        const area = el instanceof HTMLTextAreaElement;
        if (!area && !(el instanceof HTMLInputElement)) return;
        const { selectionStart: start, selectionEnd: end } = el;
        if (start == null || start !== end) return;
        const rtl = getComputedStyle(el).direction === 'rtl';
        const [first, last] = [start === 0, end === el.value.length];
        const edge = {
            ArrowLeft: rtl ? last : first,
            ArrowRight: rtl ? first : last,
            ArrowUp: vertical && !area && first,
            ArrowDown: vertical && !area && last,
        }[event.key];
        if (edge) event.preventDefault();
    });";

/// Fills in WebView defaults that differ from a browser's, once a page.
pub(crate) fn guard_defaults() {
    if !runs_scripts() {
        return;
    }
    if let Some(page) = page()
        && !page.guarding_defaults.replace(true)
    {
        eval(RECORD_PRESSES);
        // Android's spatial navigation (on after a D-pad key) scrolls on a sideways arrow at a
        // field's edge too (3137); Up/Down stay its way out of a field on a D-pad-only device.
        eval_with(json!(!cfg!(target_os = "android")), ARROW_EDGES);
    }
}

/// Captures the pointer on its pressed element, inside the drag's own, so moves and
/// the release still bubble there (1179). A release before the eval lands is ignored.
pub(crate) fn follow_pointer(event: &Event<PointerData>) {
    if !runs_scripts() {
        return;
    }
    eval_with(
        json!(event.pointer_id()),
        "const press = window.__lsxPressed?.get(data);
        const el = press?.[0];
        if (el?.isConnected && !el.hasPointerCapture(data)) {
            try { el.setPointerCapture(data); press[3] = true; } catch {}
        }",
    );
}

/// Calls `onrelease` once `pointer_id` lifts or cancels anywhere on the page, at once if it
/// already did: iOS WebKit at times sends a touch's release past its capture (3603).
pub(crate) fn on_pointer_release(pointer_id: i32, onrelease: dioxus::prelude::Callback<()>) {
    if !runs_scripts() {
        return;
    }
    // Parked, not `join`ed: a held drag can outlast `CHANNEL_HOLD_MS`.
    let slot = Slot::new();
    let script = eval_with(
        json!(pointer_id),
        &format!(
            "await new Promise((done) => {{
                if (window.__lsxPressed && !window.__lsxPressed.has(data)) return done();
                const up = (event) => {{
                    if (event.pointerId !== data) return;
                    window.removeEventListener('pointerup', up, true);
                    window.removeEventListener('pointercancel', up, true);
                    done();
                }};
                window.addEventListener('pointerup', up, true);
                window.addEventListener('pointercancel', up, true);
            }});
            dioxus.send(true);
            {}",
            slot.park("")
        ),
    );
    spawn(async move {
        let mut script = script;
        let released = script.recv::<bool>().await.is_ok();
        drop(slot);
        if released {
            onrelease.call(());
        }
    });
}
