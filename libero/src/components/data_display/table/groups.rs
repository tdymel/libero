/// One cell of a header row: a group over adjacent columns, or a column's own
/// header, which spans down to the last header row.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum HeaderCell {
    Group { name: String, span: usize },
    Column { index: usize, rowspan: usize },
}

/// The header rows, top first, over the shown columns' group paths (outermost
/// first), by header index. Without groups, one row of the columns. A group
/// splits where `split(position)` holds, as at a pin edge.
pub(super) fn header_rows(
    shown: &[(usize, &[String])],
    split: impl Fn(usize) -> bool,
) -> Vec<Vec<HeaderCell>> {
    let depth = shown.iter().map(|(_, path)| path.len()).max().unwrap_or(0);
    let mut rows: Vec<Vec<HeaderCell>> = vec![Vec::new(); depth + 1];
    for (position, &(index, path)) in shown.iter().enumerate() {
        for level in 0..path.len() {
            // Merged with the column before when the whole path down to here matches.
            let continues = position > 0 && !split(position) && {
                let before = shown[position - 1].1;
                before.len() > level && before[..=level] == path[..=level]
            };
            match (continues, rows[level].last_mut()) {
                (true, Some(HeaderCell::Group { span, .. })) => *span += 1,
                _ => rows[level].push(HeaderCell::Group {
                    name: path[level].clone(),
                    span: 1,
                }),
            }
        }
        rows[path.len()].push(HeaderCell::Column {
            index,
            rowspan: depth + 1 - path.len(),
        });
    }
    rows
}

/// A body row's cells as `(header index, colspan)`: a span covers the shown
/// columns after it, clamped to the row's end; 0 counts as 1.
pub(super) fn spanned(shown: &[usize], span: impl Fn(usize) -> usize) -> Vec<(usize, usize)> {
    let mut cells = Vec::with_capacity(shown.len());
    let mut position = 0;
    while position < shown.len() {
        let index = shown[position];
        let width = span(index).clamp(1, shown.len() - position);
        cells.push((index, width));
        position += width;
    }
    cells
}

#[cfg(test)]
mod tests {
    use super::*;
    use HeaderCell::{Column, Group};

    fn path(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    fn rows(paths: &[Vec<String>]) -> Vec<Vec<HeaderCell>> {
        let shown: Vec<(usize, &[String])> = paths
            .iter()
            .enumerate()
            .map(|(index, path)| (index, path.as_slice()))
            .collect();
        header_rows(&shown, |_| false)
    }

    fn group(name: &str, span: usize) -> HeaderCell {
        Group {
            name: name.into(),
            span,
        }
    }

    #[test]
    fn without_groups_the_columns_are_one_row() {
        assert_eq!(
            rows(&[path(&[]), path(&[])]),
            vec![vec![
                Column {
                    index: 0,
                    rowspan: 1
                },
                Column {
                    index: 1,
                    rowspan: 1
                },
            ]]
        );
    }

    #[test]
    fn adjacent_columns_of_a_group_merge_and_ungrouped_ones_span_down() {
        let layout = rows(&[path(&["Name"]), path(&["Name"]), path(&[])]);

        assert_eq!(
            layout,
            vec![
                vec![
                    group("Name", 2),
                    Column {
                        index: 2,
                        rowspan: 2
                    },
                ],
                vec![
                    Column {
                        index: 0,
                        rowspan: 1
                    },
                    Column {
                        index: 1,
                        rowspan: 1
                    },
                ],
            ]
        );
    }

    #[test]
    fn nested_groups_merge_only_under_the_same_parent() {
        let layout = rows(&[
            path(&["A", "X"]),
            path(&["A", "X"]),
            path(&["A"]),
            path(&["B", "X"]),
        ]);

        assert_eq!(layout[0], vec![group("A", 3), group("B", 1)]);
        assert_eq!(
            layout[1],
            vec![
                group("X", 2),
                Column {
                    index: 2,
                    rowspan: 2
                },
                group("X", 1),
            ]
        );
        assert_eq!(layout[2].len(), 3);
    }

    #[test]
    fn a_group_split_by_another_column_is_two_cells() {
        let layout = rows(&[path(&["A"]), path(&[]), path(&["A"])]);

        assert_eq!(layout[0][0], group("A", 1));
        assert_eq!(layout[0][2], group("A", 1));
    }

    #[test]
    fn a_group_splits_where_told() {
        let paths = [path(&["A"]), path(&["A"])];
        let shown: Vec<(usize, &[String])> = paths
            .iter()
            .enumerate()
            .map(|(index, path)| (index, path.as_slice()))
            .collect();

        assert_eq!(
            header_rows(&shown, |position| position == 1)[0],
            vec![group("A", 1), group("A", 1)]
        );
    }

    #[test]
    fn spans_cover_the_following_columns_and_stop_at_the_row_end() {
        let shown = [0, 2, 3, 5];

        assert_eq!(spanned(&shown, |_| 1), vec![(0, 1), (2, 1), (3, 1), (5, 1)]);
        assert_eq!(
            spanned(&shown, |index| if index == 2 { 2 } else { 0 }),
            vec![(0, 1), (2, 2), (5, 1)]
        );
        assert_eq!(
            spanned(&shown, |index| if index == 3 { 9 } else { 1 }).last(),
            Some(&(3, 2))
        );
    }
}
