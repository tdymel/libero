//! A list of choices: flat or grouped, with some disabled, ready or still being fetched.

use dioxus::prelude::*;

/// One choice and its list flags: `[Berlin.into(), OptionItem::new(Bonn).disabled(true)]`.
#[derive(Clone, PartialEq, Debug)]
pub struct OptionItem<T> {
    pub(crate) value: T,
    pub(crate) disabled: bool,
}

impl<T> OptionItem<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            disabled: false,
        }
    }

    /// Drawn and announced as `aria-disabled`, but skipped by arrows, typeahead and clicks.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

impl<T> From<T> for OptionItem<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// The choices a list draws: one unnamed run, or several named groups.
#[derive(Clone, PartialEq, Debug)]
pub struct OptionList<T> {
    /// In order; a `None` label is an unnamed run.
    groups: Vec<(Option<String>, Vec<OptionItem<T>>)>,
}

impl<T> Default for OptionList<T> {
    fn default() -> Self {
        Self { groups: Vec::new() }
    }
}

impl<T> OptionList<T> {
    /// A flat list, in order.
    pub fn new(items: impl IntoIterator<Item = impl Into<OptionItem<T>>>) -> Self {
        let items: Vec<OptionItem<T>> = items.into_iter().map(Into::into).collect();
        match items.is_empty() {
            true => Self::default(),
            false => Self {
                groups: vec![(None, items)],
            },
        }
    }

    /// An empty list to hang [`group`](Self::group) calls off.
    pub fn grouped() -> Self {
        Self::default()
    }

    /// A named run of choices, after the ones already added. Order is kept: a repeated,
    /// non-adjacent label draws its heading twice rather than merging.
    pub fn group(
        mut self,
        label: impl Into<String>,
        items: impl IntoIterator<Item = impl Into<OptionItem<T>>>,
    ) -> Self {
        let items = items.into_iter().map(Into::into).collect();
        self.groups.push((Some(label.into()), items));
        self
    }

    /// Every choice in drawn order: the index space the components work in.
    pub(crate) fn items(&self) -> impl Iterator<Item = &OptionItem<T>> {
        self.groups.iter().flat_map(|(_, members)| members)
    }

    pub(crate) fn values(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.items().map(|item| item.value.clone()).collect()
    }

    /// One flag per choice, parallel to [`values`](Self::values).
    pub(crate) fn disabled(&self) -> Vec<bool> {
        self.items().map(|item| item.disabled).collect()
    }

    /// Disables every choice the predicate names. It only adds flags, never clears one.
    ///
    /// ```
    /// # use libero::components::{OptionList, Options};
    /// # #[derive(Clone, Copy, PartialEq, Options)] enum Plan { Free, Pro, Team }
    /// let plans: OptionList<Plan> =
    ///     OptionList::from_options().disabling(|plan| *plan == Plan::Team);
    /// ```
    pub fn disabling(mut self, disabled: impl Fn(&T) -> bool) -> Self {
        for (_, members) in &mut self.groups {
            for item in members {
                item.disabled = item.disabled || disabled(&item.value);
            }
        }
        self
    }

    /// One group label per choice, parallel to [`values`](Self::values), so filtering
    /// needs no remapping.
    pub(crate) fn group_labels(&self) -> Vec<Option<String>> {
        self.groups
            .iter()
            .flat_map(|(label, members)| members.iter().map(move |_| label.clone()))
            .collect()
    }
}

impl<T: super::Options> OptionList<T> {
    /// Every [`Options::options()`](super::Options::options), flat. Empty for `String`.
    ///
    /// ```
    /// # use libero::components::{OptionList, Options};
    /// # #[derive(Clone, Copy, PartialEq, Options)] enum Plan { Free, Pro, Team }
    /// let plans: OptionList<Plan> =
    ///     OptionList::from_options().disabling(|plan| *plan == Plan::Team);
    /// ```
    pub fn from_options() -> Self {
        Self::new(T::options().to_vec())
    }
}

impl<T> From<Vec<T>> for OptionList<T> {
    fn from(values: Vec<T>) -> Self {
        Self::new(values)
    }
}

/// Where a list's choices are in their lifetime.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arrival {
    /// Nothing passed, unlike an empty list: falls back to `Options::options()`.
    Unset,
    /// Being fetched; rows and the empty state are held back.
    Pending,
    Ready,
}

/// The `options` prop: choices that may still be on their way. A `Vec<T>` or a
/// [`Resource`] converts; a failed fetch is an empty list.
#[derive(Clone, PartialEq, Debug)]
pub struct OptionSource<T> {
    list: OptionList<T>,
    arrival: Arrival,
}

impl<T> Default for OptionSource<T> {
    fn default() -> Self {
        Self {
            list: OptionList::default(),
            arrival: Arrival::Unset,
        }
    }
}

impl<T> OptionSource<T> {
    pub(crate) fn is_unset(&self) -> bool {
        self.arrival == Arrival::Unset
    }

    pub(crate) fn is_pending(&self) -> bool {
        self.arrival == Arrival::Pending
    }

    pub(crate) fn list(&self) -> &OptionList<T> {
        &self.list
    }
}

impl<T: super::Options> OptionSource<T> {
    /// The caller's choices, or every `Options::options()` when the prop was left unset.
    pub(crate) fn or_static(&self) -> OptionList<T> {
        match self.is_unset() {
            true => OptionList::from_options(),
            false => self.list.clone(),
        }
    }
}

impl<T> From<OptionList<T>> for OptionSource<T> {
    fn from(list: OptionList<T>) -> Self {
        Self {
            list,
            arrival: Arrival::Ready,
        }
    }
}

impl<T> From<Vec<T>> for OptionSource<T> {
    fn from(values: Vec<T>) -> Self {
        OptionList::new(values).into()
    }
}

/// Pending until the first value; a refetch keeps showing the last answer.
impl<T: Clone + 'static> From<Resource<Vec<T>>> for OptionSource<T> {
    fn from(resource: Resource<Vec<T>>) -> Self {
        OptionSource::<T>::from(resource.read().clone().map(OptionList::from))
    }
}

impl<T: Clone + 'static> From<Resource<OptionList<T>>> for OptionSource<T> {
    fn from(resource: Resource<OptionList<T>>) -> Self {
        OptionSource::from(resource.read().clone())
    }
}

/// `None` is pending: for a caller driving its own fetch.
impl<T> From<Option<OptionList<T>>> for OptionSource<T> {
    fn from(list: Option<OptionList<T>>) -> Self {
        match list {
            Some(list) => list.into(),
            None => Self {
                list: OptionList::default(),
                arrival: Arrival::Pending,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cities() -> OptionList<&'static str> {
        OptionList::grouped()
            .group(
                "Germany",
                ["Berlin".into(), OptionItem::new("Bonn").disabled(true)],
            )
            .group("France", ["Paris"])
    }

    #[test]
    fn a_flat_list_is_one_unnamed_run() {
        let list = OptionList::new(["Apple", "Banana"]);
        assert_eq!(list.values(), ["Apple", "Banana"]);
        assert_eq!(list.group_labels(), [None, None]);
        assert_eq!(list.disabled(), [false, false]);
    }

    /// Groups flatten into one index space, and the parallel arrays line up with it.
    #[test]
    fn groups_flatten_in_order_with_parallel_labels_and_flags() {
        let list = cities();
        assert_eq!(list.values(), ["Berlin", "Bonn", "Paris"]);
        assert_eq!(
            list.group_labels(),
            [
                Some("Germany".to_string()),
                Some("Germany".to_string()),
                Some("France".to_string()),
            ]
        );
        assert_eq!(list.disabled(), [false, true, false]);
    }

    /// The caller's order wins over heading uniqueness: merging would move Kiel before Paris.
    #[test]
    fn a_repeated_group_label_keeps_the_callers_order() {
        let list = cities().group("Germany", ["Kiel"]);
        assert_eq!(list.values(), ["Berlin", "Bonn", "Paris", "Kiel"]);
        assert_eq!(
            list.group_labels(),
            [
                Some("Germany".to_string()),
                Some("Germany".to_string()),
                Some("France".to_string()),
                Some("Germany".to_string()),
            ]
        );
    }

    /// Two adjacent calls with one label cost nothing: the dropdown groups by
    /// run, so equal neighbours are one run and one heading.
    #[test]
    fn two_adjacent_calls_with_one_label_stay_adjacent() {
        let list = OptionList::grouped()
            .group("Germany", ["Berlin"])
            .group("Germany", ["Bonn"]);
        assert_eq!(list.values(), ["Berlin", "Bonn"]);
        assert_eq!(
            list.group_labels(),
            [Some("Germany".to_string()), Some("Germany".to_string())]
        );
    }

    /// Unset is not empty: it is what makes an `Options` enum list itself.
    #[test]
    fn unset_empty_and_pending_are_three_different_answers() {
        let unset = OptionSource::<&str>::default();
        assert!(unset.is_unset() && !unset.is_pending());

        let empty = OptionSource::from(Vec::<&str>::new());
        assert!(!empty.is_unset() && !empty.is_pending());
        assert!(empty.list().values().is_empty());

        let pending = OptionSource::<&str>::from(None::<OptionList<&str>>);
        assert!(!pending.is_unset() && pending.is_pending());
        assert!(pending.list().values().is_empty());
    }

    /// It only adds flags, so it cannot undo an `OptionItem` already disabled.
    #[test]
    fn disabling_flags_the_named_choices_and_never_clears_one() {
        let list = cities().disabling(|city| *city == "Paris");
        assert_eq!(list.values(), ["Berlin", "Bonn", "Paris"]);
        assert_eq!(list.disabled(), [false, true, true]);

        // A predicate that names nothing leaves the list exactly as it was.
        assert_eq!(
            cities().disabling(|_| false).disabled(),
            [false, true, false]
        );
    }

    /// A failed fetch is a ready, empty list - the caller returns an empty
    /// `Vec` and says what went wrong beside the field.
    #[test]
    fn a_ready_empty_list_is_not_pending() {
        let failed = OptionSource::from(Some(OptionList::<&str>::default()));
        assert!(!failed.is_pending());
        assert!(failed.list().values().is_empty());
    }
}
