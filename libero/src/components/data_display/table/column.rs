use std::rc::Rc;

use dioxus::prelude::*;

use super::cell_value::{CellAlign, CellValue, SortKey};

/// A column waiting for its [`value`](ColumnHeader::value).
pub struct ColumnHeader {
    header: String,
}

/// Starts a column. [`value`](ColumnHeader::value) is what makes it usable.
pub fn column(header: impl Into<String>) -> ColumnHeader {
    ColumnHeader {
        header: header.into(),
    }
}

/// One column of a [`Table`](super::Table). The cell type `V` is read for its
/// ordering and alignment, then erased - so columns over different cell types
/// live in one `Vec`.
pub struct Column<T> {
    pub(super) header: String,
    pub(super) align: CellAlign,
    pub(super) sortable: bool,
    pub(super) sort_key: Rc<dyn Fn(&T) -> SortKey>,
    pub(super) render: Rc<dyn Fn(&T) -> Element>,
}

impl ColumnHeader {
    /// Reads one cell out of a row. `V` decides how the column sorts and
    /// aligns; both are overridable afterwards.
    pub fn value<T, V: CellValue>(self, value: impl Fn(&T) -> V + 'static) -> Column<T> {
        let value = Rc::new(value);
        let sort_value = value.clone();

        Column {
            header: self.header,
            align: V::align(),
            sortable: false,
            sort_key: Rc::new(move |row| sort_value(row).sort_key()),
            render: Rc::new(move |row| {
                let text = value(row).cell_text();
                rsx! { "{text}" }
            }),
        }
    }
}

impl<T> Column<T> {
    /// Makes the header a sort button.
    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    /// Replaces the cell body. Sorting still uses `value`.
    ///
    /// Runs once per row inside `Table`'s own scope: no hooks, and capture
    /// signals rather than values - a captured value can't re-render the table.
    pub fn render(mut self, render: impl Fn(&T) -> Element + 'static) -> Self {
        self.render = Rc::new(render);
        self
    }

    /// Overrides the alignment `V` chose.
    pub fn align(mut self, align: impl Into<CellAlign>) -> Self {
        self.align = align.into();
        self
    }
}

// Hand-written: a derive would demand `T: Clone`, which the `Rc` fields don't.
impl<T> Clone for Column<T> {
    fn clone(&self) -> Self {
        Self {
            header: self.header.clone(),
            align: self.align,
            sortable: self.sortable,
            sort_key: self.sort_key.clone(),
            render: self.render.clone(),
        }
    }
}

// Closures compare equal, as `ErasedRenderNode` does for `Tree`: a re-declared
// `vec![..]` of the same columns must not defeat memoization.
impl<T> PartialEq for Column<T> {
    fn eq(&self, other: &Self) -> bool {
        self.header == other.header && self.align == other.align && self.sortable == other.sortable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn columns_compare_by_configuration_not_by_closure() {
        let a = column("Name").value(|row: &u32| *row);
        let b = column("Name").value(|row: &u32| row + 1);

        assert!(a == b);
        assert!(a != b.clone().sortable());
        assert!(a != column("Other").value(|row: &u32| *row));
    }

    #[test]
    fn a_custom_render_leaves_the_sort_key_alone() {
        let column = column("N")
            .value(|row: &u32| *row)
            .render(|_: &u32| rsx! { "x" });

        assert_eq!((column.sort_key)(&7), SortKey::Num(7.0));
    }

    #[test]
    fn the_cell_type_picks_the_alignment() {
        assert_eq!(column("N").value(|row: &u32| *row).align, CellAlign::End);
        assert_eq!(
            column("S").value(|row: &u32| row.to_string()).align,
            CellAlign::Start
        );
        assert_eq!(
            column("N")
                .value(|row: &u32| *row)
                .align(CellAlign::Center)
                .align,
            CellAlign::Center
        );
    }
}
