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

/// One column of a [`Table`](super::Table). The cell type is erased, so
/// columns over different types share one `Vec`.
pub struct Column<T> {
    pub(super) header: String,
    /// What the cell type chose; `align` and the table's defaults override it.
    pub(super) value_align: CellAlign,
    pub(super) align: Option<CellAlign>,
    pub(super) sortable: bool,
    pub(super) row_header: bool,
    pub(super) hideable: bool,
    pub(super) width: Option<String>,
    pub(super) min_width: Option<String>,
    pub(super) header_render: Option<HeaderRender>,
    pub(super) sort_key: Rc<dyn Fn(&T) -> SortKey>,
    /// The cell as plain text, drawn inline in its `td`.
    pub(super) text: Rc<dyn Fn(&T) -> String>,
    /// A caller's cell body, which replaces `text`.
    pub(super) render: Option<CellRender<T>>,
}

type CellRender<T> = Rc<dyn Fn(&T) -> Element>;
pub(super) type HeaderRender = Rc<dyn Fn() -> Element>;

impl ColumnHeader {
    /// Reads one cell out of a row. `V` decides how the column sorts and
    /// aligns; both are overridable afterwards.
    pub fn value<T, V: CellValue>(self, value: impl Fn(&T) -> V + 'static) -> Column<T> {
        let value = Rc::new(value);
        let sort_value = value.clone();

        Column {
            header: self.header,
            value_align: V::align(),
            align: None,
            sortable: false,
            row_header: false,
            hideable: true,
            width: None,
            min_width: None,
            header_render: None,
            sort_key: Rc::new(move |row| sort_value(row).sort_key()),
            text: Rc::new(move |row| value(row).cell_text()),
            render: None,
        }
    }

    /// Starts from a [`ColumnType`]; its `V` must match the
    /// [`value`](TypedColumnHeader::value) that follows.
    ///
    /// ```rust
    /// # use libero::components::{CellAlign, ColumnType, column};
    /// # struct Item { cents: u64 }
    /// const MONEY: ColumnType<u64> = ColumnType::new()
    ///     .align(CellAlign::End)
    ///     .width("8rem")
    ///     .format(|cents| format!("${}.{:02}", cents / 100, cents % 100));
    ///
    /// column("Price").of(&MONEY).value(|i: &Item| i.cents);
    /// ```
    pub fn of<V: CellValue>(self, column_type: &ColumnType<V>) -> TypedColumnHeader<V> {
        TypedColumnHeader {
            header: self.header,
            column_type: *column_type,
        }
    }
}

/// Settings shared by columns of one cell type `V`, as a `const`: alignment,
/// widths and the cell text. Applied with [`of`](ColumnHeader::of); the
/// column's own settings win.
pub struct ColumnType<V> {
    align: Option<CellAlign>,
    width: Option<&'static str>,
    min_width: Option<&'static str>,
    format: Option<fn(&V) -> String>,
}

// Hand-written: a derive would demand `V: Clone`.
impl<V> Clone for ColumnType<V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<V> Copy for ColumnType<V> {}

impl<V> Default for ColumnType<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> ColumnType<V> {
    pub const fn new() -> Self {
        Self {
            align: None,
            width: None,
            min_width: None,
            format: None,
        }
    }

    /// Overrides the alignment `V` chose.
    pub const fn align(self, align: CellAlign) -> Self {
        Self {
            align: Some(align),
            ..self
        }
    }

    /// The column's width, any CSS length.
    pub const fn width(self, width: &'static str) -> Self {
        Self {
            width: Some(width),
            ..self
        }
    }

    /// The narrowest the column gets, any CSS length.
    pub const fn min_width(self, min_width: &'static str) -> Self {
        Self {
            min_width: Some(min_width),
            ..self
        }
    }

    /// The cell text from the value. Sorting still uses the value.
    pub const fn format(self, format: fn(&V) -> String) -> Self {
        Self {
            format: Some(format),
            ..self
        }
    }
}

/// A column with a [`ColumnType`], waiting for its [`value`](TypedColumnHeader::value).
pub struct TypedColumnHeader<V> {
    header: String,
    column_type: ColumnType<V>,
}

impl<V: CellValue> TypedColumnHeader<V> {
    /// Reads one cell out of a row, as [`ColumnHeader::value`], then applies the type.
    pub fn value<T>(self, value: impl Fn(&T) -> V + 'static) -> Column<T> {
        let ColumnType {
            align,
            width,
            min_width,
            format,
        } = self.column_type;
        let value = Rc::new(value);
        let read = value.clone();
        let mut column = ColumnHeader {
            header: self.header,
        }
        .value(move |row: &T| read(row));
        column.align = align;
        column.width = width.map(String::from);
        column.min_width = min_width.map(String::from);
        if let Some(format) = format {
            column.text = Rc::new(move |row| format(&value(row)));
        }
        column
    }
}

impl<T> Column<T> {
    /// Makes the header a sort button.
    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    /// Renders this column's cells as `th scope="row"`, naming each row. One per table.
    ///
    /// ```rust
    /// # use libero::components::column;
    /// # #[derive(Clone, PartialEq)] struct User { name: String }
    /// column("Name").value(|u: &User| u.name.clone()).row_header();
    /// ```
    pub fn row_header(mut self) -> Self {
        self.row_header = true;
        self
    }

    /// Whether the column menu offers to hide it; on by default. `hidden_columns`
    /// still hides it.
    ///
    /// ```rust
    /// # use libero::components::column;
    /// # struct User { name: String }
    /// column("Name").value(|u: &User| u.name.clone()).hideable(false);
    /// ```
    pub fn hideable(mut self, hideable: bool) -> Self {
        self.hideable = hideable;
        self
    }

    /// Replaces the cell body. Sorting still uses `value`.
    ///
    /// Runs per row in `Table`'s scope: no hooks, and capture signals, not values.
    pub fn render(mut self, render: impl Fn(&T) -> Element + 'static) -> Self {
        self.render = Some(Rc::new(render));
        self
    }

    /// Replaces the cell text. Sorting still uses `value`, so a formatted
    /// number or date keeps its order.
    ///
    /// ```rust
    /// # use libero::components::column;
    /// # struct Item { cents: u64 }
    /// column("Price")
    ///     .value(|i: &Item| i.cents)
    ///     .format(|i: &Item| format!("${}.{:02}", i.cents / 100, i.cents % 100));
    /// ```
    pub fn format(mut self, format: impl Fn(&T) -> String + 'static) -> Self {
        self.text = Rc::new(format);
        self
    }

    /// Overrides the alignment `V` chose and the table's `column_defaults`.
    pub fn align(mut self, align: impl Into<CellAlign>) -> Self {
        self.align = Some(align.into());
        self
    }

    /// The column's width, any CSS length. Set on its header cell, which sizes
    /// the whole column; columns without one share the rest.
    ///
    /// ```rust
    /// # use libero::components::column;
    /// # struct Item { id: u32 }
    /// column("Id").value(|i: &Item| i.id).width("6rem");
    /// ```
    pub fn width(mut self, width: impl Into<String>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// The narrowest the column gets, any CSS length.
    pub fn min_width(mut self, min_width: impl Into<String>) -> Self {
        self.min_width = Some(min_width.into());
        self
    }

    /// Replaces the header's body, inside the sort button when sortable. The
    /// header text stays the column's name in [`TableSort`](super::TableSort).
    ///
    /// Compared as equal to any other: capture signals, not values, or the
    /// header stays stale.
    ///
    /// ```rust
    /// # use dioxus::prelude::*;
    /// # use libero::components::column;
    /// # struct Item { cents: u64 }
    /// column("Price")
    ///     .value(|i: &Item| i.cents)
    ///     .header_render(|| rsx! { "Price " small { "(USD)" } });
    /// ```
    pub fn header_render(mut self, render: impl Fn() -> Element + 'static) -> Self {
        self.header_render = Some(Rc::new(render));
        self
    }
}

// Hand-written: a derive would demand `T: Clone`, which the `Rc` fields don't.
impl<T> Clone for Column<T> {
    fn clone(&self) -> Self {
        Self {
            header: self.header.clone(),
            value_align: self.value_align,
            align: self.align,
            sortable: self.sortable,
            row_header: self.row_header,
            hideable: self.hideable,
            width: self.width.clone(),
            min_width: self.min_width.clone(),
            header_render: self.header_render.clone(),
            sort_key: self.sort_key.clone(),
            text: self.text.clone(),
            render: self.render.clone(),
        }
    }
}

// Closures compare equal, so re-declared columns do not defeat memoization.
impl<T> PartialEq for Column<T> {
    fn eq(&self, other: &Self) -> bool {
        self.header == other.header
            && self.value_align == other.value_align
            && self.align == other.align
            && self.sortable == other.sortable
            && self.row_header == other.row_header
            && self.hideable == other.hideable
            && self.width == other.width
            && self.min_width == other.min_width
    }
}

/// Settings every column of a [`Table`](super::Table) starts from; a column's
/// own setting wins.
///
/// ```rust
/// # use libero::components::ColumnDefaults;
/// let defaults = ColumnDefaults::new().min_width("8rem");
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ColumnDefaults {
    pub(super) align: Option<CellAlign>,
    pub(super) width: Option<String>,
    pub(super) min_width: Option<String>,
}

impl ColumnDefaults {
    pub fn new() -> Self {
        Self::default()
    }

    /// Overrides the alignment each cell type chose.
    pub fn align(mut self, align: impl Into<CellAlign>) -> Self {
        self.align = Some(align.into());
        self
    }

    /// Every column's width, any CSS length.
    pub fn width(mut self, width: impl Into<String>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// Every column's narrowest width, any CSS length.
    pub fn min_width(mut self, min_width: impl Into<String>) -> Self {
        self.min_width = Some(min_width.into());
        self
    }
}

/// A column's settings with the table's defaults filled in.
pub(super) struct Resolved<'a> {
    pub align: CellAlign,
    pub width: Option<&'a str>,
    pub min_width: Option<&'a str>,
}

impl<T> Column<T> {
    pub(super) fn resolve<'a>(&'a self, defaults: &'a ColumnDefaults) -> Resolved<'a> {
        Resolved {
            align: self.align.or(defaults.align).unwrap_or(self.value_align),
            width: self.width.as_deref().or(defaults.width.as_deref()),
            min_width: self.min_width.as_deref().or(defaults.min_width.as_deref()),
        }
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
        assert!(a != b.clone().row_header());
        assert!(a != b.clone().hideable(false));
        assert!(a != b.clone().width("4rem"));
        assert!(a != b.clone().min_width("4rem"));
        assert!(a == b.clone().header_render(|| rsx! { "x" }));
        assert!(a != column("Other").value(|row: &u32| *row));
    }

    const MONEY: ColumnType<u32> = ColumnType::new()
        .align(CellAlign::Center)
        .width("8rem")
        .format(|cents| format!("${cents}"));

    #[test]
    fn a_column_type_sets_the_text_but_the_value_still_sorts() {
        let column = column("Price").of(&MONEY).value(|row: &u32| *row);

        assert_eq!((column.text)(&7), "$7");
        assert_eq!((column.sort_key)(&7), SortKey::Num(7.0));
        let defaults = ColumnDefaults::new().width("2rem").align(CellAlign::Start);
        let resolved = column.resolve(&defaults);
        assert_eq!(resolved.align, CellAlign::Center);
        assert_eq!(resolved.width, Some("8rem"));
    }

    #[test]
    fn a_columns_own_setting_wins_over_its_type() {
        let column = column("Price")
            .of(&MONEY)
            .value(|row: &u32| *row)
            .width("3rem")
            .format(|row: &u32| format!("{row} ct"));

        assert_eq!((column.text)(&7), "7 ct");
        assert_eq!(column.resolve(&ColumnDefaults::new()).width, Some("3rem"));
    }

    #[test]
    fn a_columns_own_setting_wins_over_the_defaults() {
        let defaults = ColumnDefaults::new()
            .align(CellAlign::Center)
            .width("8rem")
            .min_width("2rem");
        let plain = column("N").value(|row: &u32| *row);
        let resolved = plain.resolve(&defaults);
        assert_eq!(resolved.align, CellAlign::Center);
        assert_eq!(resolved.width, Some("8rem"));
        assert_eq!(resolved.min_width, Some("2rem"));

        let own = plain.clone().align(CellAlign::Start).width("3rem");
        let resolved = own.resolve(&defaults);
        assert_eq!(resolved.align, CellAlign::Start);
        assert_eq!(resolved.width, Some("3rem"));
    }

    #[test]
    fn a_custom_render_leaves_the_sort_key_alone() {
        let column = column("N")
            .value(|row: &u32| *row)
            .render(|_: &u32| rsx! { "x" });

        assert_eq!((column.sort_key)(&7), SortKey::Num(7.0));
    }

    #[test]
    fn a_format_changes_the_text_but_not_the_sort_key() {
        let column = column("N")
            .value(|row: &u32| *row)
            .format(|row: &u32| format!("#{row}"));

        assert_eq!((column.text)(&9), "#9");
        assert_eq!((column.sort_key)(&9), SortKey::Num(9.0));
        assert_eq!(column.value_align, CellAlign::End);
    }

    #[test]
    fn the_cell_type_picks_the_alignment() {
        let align = |column: Column<u32>| column.resolve(&ColumnDefaults::new()).align;
        assert_eq!(align(column("N").value(|row: &u32| *row)), CellAlign::End);
        assert_eq!(
            align(column("S").value(|row: &u32| row.to_string())),
            CellAlign::Start
        );
        assert_eq!(
            align(column("N").value(|row: &u32| *row).align(CellAlign::Center)),
            CellAlign::Center
        );
    }
}
