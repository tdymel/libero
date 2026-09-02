//! The ellipsis range: a pure function of four integers, and the only hard part
//! of `Pagination`.
//!
//! Shipped as a free function with a pinned table rather than folded into the
//! component, the way `rows_spanned` is in `grid/grid_item.rs` - it is the piece
//! worth testing on its own, and a caller drawing a custom strip can reuse the
//! arithmetic.

/// One slot in the rendered strip.
///
/// Only these two ever come out of [`pagination_range`]. The prev/next/first/
/// last controls are the component's, not the range's, which is why they are
/// not variants here - see `PaginationLabel` for what a caller naming controls
/// receives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaginationItem {
    Page(u32),
    Ellipsis,
}

/// Which page numbers to draw, and where the gaps go.
///
/// Ours is MUI's range restated symmetrically; it differs from Mantine's only
/// where Mantine's right-hand test is off by one against its left-hand one, and
/// that difference is the invariant below.
///
/// **An ellipsis never hides exactly one page.** Hiding `9` behind `…` costs
/// the same width as printing it, so the gap is only drawn where it saves
/// something. That also fixes the rendered length at `2·siblings +
/// 2·boundaries + 3` for every `page`, so the strip never reflows while the
/// user clicks through it.
///
/// Inputs are taken loosely and clamped: `total` of 0 is treated as 1 by the
/// arithmetic (the component renders nothing for 0 before calling this), `page`
/// is clamped into range, and `boundaries` of 0 becomes 1 - both references
/// emit leading dots with nothing outside them at 0, which is a worse strip
/// than simply pinning the first page.
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

    // Past here `total > window`, which is what makes the arithmetic below
    // safe, and it is worth spelling out because both facts read as merely
    // probable:
    //
    // - **Neither `clamp` can panic.** `u32::clamp` panics when its bounds
    //   invert, and both widths here are exactly `total - window`:
    //   `(total - boundaries - 2·siblings - 1) - (boundaries + 2)` and
    //   `(total - boundaries - 1) - (boundaries + 2·siblings + 2)` both reduce
    //   to it. In this branch that is at least 1, so `low < high` always.
    // - **No subtraction underflows.** `total >= window + 1`, so
    //   `total - boundaries - 2·siblings - 1 >= boundaries + 3` and
    //   `total - boundaries - 1 >= boundaries + 2·siblings + 3`, both above
    //   zero.
    //
    // - **The rendered width is the same for every `page`**, which is what keeps
    //   the strip from reflowing as you click through it. **This rests on the
    //   same branch condition as the clamp fact above and is not independent of
    //   it**: given neither clamp inverts, the two have identical widths and
    //   bounds differing by exactly `2 * siblings`, so `right - left` is
    //   `2 * siblings` whatever `page` is and the total is always
    //   `2 * boundaries + 2 * siblings + 3` - `window`, which is what
    //   `with_capacity` below is given. Move one of these without the other and
    //   the survivor loses its footing. (Karen3, reviewing C4.)
    //
    // The test sweep and this derivation are not the same evidence, and the
    // difference is worth keeping. The sweep establishes that the width was
    // constant for `total` 1..39; the derivation is why it is constant at
    // `window` for every `total`. A reader who sees only the sweep will take
    // the bound for empirical and re-run it after any change.
    //
    // Proved rather than sampled: a u32 underflow panics in debug and wraps in
    // release, and neither shows up in a test that only walks the pinned table.
    let left = (page.saturating_sub(siblings))
        .clamp(boundaries + 2, total - boundaries - 2 * siblings - 1);
    let right = (page + siblings).clamp(boundaries + 2 * siblings + 2, total - boundaries - 1);

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

    /// The plan's pinned table, every row. Checked against MUI's
    /// `usePagination` output; the `total: 10, page: 7` row is the one where
    /// Mantine differs, and ours is the one that does not hide a single page
    /// behind a gap the same width.
    #[test]
    fn the_pinned_table() {
        let rows: &[(u32, u8, u8, u32, &str)] = &[
            (7, 1, 1, 4, "1 2 3 4 5 6 7"),
            (10, 1, 1, 1, "1 2 3 4 5 … 10"),
            (10, 1, 1, 4, "1 2 3 4 5 … 10"),
            (10, 1, 1, 5, "1 … 4 5 6 … 10"),
            (10, 1, 1, 6, "1 … 5 6 7 … 10"),
            // Mantine renders `1 … 6 7 8 … 10` here, hiding only `9`.
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

    /// `boundaries: 0` is clamped to 1. Both references emit leading dots with
    /// nothing outside them at 0, which hides a page to show a gap.
    #[test]
    fn zero_boundaries_is_clamped_to_one() {
        assert_eq!(strip(20, 10, 1, 0), strip(20, 10, 1, 1));
        assert!(strip(20, 10, 1, 0).starts_with("1 …"));
    }

    /// The three properties the whole design rests on, over the range the plan
    /// claims them for. The pinned table is a sample; these are the reason it
    /// looks the way it does.
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

                        // Pages strictly increase, so nothing is repeated or
                        // out of order.
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

                        // An ellipsis always stands for two pages or more.
                        // `index - 1` and `index + 1` cannot go out of bounds:
                        // `boundaries >= 1` puts a page before every ellipsis,
                        // and the trailing boundary pages are pushed after the
                        // second one.
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
