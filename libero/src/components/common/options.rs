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
#[cfg(any(feature = "accordion", feature = "menubar", feature = "tabs"))]
pub(crate) fn use_identity_keys<T: PartialEq + Clone + 'static>(values: &[T]) -> Vec<String> {
    use std::{cell::RefCell, rc::Rc};
    let known = use_hook(|| Rc::new(RefCell::new((0_u64, Vec::<(T, u64)>::new()))));
    let mut known = known.borrow_mut();
    let (last, previous) = &mut *known;
    *previous = assign_keys(previous, last, values);
    previous.iter().map(|(_, key)| key.to_string()).collect()
}

#[cfg(any(feature = "accordion", feature = "menubar", feature = "tabs", test))]
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

#[cfg(test)]
mod tests {
    use super::assign_keys;

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
}
