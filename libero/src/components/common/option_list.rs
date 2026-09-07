//! The shape of a list of choices, and where it comes from.
//!
//! [`Options`](super::Options) says what one choice is. This says how a
//! component is handed a set of them: flat or in named groups, with any of
//! them disabled, and either already here or still being fetched.
//!
//! Three types, each one axis:
//!
//! - [`OptionItem`] - one choice plus the flags a *list* gives it. `disabled`
//!   belongs here rather than on the caller's `T`, because a type does not know
//!   which of its values this particular field refuses.
//! - [`OptionList`] - flat, or a run of named groups.
//! - [`OptionSource`] - an [`OptionList`] that may not have arrived yet. It is
//!   what the `options` prop takes, and a `Vec<T>` converts into it, so every
//!   call site that passes one keeps working.

use dioxus::prelude::*;

/// One choice, and the flags the list around it gives that choice.
///
/// A plain value converts, so a list of them mixes freely with the flagged
/// ones: `[Berlin.into(), OptionItem::new(Bonn).disabled(true)]`.
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

    /// A row that is drawn and read out, `aria-disabled`, but that the arrows,
    /// typeahead and clicks all pass over.
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

/// The choices a list draws: one unnamed run, or several named ones.
///
/// Groups are explicit rather than derived from a field or a closure, because
/// the caller is the only one who knows both the order the groups go in and
/// what each is called. Nothing below the component sees them: they change how
/// the rows are wrapped, never which index a row reports.
#[derive(Clone, PartialEq, Debug)]
pub struct OptionList<T> {
    /// In order. A `None` label is an unnamed run, which is what a flat list
    /// is made of.
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

    /// A named run of choices, after the ones already added.
    ///
    /// **The caller's order is kept, always.** A label used twice with another
    /// group between the two - `group("A", ..).group("B", ..).group("A", ..)` -
    /// draws the heading twice rather than merging the two runs, because
    /// merging would silently reorder options the caller listed in a
    /// particular order. Two groups that happen to share a name are legal:
    /// nothing requires a `role="group"` to be uniquely named, and each
    /// heading gets an id of its own, keyed on its run's first row.
    ///
    /// Two *adjacent* calls with one label still draw one heading - the
    /// dropdown builds its groups by run, so equal neighbours are one run and
    /// nothing is lost.
    pub fn group(
        mut self,
        label: impl Into<String>,
        items: impl IntoIterator<Item = impl Into<OptionItem<T>>>,
    ) -> Self {
        let items = items.into_iter().map(Into::into).collect();
        self.groups.push((Some(label.into()), items));
        self
    }

    /// Every choice, groups flattened away, in the order they are drawn. This
    /// is the index space every component below works in.
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

    /// One group label per choice, parallel to [`values`](Self::values),
    /// `None` for a choice in an unnamed run.
    ///
    /// Parallel rather than nested, because the search filter drops rows from
    /// the middle of the list: a group's members stay next to each other
    /// whatever survives, so the dropdown rebuilds the runs from this and
    /// nothing has to be remapped.
    pub(crate) fn group_labels(&self) -> Vec<Option<String>> {
        self.groups
            .iter()
            .flat_map(|(label, members)| members.iter().map(move |_| label.clone()))
            .collect()
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
    /// The caller passed nothing, which is not the same as passing an empty
    /// list: it is what lets a component fall back to
    /// [`Options::options()`](super::Options::options).
    Unset,
    /// Being fetched. The rows and the empty state are both held back - an
    /// async list is empty between the request and its answer, and flashing
    /// "no results" there is a lie.
    Pending,
    Ready,
}

/// The `options` prop: a list of choices that may still be on its way.
///
/// A `Vec<T>` converts, so a call site that has its options already passes them
/// exactly as it always did. A [`Resource`] converts too, and that is the whole
/// of the async wiring - the component reads pending against ready itself and
/// derives the loading row, `aria-busy` and the held-back empty state from it.
///
/// There is deliberately **no separate `loading` flag**: two ways to say the
/// same thing can be set to disagree.
///
/// A fetch that failed is an **empty list**. The library knows pending and
/// ready, nothing else; an error message belongs beside the field, where the
/// caller can say what went wrong and offer a retry.
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
    /// Whether the caller passed no options at all, so a component may list
    /// `Options::options()` instead.
    pub(crate) fn is_unset(&self) -> bool {
        self.arrival == Arrival::Unset
    }

    /// Whether the choices are still being fetched.
    pub(crate) fn is_pending(&self) -> bool {
        self.arrival == Arrival::Pending
    }

    pub(crate) fn list(&self) -> &OptionList<T> {
        &self.list
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

/// A resource is pending until it first holds a value. A refetch that still
/// holds the last answer keeps showing it rather than blanking the list, which
/// is what makes search-as-you-type readable.
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

/// The shape both resource conversions land on, and the way to say "pending"
/// without a [`Resource`] - a caller driving its own fetch passes `None` while
/// it runs and `Some(list)` once it has an answer, an empty one included.
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

    /// Groups flatten into one index space, and the parallel arrays line up
    /// with it - which is the whole contract everything below the prop relies
    /// on.
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

    /// The caller's order wins over heading uniqueness. Merging the two
    /// "Germany" runs would put Kiel before Paris although the caller listed it
    /// after - silently reordering data they supplied. Drawing the heading
    /// twice is the honest answer, and two groups sharing a name are legal.
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

    /// A failed fetch is a ready, empty list - the caller returns an empty
    /// `Vec` and says what went wrong beside the field.
    #[test]
    fn a_ready_empty_list_is_not_pending() {
        let failed = OptionSource::from(Some(OptionList::<&str>::default()));
        assert!(!failed.is_pending());
        assert!(failed.list().values().is_empty());
    }
}
