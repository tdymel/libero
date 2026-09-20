//! `Pagination`'s ellipsis range, public so a custom strip can reuse it.

/// One slot in the strip; the arrow controls are the component's, not the range's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaginationItem {
    Page(u32),
    Ellipsis,
}

/// Which page numbers to draw and where the gaps go. An ellipsis never hides
/// exactly one page, so the strip's length is the same for every `page`.
///
/// `total` and `boundaries` of 0 count as 1; `page` is clamped into range.
///
/// ```
/// # use libero::components::{PaginationItem, pagination_range};
/// use PaginationItem::{Ellipsis, Page};
/// assert_eq!(
///     pagination_range(10, 6, 1, 1),
///     [Page(1), Ellipsis, Page(5), Page(6), Page(7), Ellipsis, Page(10)],
/// );
/// ```
pub fn pagination_range(
    total: u32,
    page: u32,
    siblings: u8,
    boundaries: u8,
) -> Vec<PaginationItem> {
    let total = total.max(1);
    let page = page.clamp(1, total);
    let siblings = u32::from(siblings);
    let boundaries = u32::from(boundaries).max(1);

    // Every page fits, so there is nothing to hide.
    let window = 2 * siblings + 2 * boundaries + 3;
    if total <= window {
        return (1..=total).map(PaginationItem::Page).collect();
    }

    // `total > window` keeps both clamp ranges `total - window` wide, so nothing
    // inverts or underflows and `right - left` is always `2 * siblings`.
    let left = (page.saturating_sub(siblings))
        .clamp(boundaries + 2, total - boundaries - 2 * siblings - 1);
    let right = page
        .saturating_add(siblings)
        .clamp(boundaries + 2 * siblings + 2, total - boundaries - 1);

    let mut items = Vec::with_capacity(window as usize);
    items.extend((1..=boundaries).map(PaginationItem::Page));

    // A gap of exactly one page is printed as that page instead.
    items.push(match left > boundaries + 2 {
        true => PaginationItem::Ellipsis,
        false => PaginationItem::Page(boundaries + 1),
    });

    items.extend((left..=right).map(PaginationItem::Page));

    items.push(match right < total - boundaries - 1 {
        true => PaginationItem::Ellipsis,
        false => PaginationItem::Page(total - boundaries),
    });

    items.extend((total - boundaries + 1..=total).map(PaginationItem::Page));
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The strip as a reader sees it, so a failure prints the row rather than a
    /// `Vec` of variants.
    fn strip(total: u32, page: u32, siblings: u8, boundaries: u8) -> String {
        pagination_range(total, page, siblings, boundaries)
            .into_iter()
            .map(|item| match item {
                PaginationItem::Page(n) => n.to_string(),
                PaginationItem::Ellipsis => "…".to_string(),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// The plan's pinned table, every row. The `total: 10, page: 7` row never
    /// hides a single page behind a gap the same width.
    #[test]
    fn the_pinned_table() {
        let rows: &[(u32, u8, u8, u32, &str)] = &[
            (7, 1, 1, 4, "1 2 3 4 5 6 7"),
            (10, 1, 1, 1, "1 2 3 4 5 … 10"),
            (10, 1, 1, 4, "1 2 3 4 5 … 10"),
            (10, 1, 1, 5, "1 … 4 5 6 … 10"),
            (10, 1, 1, 6, "1 … 5 6 7 … 10"),
            // Not `1 … 6 7 8 … 10`, which would hide only `9`.
            (10, 1, 1, 7, "1 … 6 7 8 9 10"),
            (10, 1, 1, 10, "1 … 6 7 8 9 10"),
            (20, 2, 1, 10, "1 … 8 9 10 11 12 … 20"),
            (20, 1, 2, 10, "1 2 … 9 10 11 … 19 20"),
            (20, 1, 2, 3, "1 2 3 4 5 6 … 19 20"),
            (11, 0, 1, 6, "1 … 6 … 11"),
            (1, 1, 1, 1, "1"),
            // Out of range, clamped rather than panicking or rendering empty.
            (10, 1, 1, 99, "1 … 6 7 8 9 10"),
        ];

        for &(total, siblings, boundaries, page, expected) in rows {
            assert_eq!(
                strip(total, page, siblings, boundaries),
                expected,
                "total={total} siblings={siblings} boundaries={boundaries} page={page}"
            );
        }
    }

    /// `page + siblings` overflowed here. The strip keeps its width and ends on
    /// the last page.
    #[test]
    fn the_last_pages_of_u32_max_do_not_overflow() {
        let max = u32::MAX;
        for siblings in [0, 1, 2, u8::MAX] {
            for page in [max, max - 1, max - u32::from(siblings)] {
                let items = pagination_range(max, page, siblings, 1);
                assert_eq!(
                    items.len(),
                    2 * usize::from(siblings) + 5,
                    "siblings={siblings} page={page}"
                );
                assert_eq!(items.last(), Some(&PaginationItem::Page(max)));
            }
        }
        assert_eq!(
            strip(max, max, 1, 1),
            format!("1 … {} {} {} {} {max}", max - 4, max - 3, max - 2, max - 1)
        );
        assert_eq!(strip(max, max - 1, 2, 1), strip(max, max, 2, 1));
    }

    /// `boundaries: 0` is clamped to 1. Both references emit leading dots with
    /// nothing outside them at 0, which hides a page to show a gap.
    #[test]
    fn zero_boundaries_is_clamped_to_one() {
        assert_eq!(strip(20, 10, 1, 0), strip(20, 10, 1, 1));
        assert!(strip(20, 10, 1, 0).starts_with("1 …"));
    }

    /// Order, no one-page gap and a constant width, over a sweep of inputs.
    #[test]
    fn the_invariants_hold_across_the_sweep() {
        for total in 1..=39u32 {
            for siblings in 0..=3u8 {
                for boundaries in 1..=3u8 {
                    let window = 2 * u32::from(siblings) + 2 * u32::from(boundaries) + 3;
                    let mut widths = std::collections::BTreeSet::new();

                    for page in 1..=total {
                        let items = pagination_range(total, page, siblings, boundaries);
                        let at = |i: usize| items[i];
                        let where_ = format!(
                            "total={total} siblings={siblings} boundaries={boundaries} page={page}"
                        );

                        // Pages strictly increase.
                        let pages: Vec<u32> = items
                            .iter()
                            .filter_map(|item| match item {
                                PaginationItem::Page(n) => Some(*n),
                                PaginationItem::Ellipsis => None,
                            })
                            .collect();
                        assert!(
                            pages.windows(2).all(|pair| pair[0] < pair[1]),
                            "pages out of order at {where_}: {pages:?}"
                        );

                        // An ellipsis stands for two pages or more; boundary pages
                        // flank each one, so `index ± 1` is in bounds.
                        for index in 0..items.len() {
                            if at(index) != PaginationItem::Ellipsis {
                                continue;
                            }
                            let (PaginationItem::Page(before), PaginationItem::Page(after)) =
                                (at(index - 1), at(index + 1))
                            else {
                                panic!("an ellipsis beside an ellipsis at {where_}");
                            };
                            assert!(
                                after - before > 2,
                                "an ellipsis hides exactly one page at {where_}"
                            );
                        }

                        if total > window {
                            widths.insert(items.len());
                        }
                    }

                    // The strip never reflows as the user clicks through it.
                    assert!(
                        widths.len() <= 1,
                        "rendered width varies by page at total={total} \
                         siblings={siblings} boundaries={boundaries}: {widths:?}"
                    );
                }
            }
        }
    }
}
