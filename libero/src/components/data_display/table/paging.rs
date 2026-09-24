use std::ops::Range;

use dioxus::prelude::*;

use super::{core::TableSort, use_table::TableState};
use crate::{
    components::{
        accessibility::use_announcer,
        common::{HtmlTag, Options},
        form::Select,
        layout::use_box,
        navigation::Pagination,
    },
    hooks::use_localization,
    sx::{StaticSx, sx},
    theme::Size,
};

static PAGER_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_wrap("wrap")
        .align_items("center")
        .justify_content("flex-end")
        .gap("8px 16px")
        .padding("8px 0 0")
});

/// The label beside its picker, not above it.
static PAGE_SIZE_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("row")
        .align_items("center")
        .gap("8px")
});

/// Pages for `total` rows; none for no rows, as `Pagination` draws nothing then.
pub(super) fn page_count(total: usize, size: usize) -> u32 {
    u32::try_from(total.div_ceil(size.max(1))).unwrap_or(u32::MAX)
}

/// `page`, 1-based, clamped into the pages `total` rows fill.
pub(super) fn clamp_page(total: usize, page: u32, size: usize) -> u32 {
    page.clamp(1, page_count(total, size).max(1))
}

/// The rows of an already clamped `page`, as indices into the display order.
pub(super) fn page_rows(total: usize, page: u32, size: usize) -> Range<usize> {
    let size = size.max(1);
    let start = (page.saturating_sub(1) as usize)
        .saturating_mul(size)
        .min(total);
    start..start.saturating_add(size).min(total)
}

/// The page that shows row `row` (0-based) at `size` rows a page.
pub(super) fn page_of_row(row: usize, size: usize) -> u32 {
    u32::try_from(row / size.max(1) + 1).unwrap_or(u32::MAX)
}

/// Back to page 1 when the sort changes: the rows the reader paged to are gone.
pub(super) fn use_page_reset(state: TableState) {
    let mut last = use_signal(|| state.sort.read());
    use_effect(move || {
        let sort: Vec<TableSort> = state.sort.read();
        if *last.peek() == sort {
            return;
        }
        last.set(sort);
        if state.page.peek() != 1 {
            state.page.set(1);
        }
    });
}

/// One choice of the page-size picker.
#[derive(Clone, PartialEq)]
struct PageSize(usize);

impl Options for PageSize {
    fn label(&self) -> String {
        self.0.to_string()
    }
}

/// Everything the footer needs, once `T` is gone.
#[derive(Clone, PartialEq, Props)]
pub(super) struct TablePagerProps {
    pub state: TableState,
    /// Rows over all pages.
    pub total: usize,
    /// Clamped.
    pub page: u32,
    pub page_size: usize,
    pub page_sizes: Vec<usize>,
    pub caption: Option<String>,
    pub size: Size,
}

/// The page-size picker, the shown rows' range and the page buttons.
#[component]
pub(super) fn TablePager(props: TablePagerProps) -> Element {
    let labels = use_localization().table;
    let TablePagerProps {
        state,
        total,
        page,
        page_size,
        ..
    } = props;
    let rows = page_rows(total, page, page_size);
    let from = if rows.is_empty() { 0 } else { rows.start + 1 };
    let range = (labels.range)(from, rows.end, total);
    let aria_label = match &props.caption {
        Some(caption) => (labels.pages_of)(caption),
        None => labels.pages.to_string(),
    };
    // Said on a change only: the first render's range is no news.
    let announcer = use_announcer();
    let mut said = use_signal(|| range.clone());
    let shown = range.clone();
    use_effect(use_reactive!(|(shown,)| {
        if *said.peek() != shown {
            said.set(shown.clone());
            announcer.say(shown);
        }
    }));
    let first = rows.start;
    let pick = move |next: Option<PageSize>| {
        let Some(PageSize(next)) = next else {
            return;
        };
        // Keeps the first shown row on screen.
        let page_next = page_of_row(first, next);
        state.page_size.set(next);
        if page_next != page {
            state.page.set(page_next);
        }
    };
    let pager = use_box().framework_sx(&PAGER_SX).prepare();
    pager.render(
        HtmlTag::Div,
        vec![],
        rsx! {
            if !props.page_sizes.is_empty() {
                Select {
                    label: labels.rows_per_page,
                    options: props.page_sizes.iter().map(|&size| PageSize(size)).collect::<Vec<_>>(),
                    value: Some(PageSize(page_size)),
                    onchange: pick,
                    size: props.size,
                    sx: &PAGE_SIZE_SX,
                }
            }
            span { "data-slot": "range", "{range}" }
            {announcer.render()}
            Pagination {
                total: page_count(total, page_size),
                page,
                onchange: move |next| state.page.set(next),
                aria_label,
                size: props.size,
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_partial_pages() {
        assert_eq!(page_count(0, 10), 0);
        assert_eq!(page_count(10, 10), 1);
        assert_eq!(page_count(11, 10), 2);
        assert_eq!(page_count(5, 0), 5);
    }

    #[test]
    fn clamps_the_page_into_range() {
        assert_eq!(clamp_page(25, 0, 10), 1);
        assert_eq!(clamp_page(25, 9, 10), 3);
        assert_eq!(clamp_page(0, 4, 10), 1);
    }

    #[test]
    fn slices_the_last_page_short() {
        assert_eq!(page_rows(25, 1, 10), 0..10);
        assert_eq!(page_rows(25, 3, 10), 20..25);
        assert_eq!(page_rows(0, 1, 10), 0..0);
    }

    #[test]
    fn a_new_size_keeps_the_first_row_in_view() {
        // Rows 21-30 at 10 a page; at 25 a page row 21 is on page 1, at 5 on page 5.
        assert_eq!(page_of_row(20, 25), 1);
        assert_eq!(page_of_row(20, 5), 5);
        assert_eq!(page_of_row(0, 50), 1);
    }
}
