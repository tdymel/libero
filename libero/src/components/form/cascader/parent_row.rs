use dioxus::prelude::*;

use crate::hooks::use_media_query;

use super::{
    core::{CascaderLayout, narrow_query},
    nodes::parent_pick,
    option::CascaderNode,
};

/// The narrow list's parent row ("Select Europe") as a keyboard row, above the first row of the last
/// column. It marks the cursor it was reached from, so any other move leaves it.
#[derive(Clone, Copy)]
pub(super) struct ParentRow {
    narrow: ReadSignal<bool>,
    above: Signal<Option<Vec<usize>>>,
}

/// `false` where nothing can measure the viewport (a server render, Blitz): the row stays a touch row.
pub(super) fn use_parent_row() -> ParentRow {
    ParentRow {
        narrow: use_media_query(&narrow_query()),
        above: use_signal(|| None),
    }
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
