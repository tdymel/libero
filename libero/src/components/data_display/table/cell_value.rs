use crate::{platform::Collator, str_enum::str_enum};

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

/// Which filter operators a column offers, from its cell type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum FilterKind {
    #[default]
    Text,
    Number,
    Boolean,
    Date,
    /// One of a fixed set of options, set with `Column::options`.
    Select,
}

/// What a cell sorts by. `Text` is pre-lowercased, so a comparison during the sort
/// allocates nothing. Off the web it is also accent-folded: a text with accents
/// follows its folded form after a NUL and the lowercased original, which orders it
/// after its plain twin. On the web the browser's collator orders it by the language.
#[derive(Clone, Debug, PartialEq)]
pub enum SortKey {
    Text(String),
    Num(f64),
    /// No value. Sorts last in both directions.
    Empty,
}

impl SortKey {
    pub fn text(value: impl AsRef<str>) -> Self {
        let lower = value.as_ref().to_lowercase();
        if lower.is_ascii() || cfg!(target_arch = "wasm32") {
            return Self::Text(lower);
        }
        let mut key = String::with_capacity(lower.len() * 2 + 1);
        for c in lower.chars() {
            match fold_accent(c) {
                Some(plain) => key.push_str(plain),
                None => key.push(c),
            }
        }
        if key != lower {
            key.push('\0');
            key.push_str(&lower);
        }
        Self::Text(key)
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

    /// `collator` orders texts by a language's rules, else their bytes.
    pub(super) fn compare(&self, other: &Self, collator: Option<&Collator>) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        match (self, other) {
            (Self::Text(a), Self::Text(b)) => match collator {
                Some(collator) => collator.compare(a, b),
                None => a.cmp(b),
            },
            (Self::Num(a), Self::Num(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            // A `CellValue` that mixes variants has no defined order.
            _ => Ordering::Equal,
        }
    }
}

/// The unaccented form of a lowercase Latin letter, so "émile" sorts among the e's.
/// Umlauts fold to their base letter, not to "ae".
fn fold_accent(c: char) -> Option<&'static str> {
    Some(match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' => "a",
        'ç' | 'ć' | 'č' => "c",
        'è' | 'é' | 'ê' | 'ë' | 'ē' => "e",
        'ì' | 'í' | 'î' | 'ï' | 'ī' => "i",
        'ñ' | 'ń' => "n",
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' => "o",
        'ù' | 'ú' | 'û' | 'ü' | 'ū' => "u",
        'ý' | 'ÿ' => "y",
        'ß' => "ss",
        _ => return None,
    })
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

    fn filter_kind() -> FilterKind {
        FilterKind::Text
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

    fn filter_kind() -> FilterKind {
        FilterKind::Boolean
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

                fn filter_kind() -> FilterKind {
                    FilterKind::Number
                }
            }
        )+
    };
}

numeric_cell_value!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64);

/// Printed as `2024-03-09`; give the column a `.format` for a local form.
impl CellValue for chrono::NaiveDate {
    fn cell_text(&self) -> String {
        self.format("%Y-%m-%d").to_string()
    }

    fn sort_key(&self) -> SortKey {
        SortKey::num(date_number(*self))
    }

    fn filter_kind() -> FilterKind {
        FilterKind::Date
    }
}

/// A day as the number it sorts and filters by.
pub(super) fn date_number(date: chrono::NaiveDate) -> f64 {
    use chrono::Datelike;
    f64::from(date.num_days_from_ce())
}

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

    fn filter_kind() -> FilterKind {
        V::filter_kind()
    }
}

/// Which editor an editable column's cells open, from its cell type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum EditKind {
    #[default]
    Text,
    Number,
    /// A checkbox.
    Bool,
    /// A checkbox with a third, empty state: an `Option<bool>`.
    Tristate,
    Date,
}

/// A cell type an [`editable`](super::Column::editable) column takes back from its
/// editor's text, which starts as the value's `cell_text`.
///
/// ```rust
/// use libero::components::{CellValue, EditValue};
///
/// struct Percent(u8);
///
/// impl CellValue for Percent {
///     fn cell_text(&self) -> String {
///         self.0.to_string()
///     }
/// }
///
/// impl EditValue for Percent {
///     fn parse(text: &str) -> Option<Self> {
///         text.trim().parse().ok().filter(|n| *n <= 100).map(Percent)
///     }
/// }
/// ```
pub trait EditValue: CellValue + Sized {
    /// The value of `text`, `None` when it is not one.
    fn parse(text: &str) -> Option<Self>;

    fn edit_kind() -> EditKind {
        EditKind::Text
    }
}

impl EditValue for String {
    fn parse(text: &str) -> Option<Self> {
        Some(text.to_string())
    }
}

impl EditValue for bool {
    fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        if text.eq_ignore_ascii_case("true") {
            Some(true)
        } else if text.eq_ignore_ascii_case("false") {
            Some(false)
        } else {
            None
        }
    }

    fn edit_kind() -> EditKind {
        EditKind::Bool
    }
}

macro_rules! numeric_edit_value {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl EditValue for $ty {
                fn parse(text: &str) -> Option<Self> {
                    text.trim().parse().ok()
                }

                fn edit_kind() -> EditKind {
                    EditKind::Number
                }
            }
        )+
    };
}

numeric_edit_value!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

/// Not `NaN`, `inf` or `infinity`, which `str::parse` takes but no ordering or sum can.
macro_rules! float_edit_value {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl EditValue for $ty {
                fn parse(text: &str) -> Option<Self> {
                    text.trim().parse().ok().filter(|n: &Self| n.is_finite())
                }

                fn edit_kind() -> EditKind {
                    EditKind::Number
                }
            }
        )+
    };
}

float_edit_value!(f32, f64);

/// Parsed as `2024-03-09`, the form `cell_text` prints.
impl EditValue for chrono::NaiveDate {
    fn parse(text: &str) -> Option<Self> {
        chrono::NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d").ok()
    }

    fn edit_kind() -> EditKind {
        EditKind::Date
    }
}

/// An empty text is `None`.
impl<V: EditValue> EditValue for Option<V> {
    fn parse(text: &str) -> Option<Self> {
        match text.trim() {
            "" => Some(None),
            _ => V::parse(text).map(Some),
        }
    }

    fn edit_kind() -> EditKind {
        match V::edit_kind() {
            EditKind::Bool => EditKind::Tristate,
            kind => kind,
        }
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
        assert!(9u32.sort_key().compare(&10u32.sort_key(), None).is_lt());
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
    fn accented_text_sorts_among_its_base_letters() {
        fn order<'a>(names: &[&'a str]) -> Vec<&'a str> {
            let mut names = names.to_vec();
            names.sort_by(|a, b| SortKey::text(a).compare(&SortKey::text(b), None));
            names
        }
        assert_eq!(
            order(&[
                "Zoe", "Émile", "Ärger", "Eli", "Ben", "emil", "Straße", "Strasse"
            ]),
            [
                "Ärger", "Ben", "Eli", "emil", "Émile", "Strasse", "Straße", "Zoe"
            ]
        );
    }

    #[test]
    fn accents_are_a_tiebreak_after_the_folded_text() {
        let (plain, accent) = (SortKey::text("resume"), SortKey::text("résumé"));
        assert!(plain.compare(&accent, None).is_lt());
        assert!(accent.compare(&SortKey::text("resumes"), None).is_lt());
        assert_eq!(SortKey::text("RÉSUMÉ"), accent);
        assert_eq!(SortKey::text("Alpha"), SortKey::Text("alpha".into()));
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
    fn the_filter_kind_follows_the_cell_type() {
        assert_eq!(<String as CellValue>::filter_kind(), FilterKind::Text);
        assert_eq!(<f32 as CellValue>::filter_kind(), FilterKind::Number);
        assert_eq!(
            <Option<bool> as CellValue>::filter_kind(),
            FilterKind::Boolean
        );
    }

    #[test]
    fn dates_print_iso_and_sort_by_day() {
        let day = |d| chrono::NaiveDate::from_ymd_opt(2024, 3, d).unwrap();
        assert_eq!(day(9).cell_text(), "2024-03-09");
        assert!(day(9).sort_key().compare(&day(10).sort_key(), None).is_lt());
        assert_eq!(
            <chrono::NaiveDate as CellValue>::filter_kind(),
            FilterKind::Date
        );
    }

    #[test]
    fn a_float_edit_refuses_what_has_no_place_in_an_order() {
        for text in ["NaN", "nan", "inf", "-inf", "infinity", "-Infinity"] {
            assert_eq!(<f64 as EditValue>::parse(text), None, "{text}");
            assert_eq!(<f32 as EditValue>::parse(text), None, "{text}");
            assert_eq!(<Option<f64> as EditValue>::parse(text), None, "{text}");
        }
        assert_eq!(<f64 as EditValue>::parse(" -1.5 "), Some(-1.5));
        assert_eq!(<f32 as EditValue>::parse("1e3"), Some(1000.0));
    }

    #[test]
    fn a_bool_edit_ignores_case() {
        assert_eq!(<bool as EditValue>::parse(" True "), Some(true));
        assert_eq!(<bool as EditValue>::parse("FALSE"), Some(false));
        assert_eq!(<bool as EditValue>::parse("yes"), None);
    }

    #[test]
    fn bools_render_as_plain_text() {
        assert_eq!(true.cell_text(), "true");
        assert_eq!(false.cell_text(), "false");
    }
}
