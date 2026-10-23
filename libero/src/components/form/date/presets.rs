//! Named values a day or range picker offers beside its calendar.

/// A value a picker offers as one button beside its calendar, as `Today` or `Last 7 days`.
/// Pressing it picks the value; in a field it commits and closes like a picked day.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::chrono::NaiveDate;
/// # use libero::components::{DateField, DatePreset};
/// # fn app() -> Element {
/// let mut date = use_signal(|| None::<NaiveDate>);
/// let new_year = NaiveDate::from_ymd_opt(2027, 1, 1).unwrap();
/// rsx! {
///     DateField {
///         label: "Due",
///         value: date(),
///         onchange: move |v| date.set(v),
///         presets: vec![DatePreset::new("New Year", new_year)],
///     }
/// }
/// # }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatePreset<V> {
    pub label: String,
    pub value: V,
}

impl<V> DatePreset<V> {
    pub fn new(label: impl Into<String>, value: V) -> Self {
        Self {
            label: label.into(),
            value,
        }
    }
}
