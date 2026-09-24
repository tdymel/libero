use crate::str_enum::str_enum;

str_enum! {
    /// A column's horizontal alignment, from its cell type.
    pub enum CellAlign {
        #[default]
        Start = "start",
        Center = "center",
        End = "end",
    }
}

/// Which way a sorted column runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

impl SortDirection {
    pub(super) const fn aria_value(self) -> &'static str {
        match self {
            Self::Ascending => "ascending",
            Self::Descending => "descending",
        }
    }
}

/// What a cell sorts by. `Text` is pre-lowercased, so a comparison during the
/// sort allocates nothing.
#[derive(Clone, Debug, PartialEq)]
pub enum SortKey {
    Text(String),
    Num(f64),
    /// No value. Sorts last in both directions.
    Empty,
}

impl SortKey {
    pub fn text(value: impl AsRef<str>) -> Self {
        Self::Text(value.as_ref().to_lowercase())
    }

    /// `NaN` has no position in an ordering, so it becomes `Empty`. Integers
    /// past 2^53 lose precision here and may compare equal.
    pub fn num(value: f64) -> Self {
        if value.is_nan() {
            Self::Empty
        } else {
            Self::Num(value)
        }
    }

    pub(super) const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    pub(super) fn compare(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        match (self, other) {
            (Self::Text(a), Self::Text(b)) => a.cmp(b),
            (Self::Num(a), Self::Num(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            // A `CellValue` that mixes variants has no defined order.
            _ => Ordering::Equal,
        }
    }
}

/// What a column's cell type tells `Table`: how to print it, how to order it,
/// and which way to align it.
pub trait CellValue: 'static {
    fn cell_text(&self) -> String;

    fn sort_key(&self) -> SortKey {
        SortKey::text(self.cell_text())
    }

    fn align() -> CellAlign {
        CellAlign::Start
    }
}

impl CellValue for String {
    fn cell_text(&self) -> String {
        self.clone()
    }
}

impl CellValue for &'static str {
    fn cell_text(&self) -> String {
        (*self).to_string()
    }
}

impl CellValue for bool {
    fn cell_text(&self) -> String {
        self.to_string()
    }
}

macro_rules! numeric_cell_value {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl CellValue for $ty {
                fn cell_text(&self) -> String {
                    self.to_string()
                }

                fn sort_key(&self) -> SortKey {
                    SortKey::num(*self as f64)
                }

                fn align() -> CellAlign {
                    CellAlign::End
                }
            }
        )+
    };
}

numeric_cell_value!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64);

/// `None` renders empty and sorts last, but keeps the inner type's alignment.
impl<V: CellValue> CellValue for Option<V> {
    fn cell_text(&self) -> String {
        self.as_ref().map(V::cell_text).unwrap_or_default()
    }

    fn sort_key(&self) -> SortKey {
        self.as_ref().map_or(SortKey::Empty, V::sort_key)
    }

    fn align() -> CellAlign {
        V::align()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_sort_numerically_and_align_end() {
        assert_eq!(10u32.sort_key(), SortKey::Num(10.0));
        assert_eq!(<u32 as CellValue>::align(), CellAlign::End);
        // The trap this exists to prevent: "9" after "10" as text.
        assert!(9u32.sort_key().compare(&10u32.sort_key()).is_lt());
    }

    #[test]
    fn text_keys_are_lowercased_once() {
        assert_eq!(
            "Alpha".to_string().sort_key(),
            SortKey::Text("alpha".into())
        );
        assert_eq!(<String as CellValue>::align(), CellAlign::Start);
    }

    #[test]
    fn nan_has_no_position_so_it_sorts_as_empty() {
        assert_eq!(f64::NAN.sort_key(), SortKey::Empty);
    }

    #[test]
    fn option_keeps_the_inner_alignment_but_sorts_none_last() {
        assert_eq!(<Option<u32> as CellValue>::align(), CellAlign::End);
        assert_eq!(None::<u32>.sort_key(), SortKey::Empty);
        assert_eq!(None::<u32>.cell_text(), "");
        assert_eq!(Some(7u32).sort_key(), SortKey::Num(7.0));
    }

    #[test]
    fn bools_render_as_plain_text() {
        assert_eq!(true.cell_text(), "true");
        assert_eq!(false.cell_text(), "false");
    }
}
