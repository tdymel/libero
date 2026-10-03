//! The DOM side of the editor: one script per editor reports selection changes and
//! reads or places the caret. Offsets cross it in DOM units (see `offsets`).
//!
//! Inside a leaf (`data-key`) a text node counts its UTF-16 length, a `br` or a
//! `data-atom` element counts 1, and a `data-skip` element counts nothing.

use dioxus::prelude::*;
use serde::Deserialize;

/// Marks an editor root; its value is the token the script finds it by.
pub(crate) const ROOT_ATTR: &str = "data-lsx-rich-text";

const SCRIPT: &str = r#"
const token = await dioxus.recv();
const clips = await dioxus.recv();
const root =() => document.querySelector(`[data-lsx-rich-text="${token}"]`);
const leafOf = (node) => {
    let el = node && (node.nodeType === 1 ? node : node.parentElement);
    el = el && el.closest('[data-key]');
    const r = root();
    return el && r && r.contains(el) ? el : null;
};
const units = (leaf, node, offset) => {
    let count = 0, done = false;
    const walk = (n) => {
        if (done) return;
        if (n === node && n.nodeType === 3) { count += offset; done = true; return; }
        if (n.nodeType === 3) { count += n.length; return; }
        if (n.nodeType !== 1 || n.hasAttribute('data-skip')) return;
        if (n === node) {
            for (let i = 0; i < offset && i < n.childNodes.length; i++) walk(n.childNodes[i]);
            done = true;
            return;
        }
        if (n.tagName === 'BR' || n.hasAttribute('data-atom')) { count += 1; return; }
        for (const c of n.childNodes) { walk(c); if (done) return; }
    };
    walk(leaf);
    return count;
};
const text = (leaf) => {
    let out = '';
    const walk = (n) => {
        if (n.nodeType === 3) { out += n.data; return; }
        if (n.nodeType !== 1 || n.hasAttribute('data-skip')) return;
        if (n.tagName === 'BR') { out += '\n'; return; }
        if (n.hasAttribute('data-atom')) { out += '￼'; return; }
        for (const c of n.childNodes) walk(c);
    };
    for (const c of leaf.childNodes) walk(c);
    return out;
};
const locate = (leaf, offset) => {
    let left = offset, found = null;
    const walk = (n) => {
        if (found) return;
        if (n.nodeType === 3) {
            if (left <= n.length) { found = [n, left]; return; }
            left -= n.length;
            return;
        }
        if (n.nodeType !== 1 || n.hasAttribute('data-skip')) return;
        if (n.tagName === 'BR' || n.hasAttribute('data-atom')) {
            const at = Array.prototype.indexOf.call(n.parentNode.childNodes, n);
            if (left === 0) { found = [n.parentNode, at]; return; }
            left -= 1;
            if (left === 0 && n.hasAttribute('data-atom')) found = [n.parentNode, at + 1];
            return;
        }
        for (const c of n.childNodes) { walk(c); if (found) return; }
    };
    for (const c of leaf.childNodes) walk(c);
    return found || [leaf, leaf.childNodes.length];
};
const leaf = (key) => { const r = root(); return r && r.querySelector(`[data-key="${key}"]`); };
// The DOM selection as a report, or null outside the leaves.
const current = () => {
    const s = document.getSelection();
    if (!s || !s.anchorNode) return null;
    const a = leafOf(s.anchorNode), f = leafOf(s.focusNode);
    if (!a || !f) return null;
    return { selection: [+a.dataset.key, units(a, s.anchorNode, s.anchorOffset),
        +f.dataset.key, units(f, s.focusNode, s.focusOffset)], caret: caretAt(s, f) };
};
const report = () => {
    if (!root()) { document.removeEventListener('selectionchange', report); return; }
    const now = current();
    if (now) dioxus.send({ ...now, press: fresh });
};
// The head's line box and the viewport, in viewport px.
const caretAt = (s, leaf) => {
    const range = document.createRange();
    range.setStart(s.focusNode, s.focusOffset);
    let rect = range.getClientRects()[0];
    // An empty line has no rect: its line-box filler or the leaf stands in.
    if (!rect) rect = (leaf.querySelector('[data-skip]') || leaf).getBoundingClientRect();
    return { x: rect.left, y: rect.top, height: rect.height, width: innerWidth,
        viewport_height: innerHeight, rtl: getComputedStyle(root()).direction === 'rtl' };
};
document.addEventListener('selectionchange', report);
// Chrome queues `selectionchange` behind input, and a report may reach the model after the
// next key: input right after a press goes with its selection, until the model acks it (todo 2062).
// `fresh`: the DOM selection is a press's until a key or a model caret (todo 2072).
let pressed = false, fresh = false, held = 0, lastKey = '';
document.addEventListener('mousedown', (e) => {
    const r = root();
    fresh = !!r && r.contains(e.target);
    if (fresh) pressed = true;
}, true);
// Typed text and clipboard keys go natively; another key waits for held input too (todo 2071).
const native = ['Control', 'Meta', 'Shift', 'Alt', 'AltGraph', 'CapsLock', 'Process', 'Unidentified', 'Dead'];
document.addEventListener('keydown', (e) => {
    lastKey = e.key;
    fresh = false;
    const mod = e.ctrlKey || e.metaKey;
    if (!held || e.target !== root() || e.isComposing || native.includes(e.key)
        || (e.key.length === 1 && !mod) || (mod && ['c', 'x', 'v'].includes(e.key.toLowerCase()))) return;
    e.preventDefault();
    e.stopImmediatePropagation();
    held += 1;
    dioxus.send({ key: [e.key, e.code, e.ctrlKey, e.metaKey, e.altKey, e.shiftKey] });
}, true);
document.addEventListener('beforeinput', (e) => {
    const r = root();
    if (!r || !r.contains(e.target) || !(pressed || held) || e.isComposing
        || e.inputType === 'insertCompositionText') return;
    const now = current();
    if (!now) return;
    e.preventDefault();
    e.stopImmediatePropagation();
    held += 1;
    dioxus.send({ ...now, press: pressed, input: [e.inputType, e.data, lastKey] });
    pressed = false;
}, true);
// A scroll or resize moves the caret in the viewport: re-place an open overlay. Both fire
// at most once a frame; no requestAnimationFrame, which a background tab never runs.
const onMove = () => {
    const r = root();
    if (!r) { removeEventListener('scroll', onMove, true); removeEventListener('resize', onMove); return; }
    const s = document.getSelection(), f = s && leafOf(s.focusNode);
    if (f && document.querySelector(`[data-overlay="${token}"]`)) dioxus.send({ caret: caretAt(s, f) });
};
addEventListener('scroll', onMove, { capture: true, passive: true });
addEventListener('resize', onMove);
// Chrome steps over a rendered code block (contenteditable=false): step into it instead.
const onArrow = (e) => {
    const up = e.key === 'ArrowUp';
    if (!up && e.key !== 'ArrowDown') return;
    if (e.shiftKey || e.altKey || e.ctrlKey || e.metaKey || e.isComposing) return;
    const r = root(), s = document.getSelection();
    if (!r || !r.contains(e.target) || !s || !s.isCollapsed || !s.rangeCount) return;
    const l = leafOf(s.focusNode);
    if (!l) return;
    const stops = [...r.querySelectorAll('[data-key], [data-code-key]')];
    const next = stops[stops.indexOf(l) + (up ? -1 : 1)];
    // Down from the last line of a trailing code block leaves it for a new paragraph.
    const exit = !next && !up && l.closest('[data-code="source"]');
    if (!exit && !(next && next.hasAttribute('data-code-key'))) return;
    const caret = s.getRangeAt(0).getBoundingClientRect(), box = l.getBoundingClientRect();
    const line = parseFloat(getComputedStyle(l).lineHeight) || caret.height || 16;
    const edge = caret.height === 0 || (up ? caret.top < box.top + line : caret.bottom > box.bottom - line);
    if (!edge) return;
    e.preventDefault();
    dioxus.send(exit ? { exit: true } : { code: [+next.dataset.codeKey, up] });
};
document.addEventListener('keydown', onArrow);
// A press in the empty space under the last block puts the caret on a line there.
const onPress = (e) => {
    const r = root();
    if (!r || e.target !== r || e.button !== 0 || r.contentEditable !== 'true') return;
    const parts = r.querySelectorAll('[data-key], [data-code], [data-node]');
    const bottom = Math.max(...[...parts].map((p) => p.getBoundingClientRect().bottom));
    if (!parts.length || e.clientY <= bottom) return;
    e.preventDefault();
    dioxus.send({ end: true });
};
document.addEventListener('mousedown', onPress);
// A WebView hands Rust a copy of the clipboard event: the model's Markdown goes in here,
// or the DOM's text if the selection moved since (todo 2106).
let clip = null;
const onClip = (e) => {
    const r = root();
    if (!r) { removeEventListener('copy', onClip, true); removeEventListener('cut', onClip, true); return; }
    const now = current(), text = String(document.getSelection());
    if (!r.contains(e.target) || !e.clipboardData || !now || !text) return;
    const same = clip && clip.at.every((v, i) => v === now.selection[i]);
    e.clipboardData.setData('text/plain', same ? clip.markdown : text);
    if (same) e.clipboardData.setData('text/markdown', clip.markdown);
    e.preventDefault();
};
if (clips) { addEventListener('copy', onClip, true); addEventListener('cut', onClip, true); }
while (true) {
    const m = await dioxus.recv();
    if (m.ack) held -= 1;
    if (m.clip) clip = { at: m.clip, markdown: m.markdown };
    const r = root();
    if (!r) continue;
    if (m.read !== undefined) {
        const l = leaf(m.read);
        if (l) dioxus.send({ text: [m.read, text(l)] });
    }
    if (m.select) {
        fresh = false;
        const [ak, ao, hk, ho] = m.select;
        const a = leaf(ak), h = leaf(hk);
        if (a && h && (m.focus || r.contains(document.activeElement))) {
            if (m.focus) r.focus({ preventScroll: true });
            const [an, aoff] = locate(a, ao), [hn, hoff] = locate(h, ho);
            document.getSelection().setBaseAndExtent(an, aoff, hn, hoff);
        }
        // After the selectionchange this move queues, so that one is not stale.
        setTimeout(() => dioxus.send({ synced: true }), 0);
    }
}
"#;

/// What the script reports.
#[derive(Deserialize, Debug)]
pub(crate) struct Report {
    /// Anchor key, anchor DOM offset, head key, head DOM offset.
    pub selection: Option<(u64, usize, u64, usize)>,
    /// With a selection, or alone after a scroll or resize: where its head is, for the overlay.
    #[serde(default)]
    pub caret: Option<Caret>,
    /// A leaf's key and its DOM text, asked for by [`Surface::read`].
    pub text: Option<(u64, String)>,
    /// An arrow key left a leaf toward this rendered code block; `true` enters at its end.
    pub code: Option<(u64, bool)>,
    /// ArrowDown on the last line of a trailing code block.
    #[serde(default)]
    pub exit: bool,
    /// A press below the last block.
    #[serde(default)]
    pub end: bool,
    /// The DOM selection caught up with the last [`Surface::select`].
    #[serde(default)]
    pub synced: bool,
    /// A held-back `beforeinput` to run after `selection`: its type, data and last key.
    pub input: Option<(String, Option<String>, String)>,
    /// A keydown held behind input: key, code, ctrl, meta, alt, shift.
    pub key: Option<(String, String, bool, bool, bool, bool)>,
    /// `selection` is a press's, newer than any caret the model is still placing.
    #[serde(default)]
    pub press: bool,
}

/// The caret's line box, in viewport px.
#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
pub(crate) struct Caret {
    pub x: f64,
    pub y: f64,
    pub height: f64,
    /// The viewport's width.
    pub width: f64,
    pub viewport_height: f64,
    pub rtl: bool,
}

/// One editor's script. Dropping the editor leaves the listener until the next event.
#[derive(Clone, Copy)]
pub(crate) struct Surface {
    eval: Signal<Option<document::Eval>>,
}

impl Surface {
    /// Starts the script for the root marked `token`; each report goes to `on_report`,
    /// which may answer a reported selection with its [`clip`](Self::clip) Markdown.
    pub fn start(
        token: String,
        mut on_report: impl FnMut(Report) -> Option<String> + 'static,
    ) -> Self {
        let mut eval = Signal::new(None);
        if !crate::platform::edits_rich_text() {
            return Self { eval };
        }
        spawn(async move {
            let mut script = document::eval(SCRIPT);
            if script.send(token).is_err() || script.send(!cfg!(target_arch = "wasm32")).is_err() {
                return;
            }
            eval.set(Some(script));
            while let Ok(report) = script.recv::<Report>().await {
                let held = report.input.is_some() || report.key.is_some();
                let selection = report.selection;
                let clip = on_report(report);
                if held {
                    let _ = script.send(serde_json::json!({ "ack": true }));
                }
                if let (Some((ak, ao, hk, ho)), Some(markdown)) = (selection, clip) {
                    let _ = script.send(clip_message((ak, ao), (hk, ho), markdown));
                }
            }
        });
        Self { eval }
    }

    fn send(&self, message: serde_json::Value) {
        if let Some(eval) = *self.eval.peek() {
            let _ = eval.send(message);
        }
    }

    /// Places the DOM selection; `focus` also focuses the editor first.
    pub fn select(&self, anchor: (u64, usize), head: (u64, usize), focus: bool) {
        self.send(serde_json::json!({
            "select": [anchor.0, anchor.1, head.0, head.1],
            "focus": focus,
        }));
    }

    /// Asks for a leaf's DOM text, answered as [`Report::text`].
    pub fn read(&self, key: u64) {
        self.send(serde_json::json!({ "read": key }));
    }

    /// What a copy or cut of the DOM selection `anchor`..`head` puts on the clipboard.
    pub fn clip(&self, anchor: (u64, usize), head: (u64, usize), markdown: String) {
        self.send(clip_message(anchor, head, markdown));
    }
}

fn clip_message(anchor: (u64, usize), head: (u64, usize), markdown: String) -> serde_json::Value {
    serde_json::json!({ "clip": [anchor.0, anchor.1, head.0, head.1], "markdown": markdown })
}
