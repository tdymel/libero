use dioxus::prelude::*;

use crate::{
    hooks::{use_media_query, use_subscription_slot},
    platform::{ScrollSubscription, document, media_query, on_viewport_resize},
    theme::Size,
    utils::bump,
};

use super::{
    core::{CascaderLayout, narrow_query},
    nodes::parent_pick,
    option::CascaderNode,
};

/// The narrow list's parent row ("Select Europe") as a keyboard row, above the first row of the last
/// column. It marks the cursor it was reached from, so any other move leaves it.
#[derive(Clone, Copy)]
pub(super) struct ParentRow {
    narrow: Memo<bool>,
    above: Signal<Option<Vec<usize>>>,
}

/// `false` where nothing can measure the viewport (a server render): the row stays a touch row.
pub(super) fn use_parent_row() -> ParentRow {
    let media = use_media_query(&narrow_query());
    let window = use_narrow_window();
    ParentRow {
        narrow: use_memo(move || media() || window()),
        above: use_signal(|| None),
    }
}

/// [`narrow_query`] where no media query can be asked (Blitz): the window's CSS width against `Sm`,
/// re-read on each viewport resize. A media query's `rem` is the initial 16px, whatever the root's.
fn use_narrow_window() -> Signal<bool> {
    let mut narrow = use_signal(|| false);
    let tick = use_signal(|| 0u64);
    let resizing = use_subscription_slot::<dyn ScrollSubscription>();
    use_effect(move || {
        let _ = tick();
        if media_query().is_some() {
            return;
        }
        let Some(document) = document() else {
            return;
        };
        if !resizing.is_some() {
            resizing.set(on_viewport_resize(Box::new(move || bump(tick))));
        }
        let read = document.viewport();
        spawn(async move {
            if let Ok(viewport) = read.await {
                let now = viewport.width < breakpoint_px(Size::Sm);
                if *narrow.peek() != now {
                    narrow.set(now);
                }
            }
        });
    });
    narrow
}

fn breakpoint_px(size: Size) -> f64 {
    let rem = size.breakpoint_value().trim_end_matches("rem");
    rem.parse::<f64>().map_or(f64::INFINITY, |rem| rem * 16.0)
}

impl ParentRow {
    /// The path the highlighted row picks, while the highlight is on the parent row.
    pub(super) fn picks(
        &self,
        nodes: &[CascaderNode],
        any_level: bool,
        layout: CascaderLayout,
        cursor: &[usize],
    ) -> Option<Vec<usize>> {
        let on = layout == CascaderLayout::Columns
            && (self.narrow)()
            && self.above.read().as_deref() == Some(cursor);
        parent_pick(nodes, any_level, cursor).filter(|_| on)
    }

    /// Lifts the highlight onto the parent row of `cursor`, when the narrow list draws one.
    pub(super) fn reach(&self, nodes: &[CascaderNode], any_level: bool, cursor: &[usize]) {
        if (self.narrow)() && parent_pick(nodes, any_level, cursor).is_some() {
            let mut above = self.above;
            above.set(Some(cursor.to_vec()));
        }
    }

    /// Back to the cursor's own row.
    pub(super) fn leave(&self) {
        if self.above.peek().is_some() {
            let mut above = self.above;
            above.set(None);
        }
    }
}
