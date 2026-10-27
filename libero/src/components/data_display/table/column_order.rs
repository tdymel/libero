use super::core::{HeaderSpec, namesake_key};

/// `column_order`'s headers as column keys: a header's `n`th listing keys its `n`th namesake.
pub(super) fn order_keys(column_order: &[String]) -> Vec<String> {
    column_order
        .iter()
        .enumerate()
        .map(|(at, header)| {
            let earlier = column_order[..at].iter().filter(|h| *h == header).count();
            namesake_key(header, earlier)
        })
        .collect()
}

/// Column indices in `column_order`: listed headers first in its order, the
/// rest after them in column order. A repeated header ranks its namesakes in turn.
pub(super) fn ranked(headers: &[HeaderSpec], column_order: &[String]) -> Vec<usize> {
    let mut ranked: Vec<usize> = Vec::with_capacity(headers.len());
    for key in order_keys(column_order) {
        if let Some(index) = headers.iter().position(|spec| spec.key == key)
            && !ranked.contains(&index)
        {
            ranked.push(index);
        }
    }
    let rest: Vec<usize> = (0..headers.len())
        .filter(|index| !ranked.contains(index))
        .collect();
    ranked.extend(rest);
    ranked
}

/// Sorts the unpinned run of `order`, the shown columns in display order, by
/// `column_order`. Pinned columns keep their pinned order.
pub(super) fn order_unpinned(headers: &[HeaderSpec], order: &mut [usize], column_order: &[String]) {
    if column_order.is_empty() {
        return;
    }
    let ranked = ranked(headers, column_order);
    let rank = |index: &usize| ranked.iter().position(|r| r == index);
    let start = order
        .iter()
        .position(|&index| headers[index].pin.is_none())
        .unwrap_or(order.len());
    let end = order[start..]
        .iter()
        .position(|&index| headers[index].pin.is_some())
        .map_or(order.len(), |len| start + len);
    order[start..end].sort_by_key(rank);
}

/// The column order after moving `index` one shown, unpinned column towards the
/// end (`forward`) or the start; `None` at the edge or when namesakes would pass
/// one another, which a header list cannot tell. Lists every header, so the order survives showing a hidden column.
pub(super) fn moved(
    headers: &[HeaderSpec],
    column_order: &[String],
    index: usize,
    forward: bool,
) -> Option<Vec<String>> {
    let movable = |at: usize| !headers[at].hidden && headers[at].pin.is_none();
    if !movable(index) {
        return None;
    }
    let mut ranked = ranked(headers, column_order);
    let at = ranked.iter().position(|&r| r == index)?;
    let other = match forward {
        true => (at + 1..ranked.len()).find(|&o| movable(ranked[o]))?,
        false => (0..at).rev().find(|&o| movable(ranked[o]))?,
    };
    ranked.swap(at, other);
    let keys: Vec<String> = ranked
        .iter()
        .map(|&index| headers[index].key.clone())
        .collect();
    as_headers(headers.iter().map(|spec| (&spec.key, &spec.header)), &keys)
}

/// `keys` as the header list `column_order` holds, or `None` when that list would
/// read back as other keys: namesakes passing one another.
pub(super) fn as_headers<'a>(
    columns: impl Iterator<Item = (&'a String, &'a String)> + Clone,
    keys: &[String],
) -> Option<Vec<String>> {
    let headers: Vec<String> = keys
        .iter()
        .map(|key| {
            columns
                .clone()
                .find(|(k, _)| *k == key)
                .map(|(_, h)| h.clone())
        })
        .collect::<Option<_>>()?;
    (order_keys(&headers) == keys).then_some(headers)
}

/// The column order, every header by rank, after dropping `header` into gap
/// `gap` of `unpinned`, the shown unpinned headers in display order (gap `n`
/// before its `n`th, the last after it). `None` when nothing moves.
pub(super) fn dropped(
    ranked: &[String],
    unpinned: &[String],
    header: &str,
    gap: usize,
) -> Option<Vec<String>> {
    let mut next: Vec<String> = ranked.iter().filter(|h| *h != header).cloned().collect();
    let at = match unpinned.get(gap) {
        Some(before) => next.iter().position(|h| h == before)?,
        None => next.iter().position(|h| Some(h) == unpinned.last())? + 1,
    };
    next.insert(at, header.to_string());
    (next != ranked).then_some(next)
}

#[cfg(test)]
pub(super) mod tests {
    use super::{
        super::{
            cell_value::CellAlign,
            pinning::{PinnedColumns, pin_columns},
        },
        *,
    };

    /// Specs keyed as `header_specs` keys them, namesakes apart.
    pub(in super::super) fn headers(names: &[&str]) -> Vec<HeaderSpec> {
        names
            .iter()
            .zip(order_keys(&strings(names)))
            .map(|(header, key)| HeaderSpec {
                header: header.to_string(),
                key,
                align: CellAlign::Start,
                sortable: false,
                row_header: false,
                hideable: true,
                hidden: false,
                width: Some("1px".into()),
                min_width: None,
                body: None,
                groups: Vec::new(),
                pin: None,
                resize: None,
            })
            .collect()
    }

    fn strings(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    #[test]
    fn listed_headers_lead_and_the_rest_keep_column_order() {
        let specs = headers(&["A", "B", "C", "D"]);

        assert_eq!(
            ranked(&specs, &strings(&["C", "X", "A", "C"])),
            vec![2, 0, 1, 3]
        );
        assert_eq!(ranked(&specs, &[]), vec![0, 1, 2, 3]);
    }

    #[test]
    fn only_the_unpinned_run_follows_the_order() {
        let mut specs = headers(&["A", "B", "C", "D", "E"]);
        let pinned = PinnedColumns::default().start(["E"]).end(["A"]);
        let (mut order, _) = pin_columns(&mut specs, &pinned, None, &Default::default());

        order_unpinned(&specs, &mut order, &strings(&["D", "A", "C", "E", "B"]));

        assert_eq!(order, vec![4, 3, 2, 1, 0]);
    }

    #[test]
    fn a_move_steps_over_hidden_and_pinned_columns() {
        let mut specs = headers(&["A", "B", "C", "D"]);
        specs[1].hidden = true;
        pin_columns(
            &mut specs,
            &PinnedColumns::default().start(["D"]),
            None,
            &Default::default(),
        );

        assert_eq!(
            moved(&specs, &[], 0, true),
            Some(strings(&["C", "B", "A", "D"]))
        );
        assert_eq!(moved(&specs, &[], 0, false), None);
        assert_eq!(moved(&specs, &[], 2, true), None);
        assert_eq!(moved(&specs, &[], 3, false), None);
        assert_eq!(moved(&specs, &[], 1, true), None);
        assert_eq!(
            moved(&specs, &strings(&["C", "A"]), 2, true),
            Some(strings(&["A", "C", "B", "D"]))
        );
    }

    #[test]
    fn a_drop_inserts_at_its_gap_and_its_own_gaps_move_nothing() {
        // B is hidden, so the gaps lie around A, C and D only.
        let ranked = strings(&["A", "B", "C", "D"]);
        let unpinned = strings(&["A", "C", "D"]);

        assert_eq!(
            dropped(&ranked, &unpinned, "A", 2),
            Some(strings(&["B", "C", "A", "D"]))
        );
        assert_eq!(
            dropped(&ranked, &unpinned, "A", 3),
            Some(strings(&["B", "C", "D", "A"]))
        );
        assert_eq!(
            dropped(&ranked, &unpinned, "D", 0),
            Some(strings(&["D", "A", "B", "C"]))
        );
        assert_eq!(dropped(&ranked, &unpinned, "C", 1), None);
        assert_eq!(dropped(&ranked, &unpinned, "C", 2), None);
    }

    /// 4428: a repeated header ranks its namesakes in turn, and a move never swaps them.
    #[test]
    fn namesakes_rank_in_turn_and_never_pass_one_another() {
        let mut specs = headers(&["Name", "Age", "Name", "Id"]);

        assert_eq!(
            ranked(&specs, &strings(&["Name", "Name", "Age"])),
            vec![0, 2, 1, 3]
        );
        assert_eq!(
            moved(&specs, &[], 1, true),
            Some(strings(&["Name", "Name", "Age", "Id"]))
        );
        specs[1].hidden = true;
        assert_eq!(moved(&specs, &[], 0, true), None);
        assert_eq!(moved(&specs, &[], 2, false), None);
        assert_eq!(
            moved(&specs, &[], 2, true),
            Some(strings(&["Name", "Age", "Id", "Name"]))
        );
    }
}
