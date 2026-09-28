use super::core::HeaderSpec;

/// Column indices in `column_order`: listed headers first in its order, the
/// rest after them in column order. Duplicate headers resolve to the first.
pub(super) fn ranked(headers: &[HeaderSpec], column_order: &[String]) -> Vec<usize> {
    let mut ranked: Vec<usize> = Vec::with_capacity(headers.len());
    for header in column_order {
        if let Some(index) = headers.iter().position(|spec| spec.header == *header)
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
/// end (`forward`) or the start; `None` at the edge. Lists every header, so
/// the order survives showing a hidden column.
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
    Some(
        ranked
            .into_iter()
            .map(|index| headers[index].header.clone())
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        super::{
            cell_value::CellAlign,
            pinning::{PinnedColumns, pin_columns},
        },
        *,
    };

    fn headers(names: &[&str]) -> Vec<HeaderSpec> {
        names
            .iter()
            .map(|header| HeaderSpec {
                header: header.to_string(),
                align: CellAlign::Start,
                sortable: false,
                row_header: false,
                hideable: true,
                hidden: false,
                width: Some("1px".into()),
                style: None,
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
        let (mut order, _) = pin_columns(&mut specs, &pinned, None);

        order_unpinned(&specs, &mut order, &strings(&["D", "A", "C", "E", "B"]));

        assert_eq!(order, vec![4, 3, 2, 1, 0]);
    }

    #[test]
    fn a_move_steps_over_hidden_and_pinned_columns() {
        let mut specs = headers(&["A", "B", "C", "D"]);
        specs[1].hidden = true;
        pin_columns(&mut specs, &PinnedColumns::default().start(["D"]), None);

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
}
