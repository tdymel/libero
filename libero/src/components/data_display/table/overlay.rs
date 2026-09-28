use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, attr},
        feedback::{ProgressBar, Skeleton},
        layout::use_box,
    },
    localization::TableLabels,
    sx::{StaticSx, sx},
};

/// Skeleton rows of a `loading` table that is not paged.
pub(super) const SKELETON_ROWS: usize = 5;

/// What a table without rows to show draws in its body.
pub(super) enum EmptyBody {
    /// One full-width row: `empty`, `no_results` or their localized text.
    Message(Element),
    /// Placeholder rows while `loading`.
    Skeleton(usize),
}

impl EmptyBody {
    /// `columns` counts the leading columns too.
    pub fn render(self, columns: usize) -> Element {
        match self {
            Self::Message(body) => rsx! {
                tr { "data-empty": true,
                    td { colspan: "{columns}", {body} }
                }
            },
            // Only a look: the busy table tells assistive tech.
            Self::Skeleton(rows) => rsx! {
                for row in 0..rows {
                    tr { key: "skeleton-{row}", "data-skeleton": true, aria_hidden: "true",
                        for cell in 0..columns {
                            td { key: "{cell}",
                                Skeleton { height: "1em" }
                            }
                        }
                    }
                }
            },
        }
    }
}

/// Zero height, so the bar lies over the table's top edge without moving it.
static LOADING_BAR_SX: StaticSx = StaticSx::new(|| {
    sx().position("relative").height("0").z_index("4").selector(
        "& > *",
        sx().position("absolute").top("0").with("inset-inline", "0"),
    )
});

/// The bar over a `loading` table that keeps its rows, which stay usable.
#[component]
pub(super) fn LoadingBar(labels: TableLabels) -> Element {
    use_box().framework_sx(&LOADING_BAR_SX).prepare().render(
        HtmlTag::Div,
        vec![attr("data-loading-bar", "true")],
        rsx! {
            ProgressBar { size: "xs", aria_label: labels.loading }
        },
    )
}
