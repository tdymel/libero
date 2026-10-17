use std::{
    rc::{Rc, Weak},
    time::Duration,
};

use blitz_dom::BaseDocument;
use dioxus::prelude::*;
use dioxus_native_dom::{NodeHandle, NodeId};

use super::{
    BlitzElement, Doc, client_rect, doc, node_of, remember_document, resize, resolved_style_value,
    sticky,
};
use crate::platform::{
    SCROLL_MARGIN_BOTTOM_VAR, SCROLL_MARGIN_VAR, SCROLL_PADDING_VARS, ScrollApi, ScrollSubscription,
};

pub(in crate::platform::backend) fn scroll() -> Option<&'static dyn ScrollApi> {
    Some(&SCROLL)
}

/// Blitz's `scroll` does not bubble: a wheel reaching [`Listener`](super::Listener) and libero's
/// own scroll commands report instead. Keys and drags scroll nothing there.
pub(super) struct BlitzScroll;

static SCROLL: BlitzScroll = BlitzScroll;

pub(super) fn notify_scroll() {
    let Some(doc) = doc() else {
        return;
    };
    sticky::sync_soon();
    resize::scrolled();
    doc.scroll_callbacks.each(|callback| callback());
}

impl ScrollApi for BlitzScroll {
    fn on_scroll(&self, callback: Box<dyn Fn()>) -> Box<dyn ScrollSubscription> {
        let Some(doc) = doc() else {
            return Box::new(BlitzScrollSubscription(0, Weak::new()));
        };
        let id = doc.scroll_callbacks.add(Rc::from(callback));
        Box::new(BlitzScrollSubscription(id, Rc::downgrade(&doc)))
    }
}

struct BlitzScrollSubscription(u64, Weak<Doc>);

impl ScrollSubscription for BlitzScrollSubscription {}

impl Drop for BlitzScrollSubscription {
    fn drop(&mut self) {
        if let Some(doc) = self.1.upgrade() {
            doc.scroll_callbacks.remove(self.0);
        }
    }
}

/// An effect runs before the shell lays out what it rendered, so a node
/// mounted this frame has no box yet: retried after a frame, a few times.
pub(super) const LAYOUT_TRIES: u8 = 3;
const LAYOUT_WAIT: Duration = Duration::from_millis(20);

/// Every scroller round the node, on both axes, then the viewport (todo 2280).
/// `None` for a node that is not Blitz's.
pub(in crate::platform::backend) fn scroll_chain_into_view(
    mounted: &Rc<MountedData>,
) -> Option<()> {
    let (anchor, node_id) = node_of(mounted)?;
    remember_document(&anchor);
    show(anchor, node_id, LAYOUT_TRIES, true);
    Some(())
}

/// `chain` moves every scroller round it on both axes, as [`reveal`]; otherwise
/// only the nearest vertical one.
pub(super) fn show(anchor: NodeHandle, node_id: NodeId, tries: u8, chain: bool) {
    let element = BlitzElement {
        anchor: anchor.clone(),
        node_id,
    };
    element.command(move |doc| {
        let unlaid = doc.get_node(node_id).is_some_and(|node| {
            let size = node.final_layout().size;
            size.width == 0.0 && size.height == 0.0
        });
        if unlaid && tries > 0 {
            let retry = crate::platform::backend::thread::timer().map(|timer| {
                timer.after(
                    LAYOUT_WAIT,
                    Box::new(move || show(anchor, node_id, tries - 1, chain)),
                )
            });
            // The latest call wins, as a second scroll would override the first.
            if let Some(state) = self::doc() {
                drop(state.show_retry.replace(retry));
            }
        } else if chain {
            reveal(doc, node_id);
        } else if let Some((scroller, delta)) = into_view(doc, node_id) {
            match scroller {
                Some(scroller) => {
                    doc.scroll_node_by(scroller, 0.0, -delta, |_| {});
                }
                None => {
                    doc.scroll_viewport_by(0.0, -delta);
                }
            }
            notify_scroll();
        }
    });
}

/// `WebElement::scroll_into_view`'s walk: the nearest `overflow-y` scroller and
/// how far it scrolls, [`SCROLL_MARGIN_VAR`] for `scroll-margin`. `None` for the
/// viewport, when no element scrolls.
fn into_view(doc: &BaseDocument, node_id: NodeId) -> Option<(Option<NodeId>, f64)> {
    let (_, y, width, height) = client_rect(doc, node_id)?;
    if width == 0.0 && height == 0.0 {
        return None;
    }
    let [margin, bottom] = scroll_margin(doc, node_id);
    let (y, height) = (y - margin, height + margin + bottom);
    let mut ancestor = doc.get_node(node_id)?.parent;
    let scroller = loop {
        let Some(id) = ancestor else {
            let (window, scale) = (doc.viewport().window_size, doc.viewport().scale_f64());
            let [top, _, bottom, _] = viewport_padding(doc);
            let view_bottom = f64::from(window.1) / scale - bottom;
            let delta = crate::platform::backend::nearest_scroll(y, y + height, top, view_bottom)?;
            return Some((None, delta));
        };
        let node = doc.get_node(id)?;
        if node.is_element()
            && node.final_layout().scroll_height() > 0.0
            && matches!(
                resolved_style_value(doc, node.id, "overflow-y").as_str(),
                "auto" | "scroll"
            )
        {
            break node;
        }
        ancestor = node.parent;
    };
    let layout = scroller.final_layout();
    let view_top = client_rect(doc, scroller.id)?.1 + f64::from(layout.border.top);
    let client_height = layout.size.height
        - layout.border.top
        - layout.border.bottom
        - layout.scrollbar_size.height;
    let delta = crate::platform::backend::nearest_scroll(
        y,
        y + height,
        view_top,
        view_top + f64::from(client_height),
    )?;
    Some((Some(scroller.id), delta))
}

/// The web's focus scroll, which Blitz lacks: every scroller round `node_id`,
/// innermost first, then the viewport, moves just far enough on both axes.
pub(in crate::platform::backend) fn reveal(doc: &mut BaseDocument, node_id: NodeId) {
    let Some((_, _, width, height)) = client_rect(doc, node_id) else {
        return;
    };
    if width == 0.0 && height == 0.0 {
        return;
    }
    let [margin, margin_bottom] = scroll_margin(doc, node_id);
    let nearest = |start: f64, size: f64, view: f64, view_size: f64, after: f64| {
        crate::platform::backend::nearest_scroll(
            start - margin,
            start + size + after,
            view,
            view + view_size,
        )
        .unwrap_or(0.0)
    };
    let mut moved = false;
    let mut ancestor = doc.get_node(node_id).and_then(|node| node.parent);
    while let Some(id) = ancestor {
        let Some(node) = doc.get_node(id) else {
            break;
        };
        ancestor = node.parent;
        let layout = *node.final_layout();
        let scrolls = |axis: &str, range: f32| {
            range > 0.0
                && matches!(
                    resolved_style_value(doc, id, axis).as_str(),
                    "auto" | "scroll" | "hidden"
                )
        };
        let x = node.is_element() && scrolls("overflow-x", layout.scroll_width());
        let y = node.is_element() && scrolls("overflow-y", layout.scroll_height());
        if !x && !y {
            continue;
        }
        let (Some(target), Some(view)) = (client_rect(doc, node_id), client_rect(doc, id)) else {
            break;
        };
        let (border, bar) = (layout.border, layout.scrollbar_size);
        let view_width = f64::from(layout.size.width - border.left - border.right - bar.width);
        let view_height = f64::from(layout.size.height - border.top - border.bottom - bar.height);
        let [top, right, bottom, left] = scroll_padding(doc, id);
        let dx = match x {
            true => nearest(
                target.0,
                target.2,
                view.0 + f64::from(border.left) + left,
                view_width - left - right,
                margin,
            ),
            false => 0.0,
        };
        let dy = match y {
            true => nearest(
                target.1,
                target.3,
                view.1 + f64::from(border.top) + top,
                view_height - top - bottom,
                margin_bottom,
            ),
            false => 0.0,
        };
        if dx != 0.0 || dy != 0.0 {
            doc.scroll_node_by(id, -dx, -dy, |_| {});
            moved = true;
        }
    }
    if let Some((x, y, width, height)) = client_rect(doc, node_id) {
        let (window, scale) = (doc.viewport().window_size, doc.viewport().scale_f64());
        let [top, right, bottom, left] = viewport_padding(doc);
        let dx = nearest(
            x,
            width,
            left,
            f64::from(window.0) / scale - left - right,
            margin,
        );
        let dy = nearest(
            y,
            height,
            top,
            f64::from(window.1) / scale - top - bottom,
            margin_bottom,
        );
        if dx != 0.0 || dy != 0.0 {
            doc.scroll_viewport_by(-dx, -dy);
            moved = true;
        }
    }
    if moved {
        notify_scroll();
    }
}

/// [`SCROLL_PADDING_VARS`] and [`SCROLL_MARGIN_VAR`] as `@property` rules that do not inherit,
/// so a nested scroller or a descendant gets none of its ancestor's padding or margin.
/// The bottom margin stays unregistered-like (`*`, no initial value): unset is told from `0`.
pub(in crate::platform::backend) fn scroll_padding_properties() -> &'static str {
    static CSS: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        let lengths = SCROLL_PADDING_VARS
            .into_iter()
            .chain([SCROLL_MARGIN_VAR])
            .map(|var| {
                format!("@property {var}{{syntax:'<length>';inherits:false;initial-value:0px;}}")
            })
            .collect::<String>();
        format!("{lengths}@property {SCROLL_MARGIN_BOTTOM_VAR}{{syntax:'*';inherits:false;}}")
    });
    &CSS
}

/// A scroller's own [`SCROLL_PADDING_VARS`] in px.
fn scroll_padding(doc: &BaseDocument, node_id: NodeId) -> [f64; 4] {
    SCROLL_PADDING_VARS.map(|var| px_var(doc, node_id, var))
}

/// The viewport's [`SCROLL_PADDING_VARS`] in px: the root's, as a published Header sets them.
fn viewport_padding(doc: &BaseDocument) -> [f64; 4] {
    scroll_padding(doc, doc.root_element().id)
}

/// `[margin, bottom]` in px: [`SCROLL_MARGIN_VAR`] on every side, the bottom
/// [`SCROLL_MARGIN_BOTTOM_VAR`] instead when set.
fn scroll_margin(doc: &BaseDocument, node_id: NodeId) -> [f64; 2] {
    let margin = px_var(doc, node_id, SCROLL_MARGIN_VAR);
    let bottom = resolved_style_value(doc, node_id, SCROLL_MARGIN_BOTTOM_VAR);
    match bottom.trim().is_empty() {
        true => [margin, margin],
        false => [margin, px_var(doc, node_id, SCROLL_MARGIN_BOTTOM_VAR)],
    }
}

/// A custom property in px: a `px` length, or `rem` of the root's font size.
/// `0` for anything else.
fn px_var(doc: &BaseDocument, node_id: NodeId, var: &str) -> f64 {
    let value = resolved_style_value(doc, node_id, var);
    let value = value.trim();
    let px = |value: &str| value.trim().parse::<f64>().ok();
    if let Some(rem) = value.strip_suffix("rem") {
        let root = resolved_style_value(doc, doc.root_element().id, "font-size");
        let root = root.strip_suffix("px").and_then(px).unwrap_or(16.0);
        return px(rem).map_or(0.0, |rem| rem * root);
    }
    value.strip_suffix("px").and_then(px).unwrap_or(0.0)
}
