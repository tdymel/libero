//! A finite, ordered set of choices, so a component's choices are the caller's own type.

use dioxus::prelude::*;

/// A value one choice stands for; `#[derive(Options)]` writes it for a unit enum.
///
/// `PartialEq` is identity: a type whose fields change must compare its id alone,
/// or the selection vanishes.
///
/// ```rust
/// # use dioxus::prelude::*;
/// use libero::components::{NativeSelect, Options};
///
/// #[derive(Clone, PartialEq)]
/// struct Order {
///     id: u32,
///     customer: String,
/// }
///
/// impl Options for Order {
///     fn label(&self) -> String {
///         format!("#{} - {}", self.id, self.customer)
///     }
/// }
///
/// #[component]
/// fn OrderPicker(orders: Vec<Order>) -> Element {
///     let mut selected = use_signal(|| None::<Order>);
///     rsx! {
///         NativeSelect {
///             label: "Order",
///             placeholder: "Pick an order",
///             value: selected(),
///             options: orders.clone(),
///             onchange: move |order| selected.set(Some(order)),
///         }
///     }
/// }
/// ```
pub trait Options: Clone + PartialEq + 'static {
    /// Every choice, in order. None by default: a runtime set comes through the component's prop.
    fn options() -> &'static [Self]
    where
        Self: Sized,
    {
        &[]
    }

    /// The choice's visible text and accessible name.
    fn label(&self) -> String;

    /// What a form posts for this choice. The derive uses the variant's name, so a
    /// translated label never changes it.
    fn value(&self) -> String {
        self.label()
    }
}

/// A runtime set, listed by the component rather than the type.
impl Options for String {
    fn label(&self) -> String {
        self.clone()
    }
}

/// What a choice is called, and optionally how it is drawn. A bare string is both.
#[derive(Clone, PartialEq)]
pub struct OptionLabel {
    pub(crate) name: String,
    pub(crate) content: Option<Element>,
}

impl OptionLabel {
    /// Drawn `content` with a plain-text accessible `name`. In a segment, a `<label>`,
    /// keep `content` to phrasing content: an `Icon` works, a `Flex` does not.
    ///
    /// ```rust
    /// # use dioxus::prelude::*;
    /// use libero::components::{Icon, OptionLabel, Options, SegmentedControl};
    ///
    /// #[derive(Clone, Copy, PartialEq, Options)]
    /// enum Alignment {
    ///     Left,
    ///     Center,
    ///     Right,
    /// }
    ///
    /// # fn app() -> Element {
    /// let mut alignment = use_signal(|| Alignment::Center);
    /// rsx! {
    ///     SegmentedControl {
    ///         value: alignment(),
    ///         onchange: move |next| alignment.set(next),
    ///         option_label: |alignment: Alignment| OptionLabel::rich(
    ///             alignment.label(),
    ///             rsx! {
    ///                 Icon {
    ///                     variant: "standard",
    ///                     size: "sm",
    ///                     match alignment {
    ///                         Alignment::Left => rsx! { AlignLeftIcon {} },
    ///                         Alignment::Center => rsx! { AlignCenterIcon {} },
    ///                         Alignment::Right => rsx! { AlignRightIcon {} },
    ///                     }
    ///                 }
    ///                 "{alignment.label()}"
    ///             },
    ///         ),
    ///     }
    /// }
    /// # }
    /// # #[component] fn AlignLeftIcon() -> Element { rsx! {} }
    /// # #[component] fn AlignCenterIcon() -> Element { rsx! {} }
    /// # #[component] fn AlignRightIcon() -> Element { rsx! {} }
    /// ```
    pub fn rich(name: impl Into<String>, content: Element) -> Self {
        Self {
            name: name.into(),
            content: Some(content),
        }
    }

    /// The accessible name - always plain text, even when the label is rich.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// What to draw: the rich content, or the name as text.
    pub fn render(&self) -> Element {
        match &self.content {
            Some(content) => content.clone(),
            None => {
                let name = &self.name;
                rsx! { "{name}" }
            }
        }
    }
}

impl From<String> for OptionLabel {
    fn from(name: String) -> Self {
        Self {
            name,
            content: None,
        }
    }
}

impl From<&str> for OptionLabel {
    fn from(name: &str) -> Self {
        Self::from(name.to_string())
    }
}

/// One key per value that follows it by `PartialEq`, its identity: a keyed node
/// moves with its value when the list changes. Equal values get keys of their own.
#[cfg(any(feature = "accordion", feature = "tabs"))]
pub(crate) fn use_identity_keys<T: PartialEq + Clone + 'static>(values: &[T]) -> Vec<String> {
    use std::{cell::RefCell, rc::Rc};
    let known = use_hook(|| Rc::new(RefCell::new((0_u64, Vec::<(T, u64)>::new()))));
    let mut known = known.borrow_mut();
    let (last, previous) = &mut *known;
    *previous = assign_keys(previous, last, values);
    previous.iter().map(|(_, key)| key.to_string()).collect()
}

#[cfg(any(feature = "accordion", feature = "tabs", test))]
fn assign_keys<T: PartialEq + Clone>(
    previous: &[(T, u64)],
    last: &mut u64,
    values: &[T],
) -> Vec<(T, u64)> {
    let mut taken = vec![false; previous.len()];
    values
        .iter()
        .map(|value| {
            let reused = (0..previous.len()).find(|&at| !taken[at] && previous[at].0 == *value);
            let key = match reused {
                Some(at) => {
                    taken[at] = true;
                    previous[at].1
                }
                None => {
                    *last += 1;
                    *last
                }
            };
            (value.clone(), key)
        })
        .collect()
}

/// As [`use_identity_keys`] for labels: a new label where a gone one stood keeps
/// its key, so a renamed entry ("Inbox (3)" to "Inbox (4)") keeps its state.
#[cfg(feature = "menubar")]
pub(crate) fn use_label_keys(labels: &[String]) -> Vec<String> {
    use std::{cell::RefCell, rc::Rc};
    let known = use_hook(|| Rc::new(RefCell::new((0_u64, Vec::<(String, u64)>::new()))));
    let mut known = known.borrow_mut();
    let (last, previous) = &mut *known;
    *previous = assign_label_keys(previous, last, labels);
    previous.iter().map(|(_, key)| key.to_string()).collect()
}

#[cfg(any(feature = "menubar", test))]
fn assign_label_keys(
    previous: &[(String, u64)],
    last: &mut u64,
    labels: &[String],
) -> Vec<(String, u64)> {
    let mut taken = vec![false; previous.len()];
    let mut keys: Vec<Option<u64>> = labels
        .iter()
        .map(|label| {
            let at = (0..previous.len()).find(|&at| !taken[at] && previous[at].0 == *label)?;
            taken[at] = true;
            Some(previous[at].1)
        })
        .collect();
    for (at, key) in keys.iter_mut().enumerate() {
        let renamed =
            key.is_none() && at < previous.len() && !taken[at] && !labels.contains(&previous[at].0);
        if renamed {
            taken[at] = true;
            *key = Some(previous[at].1);
        }
    }
    labels
        .iter()
        .zip(keys)
        .map(|(label, key)| {
            let key = key.unwrap_or_else(|| {
                *last += 1;
                *last
            });
            (label.clone(), key)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{assign_keys, assign_label_keys};

    #[test]
    fn a_key_follows_its_value_through_an_insert_and_a_removal() {
        let mut last = 0;
        let first = assign_keys(&[], &mut last, &['b', 'c']);
        assert_eq!(first, [('b', 1), ('c', 2)]);
        let inserted = assign_keys(&first, &mut last, &['a', 'b', 'c']);
        assert_eq!(inserted, [('a', 3), ('b', 1), ('c', 2)]);
        let removed = assign_keys(&inserted, &mut last, &['c', 'a']);
        assert_eq!(removed, [('c', 2), ('a', 3)]);
    }

    #[test]
    fn equal_values_get_keys_of_their_own() {
        let mut last = 0;
        let first = assign_keys(&[], &mut last, &['a', 'a']);
        assert_eq!(first, [('a', 1), ('a', 2)]);
        assert_eq!(assign_keys(&first, &mut last, &['a']), [('a', 1)]);
    }

    fn labels(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    fn keys(assigned: &[(String, u64)]) -> Vec<u64> {
        assigned.iter().map(|(_, key)| *key).collect()
    }

    #[test]
    fn a_renamed_label_keeps_its_key_where_the_old_one_stood() {
        let mut last = 0;
        let first = assign_label_keys(&[], &mut last, &labels(&["File", "Inbox (3)"]));
        assert_eq!(keys(&first), [1, 2]);
        let renamed = assign_label_keys(&first, &mut last, &labels(&["File", "Inbox (4)"]));
        assert_eq!(keys(&renamed), [1, 2]);
    }

    #[test]
    fn a_label_key_follows_its_label_through_an_insert_and_a_removal() {
        let mut last = 0;
        let first = assign_label_keys(&[], &mut last, &labels(&["File", "Edit"]));
        let inserted = assign_label_keys(&first, &mut last, &labels(&["New", "File", "Edit"]));
        assert_eq!(keys(&inserted), [3, 1, 2]);
        let removed = assign_label_keys(&inserted, &mut last, &labels(&["Edit"]));
        assert_eq!(keys(&removed), [2]);
        // Equal labels keep keys of their own.
        let twice = assign_label_keys(&removed, &mut last, &labels(&["Edit", "Edit"]));
        assert_eq!(keys(&twice), [2, 4]);
    }
}
