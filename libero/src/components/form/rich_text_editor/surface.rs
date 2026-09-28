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
const root = () => document.querySelector(`[data-lsx-rich-text="${token}"]`);
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
const report = () => {
    const r = root();
    if (!r) { document.removeEventListener('selectionchange', report); return; }
    const s = document.getSelection();
    if (!s || !s.anchorNode) return;
    const a = leafOf(s.anchorNode), f = leafOf(s.focusNode);
    if (!a || !f) return;
    dioxus.send({ selection: [+a.dataset.key, units(a, s.anchorNode, s.anchorOffset),
        +f.dataset.key, units(f, s.focusNode, s.focusOffset)], caret: caretAt(s, f) });
};
// The head's line box, the overlay's box and the viewport, all in viewport px.
const caretAt = (s, leaf) => {
    const r = root(), box = r.closest('[data-lsx-rich-text-box]');
    if (!box) return null;
    const range = document.createRange();
    range.setStart(s.focusNode, s.focusOffset);
    let rect = range.getClientRects()[0];
    // An empty line has no rect: its line-box filler or the leaf stands in.
    if (!rect) rect = (leaf.querySelector('[data-skip]') || leaf).getBoundingClientRect();
    const b = box.getBoundingClientRect();
    return { x: rect.left, y: rect.top, height: rect.height, box_x: b.left, box_y: b.top,
        width: innerWidth, viewport_height: innerHeight,
        rtl: getComputedStyle(r).direction === 'rtl' };
};
document.addEventListener('selectionchange', report);
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
while (true) {
    const m = await dioxus.recv();
    const r = root();
    if (!r) continue;
    if (m.read !== undefined) {
        const l = leaf(m.read);
        if (l) dioxus.send({ text: [m.read, text(l)] });
    }
    if (m.select) {
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
    /// With a selection: where its head is, for the overlay.
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
}

/// The caret's line box and the editor's box (`data-lsx-rich-text-box`), in viewport px.
#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
pub(crate) struct Caret {
    pub x: f64,
    pub y: f64,
    pub height: f64,
    pub box_x: f64,
    pub box_y: f64,
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
    /// Starts the script for the root marked `token`; each report goes to `on_report`.
    pub fn start(token: String, mut on_report: impl FnMut(Report) + 'static) -> Self {
        let mut eval = Signal::new(None);
        if !crate::platform::edits_rich_text() {
            return Self { eval };
        }
        spawn(async move {
            let mut script = document::eval(SCRIPT);
            if script.send(token).is_err() {
                return;
            }
            eval.set(Some(script));
            while let Ok(report) = script.recv::<Report>().await {
                on_report(report);
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
}
