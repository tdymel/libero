use std::{fmt, rc::Rc};

use super::field_error::{FieldError, Worded};
use crate::localization::Localization;

/// Errors by field path, from a form's `parse` step or a server's answer. Each
/// lands on the field with that `name` and in the error summary.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// use libero::components::{Fields, FormErrors, use_form};
///
/// #[derive(Clone, PartialEq, Default, Fields)]
/// struct Signup { email: String }
///
/// # fn app() -> Element {
/// let form = use_form();
/// // After the server answered that the address is taken:
/// form.set_errors(FormErrors::new().at(Signup::FIELDS.email(), "This address is taken."));
/// # rsx! {}
/// # }
/// ```
#[derive(Clone, Default)]
pub struct FormErrors {
    entries: Vec<(String, Rc<dyn FieldError>)>,
}

impl FormErrors {
    pub fn new() -> Self {
        Self::default()
    }

    /// `error` on the field named `path`: a [`FieldPath`](super::FieldPath), a
    /// [`FieldName`](super::FieldName) or its spelling.
    pub fn at(mut self, path: impl fmt::Display, error: impl FieldError) -> Self {
        self.entries.push((path.to_string(), Rc::new(error)));
        self
    }

    /// [`at`](Self::at) for an error with no [`FieldError`] of its own, worded by `message`.
    pub fn at_with<E: 'static>(
        self,
        path: impl fmt::Display,
        error: E,
        message: impl Fn(&E, &Localization) -> String + 'static,
    ) -> Self {
        let words = Rc::new(message);
        self.at(path, Worded { error, words })
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(crate) fn into_entries(self) -> Vec<(String, Rc<dyn FieldError>)> {
        self.entries
    }
}

impl fmt::Debug for FormErrors {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_list()
            .entries(self.entries.iter().map(|(path, _)| path))
            .finish()
    }
}

/// A missing value as a [`FormErrors`] entry, so a `parse` step needs no `unwrap()`.
///
/// ```no_run
/// use libero::components::{Fields, FormErrors, RequiredValue};
///
/// #[derive(Clone, PartialEq, Default, Fields)]
/// struct Order { amount: Option<u32> }
/// struct PlaceOrder { amount: u32 }
///
/// fn place_order(order: &Order) -> Result<PlaceOrder, FormErrors> {
///     let amount = order.amount.required(Order::FIELDS.amount())?;
///     Ok(PlaceOrder { amount })
/// }
/// ```
pub trait RequiredValue<T> {
    /// The value, else an error on `path` worded as a `required` field left empty.
    fn required(self, path: impl fmt::Display) -> Result<T, FormErrors>;
}

impl<T: Clone> RequiredValue<T> for &Option<T> {
    fn required(self, path: impl fmt::Display) -> Result<T, FormErrors> {
        self.clone()
            .ok_or_else(|| FormErrors::new().at(path, Missing))
    }
}

/// The required text of the active language.
struct Missing;

impl FieldError for Missing {
    fn message(&self, locale: &Localization) -> String {
        locale.form.required.to_string()
    }
}
