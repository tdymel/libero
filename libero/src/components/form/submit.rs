use std::{future::Future, pin::Pin};

use dioxus::prelude::*;

use crate::components::form::FormErrors;

/// What a [`Form`](crate::components::Form) does with a submit its fields let through: a
/// closure over the [`FormEvent`] or an `Option<EventHandler<FormEvent>>`, or [`Submit::parse`], which builds a command from the
/// value first. [`Submit::parse_async`] and [`Submit::task`] take async handlers; a closure
/// that returns a future is not one, so wrap it in [`Submit::task`].
///
/// A `Submit` keeps its closure in the scope that built it until that scope unmounts, so
/// one built inline adds one per render. A parent that re-renders often builds it once:
/// `let onsubmit = use_hook(|| Submit::parse(parse, handler));`.
pub struct Submit<V: 'static> {
    run: Option<Callback<(FormEvent, V), Result<(), Outcome>>>,
}

/// What a submit handler left to do.
pub(crate) enum Outcome {
    /// `parse` refused the value.
    Refused(FormErrors),
    /// An async handler, for the form to spawn.
    Running(Pin<Box<dyn Future<Output = Result<(), FormErrors>>>>),
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
            handler(parse(&value).map_err(Outcome::Refused)?);
            Ok(())
        });
        Self { run: Some(run) }
    }

    /// [`parse`](Self::parse) with an async `handler`, such as a request. While it runs,
    /// [`is_submitting`](crate::components::FormHandle::is_submitting) is `true` and a submit is
    /// ignored; an `Err` it returns lands as [`set_errors`](crate::components::FormHandle::set_errors) does.
    ///
    /// ```no_run
    /// # use dioxus::prelude::*;
    /// use libero::components::{Button, Fields, Form, FormErrors, Submit, TextField, use_form};
    ///
    /// #[derive(Clone, PartialEq, Default, Fields)]
    /// struct Login { email: String }
    /// struct SignIn { email: String }
    ///
    /// fn sign_in(login: &Login) -> Result<SignIn, FormErrors> {
    ///     Ok(SignIn { email: login.email.clone() })
    /// }
    /// # async fn request(_: SignIn) -> Result<(), ()> { Ok(()) }
    ///
    /// # fn app() -> Element {
    /// let login = use_store(Login::default);
    /// let form = use_form();
    /// rsx! {
    ///     Form { form, value: login,
    ///         onsubmit: Submit::parse_async(sign_in, |command: SignIn| async move {
    ///             request(command).await.map_err(|_| FormErrors::new().form("Wrong email or password."))
    ///         }),
    ///         TextField { label: "Email", name: Login::FIELDS.email() }
    ///         Button { r#type: "submit", loading: form.is_submitting(), "Sign in" }
    ///     }
    /// }
    /// # }
    /// ```
    pub fn parse_async<C: 'static, F>(
        parse: impl Fn(&V) -> Result<C, FormErrors> + 'static,
        mut handler: impl FnMut(C) -> F + 'static,
    ) -> Self
    where
        F: Future<Output = Result<(), FormErrors>> + 'static,
    {
        let run = Callback::new(move |(_, value): (FormEvent, V)| {
            let command = parse(&value).map_err(Outcome::Refused)?;
            Err(Outcome::Running(Box::pin(handler(command))))
        });
        Self { run: Some(run) }
    }

    /// An async `handler` over the [`FormEvent`], as [`parse_async`](Self::parse_async) runs it.
    ///
    /// ```no_run
    /// # use dioxus::prelude::*;
    /// use libero::components::{Form, Submit};
    /// # async fn save() {}
    ///
    /// # fn app() -> Element {
    /// rsx! {
    ///     Form::<()> { onsubmit: Submit::task(|_| async move {
    ///         save().await;
    ///         Ok(())
    ///     }) }
    /// }
    /// # }
    /// ```
    pub fn task<F>(mut handler: impl FnMut(FormEvent) -> F + 'static) -> Self
    where
        F: Future<Output = Result<(), FormErrors>> + 'static,
    {
        let run = Callback::new(move |(event, _): (FormEvent, V)| {
            Err(Outcome::Running(Box::pin(handler(event))))
        });
        Self { run: Some(run) }
    }

    /// Runs on a submit nothing blocked. `Ok` when the handler is done.
    pub(crate) fn run(&self, event: FormEvent, value: V) -> Result<(), Outcome> {
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

/// The handler `onsubmit` took before `Submit`, so a wrapper forwarding one keeps compiling.
impl<V: 'static> From<EventHandler<FormEvent>> for Submit<V> {
    fn from(handler: EventHandler<FormEvent>) -> Self {
        Self::from(move |event: FormEvent| handler.call(event))
    }
}

/// `None` submits with no handler, as an unset `onsubmit` does.
impl<V: 'static> From<Option<EventHandler<FormEvent>>> for Submit<V> {
    fn from(handler: Option<EventHandler<FormEvent>>) -> Self {
        handler.map(Self::from).unwrap_or_default()
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

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;
    use crate::{components::form::Form, platform::submit_event};

    thread_local! {
        static FIRED: Cell<u32> = const { Cell::new(0) };
    }

    /// A wrapper written against the old `Option<EventHandler<FormEvent>>` prop.
    #[component]
    fn Wrapper(onsubmit: Option<EventHandler<FormEvent>>) -> Element {
        let set: Submit<()> = onsubmit.into();
        assert_eq!(set.is_some(), onsubmit.is_some());
        assert!(set.run(submit_event(None), ()).is_ok());
        rsx! { Form::<()> { onsubmit } }
    }

    #[test]
    fn an_old_event_handler_still_forwards_and_fires() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                Wrapper {}
                Wrapper { onsubmit: |_| FIRED.with(|fired| fired.set(fired.get() + 1)) }
            }
        });
        dom.rebuild_in_place();
        assert_eq!(FIRED.with(Cell::get), 1);
    }
}
