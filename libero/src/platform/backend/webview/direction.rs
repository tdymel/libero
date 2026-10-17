use std::collections::HashMap;

use dioxus::core::spawn_forever;
use serde_json::json;

use super::{eval, eval_with, page, runs_scripts};

/// Sends `{tag: rtl}` for each `data-lsx-observe` element whose computed `direction`
/// changed, at start and on every `dir` change; `window.__lsxDirection(tag)` sends one tag's.
const ON_DIRECTION: &str = "const sent = new Map();
    const direction = (el) => getComputedStyle(el).direction === 'rtl';
    const scan = () => {
        const changed = {};
        for (const el of document.querySelectorAll('[data-lsx-observe]')) {
            const tag = el.getAttribute('data-lsx-observe');
            const rtl = direction(el);
            if (sent.get(tag) !== rtl) { sent.set(tag, rtl); changed[tag] = rtl; }
        }
        if (Object.keys(changed).length > 0) dioxus.send(changed);
    };
    window.__lsxDirection = (tag) => {
        const el = document.querySelector(`[data-lsx-observe='${tag}']`);
        if (el) { const rtl = direction(el); sent.set(String(tag), rtl); dioxus.send({ [tag]: rtl }); }
    };
    new MutationObserver(scan).observe(document.documentElement, {
        subtree: true, attributes: true, attributeFilter: ['dir'],
    });
    scan();
    await new Promise(() => {});";

/// Starts mirroring the tagged elements' direction into Rust, once per dom, so a
/// handler reads it synchronously (1337).
pub(crate) fn watch_direction() {
    if !runs_scripts() {
        return;
    }
    let Some(page) = page() else {
        return;
    };
    if !page.watching_direction.replace(true) {
        let script = eval(ON_DIRECTION);
        spawn_forever(async move {
            let mut script = script;
            while let Ok(changed) = script.recv::<HashMap<String, bool>>().await {
                let mut mirror = page.rtl.borrow_mut();
                for (tag, rtl) in changed {
                    if let Ok(tag) = tag.parse() {
                        mirror.insert(tag, rtl);
                    }
                }
            }
        });
    }
}

/// Reads `tag`'s direction once its element mounted, which the `dir` observer never sees.
pub(crate) fn read_rtl_tag(tag: u64) {
    if runs_scripts() {
        eval_with(json!(tag), "window.__lsxDirection?.(data);");
    }
}

/// Drops what the mirror holds for `tag`, once its handle is gone.
pub(crate) fn forget_rtl_tag(tag: u64) {
    if let Some(page) = page() {
        page.rtl.borrow_mut().remove(&tag);
        page.rtl_aliases.borrow_mut().remove(&tag);
    }
}

/// Whether the element tagged `tag` laid out right to left when the page last said.
pub(crate) fn is_rtl_by_tag(tag: u64) -> bool {
    page().is_some_and(|page| {
        let aliases = page.rtl_aliases.borrow();
        let tag = aliases.get(&tag).unwrap_or(&tag);
        page.rtl.borrow().get(tag) == Some(&true)
    })
}

/// Makes `tag` answer [`is_rtl_by_tag`] as `source`'s element does, for a handle that follows it.
pub(crate) fn alias_rtl_tag(tag: u64, source: u64) {
    if let Some(page) = page() {
        page.rtl_aliases.borrow_mut().insert(tag, source);
    }
}
