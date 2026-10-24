use dioxus::prelude::*;

use crate::components::form::FormErrors;

/// What a [`Form`](crate::components::Form) does with a submit its fields let through: a
/// closure over the [`FormEvent`], or [`Submit::parse`], which builds a command from the
/// value first.
pub struct Submit<V: 'static> {
    run: Option<Callback<(FormEvent, V), Result<(), FormErrors>>>,
}

impl<V: 'static> Submit<V> {
    /// Builds a command from the form's value with `parse`, then hands it to `handler`. An
    /// `Err` puts each error on its field and in the summary, as
    /// [`set_errors`](crate::components::FormHandle::set_errors) does, and `handler` doesn't run.
    ///
    /// ```no_run
    /// # use dioxus::prelude::*;
    /// use libero::components::{Fields, Form, FormErrors, RequiredValue, Submit, TextField};
    ///
    /// #[derive(Clone, PartialEq, Default, Fields)]
    /// struct Order { amount: Option<u32> }
    /// struct PlaceOrder { amount: u32 }
    ///
    /// fn place_order(order: &Order) -> Result<PlaceOrder, FormErrors> {
    ///     let amount = order.amount.required(Order::FIELDS.amount())?;
    ///     match amount <= 100 {
    ///         true => Ok(PlaceOrder { amount }),
    ///         false => Err(FormErrors::new().at(Order::FIELDS.amount(), "At most 100 at once.")),
    ///     }
    /// }
    ///
    /// # fn app() -> Element {
    /// let order = use_store(Order::default);
    /// rsx! {
    ///     Form { value: order,
    ///         onsubmit: Submit::parse(place_order, move |command: PlaceOrder| {
    ///             let _ = command.amount;
    ///         }),
    ///         TextField {
    ///             label: "Amount",
    ///             name: Order::FIELDS.amount().parsed().message(|_, _| "Enter a whole number.".into()),
    ///         }
    ///     }
    /// }
    /// # }
    /// ```
    pub fn parse<C: 'static>(
        parse: impl Fn(&V) -> Result<C, FormErrors> + 'static,
        mut handler: impl FnMut(C) + 'static,
    ) -> Self {
        let run = Callback::new(move |(_, value): (FormEvent, V)| {
            handler(parse(&value)?);
            Ok(())
        });
        Self { run: Some(run) }
    }

    /// Runs on a submit nothing blocked. `Err` when `parse` refused the value.
    pub(crate) fn run(&self, event: FormEvent, value: V) -> Result<(), FormErrors> {
        match self.run {
            Some(run) => run.call((event, value)),
            None => Ok(()),
        }
    }

    pub(crate) fn is_some(&self) -> bool {
        self.run.is_some()
    }
}

impl<V: 'static, F: FnMut(FormEvent) + 'static> From<F> for Submit<V> {
    fn from(mut handler: F) -> Self {
        let run = Callback::new(move |(event, _): (FormEvent, V)| {
            handler(event);
            Ok(())
        });
        Self { run: Some(run) }
    }
}

impl<V: 'static> Default for Submit<V> {
    fn default() -> Self {
        Self { run: None }
    }
}

impl<V: 'static> Clone for Submit<V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<V: 'static> Copy for Submit<V> {}

impl<V: 'static> PartialEq for Submit<V> {
    fn eq(&self, other: &Self) -> bool {
        self.run == other.run
    }
}
