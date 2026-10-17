use std::rc::Rc;

use dioxus::prelude::spawn;
use serde_json::json;

use super::{Slot, WebViewListener, eval_with, runs_scripts};
use crate::platform::{
    ContentSubscription, INTERSECT_ATTR, OBSERVE_ATTR, PlatformError, Read, TagCommand,
};

/// An `IntersectionObserver` on the element carrying the tag, started once it
/// renders. Sends `[isIntersecting, ratio]` of the newest entry; a root without
/// a tag (or one that never renders) falls back to the viewport.
const ON_INTERSECTION: &str = "const [attr, tag, rootAttr, rootTag, margin, thresholds] = data;
    const find = (name, value) => document.querySelector('[' + name + '=\"' + value + '\"]');
    let observer = null;
    const start = () => {
        const target = find(attr, tag);
        if (!target) return false;
        const root = rootTag === null ? null : find(rootAttr, rootTag);
        if (rootTag !== null && !root) console.warn('use_intersection: the root carries no attribute');
        observer = new IntersectionObserver((entries) => {
            const latest = entries[entries.length - 1];
            dioxus.send([latest.isIntersecting, latest.intersectionRatio]);
        }, { root, rootMargin: margin, threshold: thresholds });
        observer.observe(target);
        return true;
    };
    const waiting = start() ? null : new MutationObserver(() => {
        if (start()) waiting.disconnect();
    });
    waiting?.observe(document.documentElement, { childList: true, subtree: true, attributes: true, attributeFilter: [attr] });";

/// A computed CSS length of the element tagged `tag` (no handle reaches the DOM
/// here, 958); `Unsupported` while no element carries the tag.
pub(crate) fn computed_px(tag: u64, property: &str) -> Read<Option<f64>> {
    if !runs_scripts() {
        return Box::pin(std::future::ready(Err(PlatformError::Unsupported)));
    }
    let read = eval_with(
        json!([OBSERVE_ATTR, tag.to_string(), property]),
        "const [attr, tag, property] = data;
        const element = document.querySelector(`[${attr}=\"${tag}\"]`);
        if (!element) return [false, null];
        const value = getComputedStyle(element).getPropertyValue(property);
        return [true, value.endsWith('px') ? parseFloat(value) : null];",
    );
    Box::pin(async move {
        match read.join::<(bool, Option<f64>)>().await {
            Ok((true, px)) => Ok(px.filter(|px| px.is_finite())),
            _ => Err(PlatformError::Unsupported),
        }
    })
}

/// Queued, like [`focus_selector`]; nothing happens while no element carries the tag.
pub(crate) fn command_by_tag(tag: u64, command: TagCommand) -> Result<(), PlatformError> {
    if !runs_scripts() {
        return Err(PlatformError::Unsupported);
    }
    let method = match command {
        TagCommand::Click => "click",
        TagCommand::Select => "select",
        TagCommand::Mixed => "mixed",
        TagCommand::Unmixed => "unmixed",
    };
    eval_with(
        json!([OBSERVE_ATTR, tag.to_string(), method]),
        "const [attr, tag, method] = data;
        const element = document.querySelector(`[${attr}=\"${tag}\"]`);
        if (method === 'mixed' || method === 'unmixed') {
            if (element) element.indeterminate = method === 'mixed';
        } else {
            element?.[method]?.();
        }",
    );
    Ok(())
}

/// Whether a descendant of the element tagged `tag` matches `selector`;
/// `Unsupported` while no element carries the tag.
pub(crate) fn has_match(tag: u64, selector: &str) -> Read<bool> {
    if !runs_scripts() {
        return Box::pin(std::future::ready(Err(PlatformError::Unsupported)));
    }
    let read = eval_with(
        json!([OBSERVE_ATTR, tag.to_string(), selector]),
        "const [attr, tag, selector] = data;
        const element = document.querySelector(`[${attr}=\"${tag}\"]`);
        if (!element) return [false, false];
        return [true, element.querySelector(selector) !== null];",
    );
    Box::pin(async move {
        match read.join::<(bool, bool)>().await {
            Ok((true, found)) => Ok(found),
            _ => Err(PlatformError::Unsupported),
        }
    })
}

/// Observed in the page, keyed by the target's `INTERSECT_ATTR` and the root
/// handle's `OBSERVE_ATTR`: it clips by every scroller up to the root and sees
/// layout-only changes.
pub(crate) fn on_intersection(
    tags: (u64, Option<u64>),
    root_margin: &str,
    thresholds: &[f64],
    callback: Box<dyn Fn(bool, f64)>,
) -> Option<Box<dyn ContentSubscription>> {
    if !runs_scripts() {
        return None;
    }
    let slot = Slot::new();
    let (tag, root_tag) = tags;
    let script = eval_with(
        json!([
            INTERSECT_ATTR,
            tag.to_string(),
            OBSERVE_ATTR,
            root_tag.map(|tag| tag.to_string()),
            root_margin,
            thresholds
        ]),
        &format!(
            "{ON_INTERSECTION}
            {}
            observer?.disconnect();
            waiting?.disconnect();",
            slot.park("")
        ),
    );
    let task = spawn(async move {
        let mut script = script;
        while let Ok((is_intersecting, ratio)) = script.recv::<(bool, f64)>().await {
            callback(is_intersecting, ratio);
        }
    });
    Some(Box::new(WebViewListener {
        task,
        _slot: Rc::new(slot),
    }))
}

/// A `ResizeObserver` on the element carrying `tag`'s `OBSERVE_ATTR`, started once it renders.
pub(crate) fn observe_resize(
    tag: u64,
    callback: Box<dyn Fn()>,
) -> Option<Box<dyn ContentSubscription>> {
    observe_tagged(
        tag,
        "observer = new ResizeObserver(() => dioxus.send(true));
        observer.observe(target);",
        callback,
    )
}

/// A `MutationObserver` on the subtree of the element carrying `tag`'s `OBSERVE_ATTR`,
/// as the web's `on_content_change` (963): one call per batch of records.
pub(crate) fn observe_content(
    tag: u64,
    callback: Box<dyn Fn()>,
) -> Option<Box<dyn ContentSubscription>> {
    observe_tagged(
        tag,
        "observer = new MutationObserver(() => dioxus.send(true));
        observer.observe(target, { childList: true, subtree: true, characterData: true,
            attributeFilter: ['tabindex', 'disabled', 'href', 'contenteditable', 'hidden', 'open', 'class'] });",
        callback,
    )
}

/// Runs `observe`, which sets `observer` watching `target`, once the element carrying
/// `tag`'s `OBSERVE_ATTR` renders; each `dioxus.send` calls `callback`.
fn observe_tagged(
    tag: u64,
    observe: &'static str,
    callback: Box<dyn Fn()>,
) -> Option<Box<dyn ContentSubscription>> {
    if !runs_scripts() {
        return None;
    }
    let slot = Slot::new();
    let script = eval_with(
        json!([OBSERVE_ATTR, tag.to_string()]),
        &format!(
            "const [attr, tag] = data;
            let observer = null;
            const start = () => {{
                const target = document.querySelector('[' + attr + '=\"' + tag + '\"]');
                if (!target) return false;
                {observe}
                return true;
            }};
            const waiting = start() ? null : new MutationObserver(() => {{
                if (start()) waiting.disconnect();
            }});
            waiting?.observe(document.documentElement, {{ childList: true, subtree: true, attributes: true, attributeFilter: [attr] }});
            {}
            observer?.disconnect();
            waiting?.disconnect();",
            slot.park("")
        ),
    );
    let task = spawn(async move {
        let mut script = script;
        while script.recv::<bool>().await.is_ok() {
            callback();
        }
    });
    Some(Box::new(WebViewListener {
        task,
        _slot: Rc::new(slot),
    }))
}

impl ContentSubscription for WebViewListener {}
