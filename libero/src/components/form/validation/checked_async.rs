use std::{
    any::{Any, TypeId},
    cell::{Cell, RefCell},
    future::{Future, poll_fn},
    pin::Pin,
    rc::Rc,
    task::{Poll, Waker},
    time::Duration,
};

use dioxus::{core::Task, prelude::*};

use super::{
    context::FormScope,
    field_error::{FieldError, Worded, WordsFn},
};
use crate::{
    components::form::FieldStatus,
    localization::Localization,
    platform::{TimerApi, timer},
};

/// How long [`checked_async`] waits after the last edit before it asks, in ms.
const DEBOUNCE_MS: u64 = 300;

/// An async rule over a mapped field's domain value, built by [`checked_async`].
pub struct CheckedAsync<V: 'static, E: 'static> {
    rule: Ask<V, E>,
    /// The rule's type when it captures nothing.
    key: Option<TypeId>,
    debounce: u64,
    warning: bool,
}

enum Ask<V: 'static, E: 'static> {
    /// Fails with `E`, which the field's words cover.
    Unworded(Rc<AsyncFn<V, E>>),
    /// Worded by its own `message`, with the words' type when they capture nothing.
    Worded(Rc<RunFn<V>>, Option<TypeId>),
}

type AsyncFn<V, E> = dyn Fn(V) -> Pin<Box<dyn Future<Output = Result<(), E>>>>;
type RunFn<V> = dyn Fn(V) -> Pin<Box<dyn Future<Output = Option<Rc<dyn FieldError>>>>>;

/// A rule that asks elsewhere - a server - whether a parsed value is fine, for a mapped field's
/// `.rules`. It runs once the value parsed and the other rules found no error, 300 ms after the
/// last edit ([`debounce`](CheckedAsync::debounce)); meanwhile the field is [`FieldStatus::Pending`] and a submit waits for
/// the answer. An answer for a value no longer there is dropped. The error takes the field's
/// words, as a [`checked`](super::checked) rule's does, or its own [`message`](CheckedAsync::message).
///
/// ```no_run
/// # use dioxus::prelude::*;
/// use libero::components::{Fields, Form, TextField, checked_async};
/// use libero::localization::Localization;
///
/// enum EmailError { Invalid, Taken }
/// fn email(text: String) -> Result<String, EmailError> {
///     match text.contains('@') { true => Ok(text), false => Err(EmailError::Invalid) }
/// }
/// fn email_message(error: &EmailError, _: &Localization) -> String {
///     match error {
///         EmailError::Invalid => "Not an email address.".into(),
///         EmailError::Taken => "This address is taken.".into(),
///     }
/// }
/// async fn email_free(email: String) -> Result<(), EmailError> {
///     // Ask the server here.
///     match email.as_str() { "taken@example.com" => Err(EmailError::Taken), _ => Ok(()) }
/// }
///
/// #[derive(Clone, PartialEq, Default, Fields)]
/// struct Signup { email: Option<String> }
///
/// # fn app() -> Element {
/// let signup = use_store(Signup::default);
/// rsx! {
///     Form { value: signup,
///         TextField { label: "Email", required: true,
///             name: Signup::FIELDS.email().map(String::clone, email).message(email_message)
///                 .rules([checked_async(email_free).debounce(400)]) }
///     }
/// }
/// # }
/// ```
pub fn checked_async<V, E, F, R>(rule: F) -> CheckedAsync<V, E>
where
    V: 'static,
    E: 'static,
    F: Fn(V) -> R + 'static,
    R: Future<Output = Result<(), E>> + 'static,
{
    CheckedAsync {
        key: (size_of::<F>() == 0).then(TypeId::of::<F>),
        rule: Ask::Unworded(Rc::new(move |value| Box::pin(rule(value)))),
        debounce: DEBOUNCE_MS,
        warning: false,
    }
}

impl<V: 'static, E: 'static> CheckedAsync<V, E> {
    /// How long to wait after the last edit before asking, in ms; 300 by default.
    pub fn debounce(mut self, ms: u64) -> Self {
        self.debounce = ms;
        self
    }

    /// Shows the error as a warning, which never blocks a submit; a submit does not wait for it.
    pub fn warning(mut self) -> Self {
        self.warning = true;
        self
    }

    /// Words the error, for an `E` that is not the field's error type. The rule then fits
    /// any mapped field over `V`.
    pub fn message<X: 'static, W>(self, words: W) -> CheckedAsync<V, X>
    where
        W: Fn(&E, &Localization) -> String + 'static,
    {
        let key = (size_of::<W>() == 0).then(TypeId::of::<W>);
        let rule = match self.rule {
            Ask::Unworded(rule) => Ask::Worded(word_with(rule, Rc::new(words)), key),
            Ask::Worded(run, key) => Ask::Worded(run, key),
        };
        CheckedAsync {
            rule,
            key: self.key,
            debounce: self.debounce,
            warning: self.warning,
        }
    }

    /// The rule with its words: its own, else the field's.
    pub(crate) fn worded(self, words: Rc<WordsFn<E>>, words_key: Option<TypeId>) -> AsyncRule<V> {
        let (run, words_key) = match self.rule {
            Ask::Unworded(rule) => (word_with(rule, words), words_key),
            Ask::Worded(run, key) => (run, key),
        };
        AsyncRule {
            run,
            key: self.key.zip(words_key),
            debounce: self.debounce,
            warning: self.warning,
        }
    }
}

fn word_with<V: 'static, E: 'static>(
    rule: Rc<AsyncFn<V, E>>,
    words: Rc<WordsFn<E>>,
) -> Rc<RunFn<V>> {
    Rc::new(move |value: V| {
        let asked = rule(value);
        let words = words.clone();
        Box::pin(async move {
            asked
                .await
                .err()
                .map(|error| Rc::new(Worded { error, words }) as Rc<dyn FieldError>)
        })
    })
}

/// An async rule with its words, as the field's adapter keeps it.
pub(crate) struct AsyncRule<V: 'static> {
    run: Rc<RunFn<V>>,
    key: Option<(TypeId, TypeId)>,
    debounce: u64,
    warning: bool,
}

impl<V: 'static> Clone for AsyncRule<V> {
    fn clone(&self) -> Self {
        Self {
            run: self.run.clone(),
            key: self.key,
            debounce: self.debounce,
            warning: self.warning,
        }
    }
}

/// Equal by both types when neither captures, as a `checked` rule is.
impl<V: 'static> PartialEq for AsyncRule<V> {
    fn eq(&self, other: &Self) -> bool {
        self.key.is_some()
            && self.key == other.key
            && self.debounce == other.debounce
            && self.warning == other.warning
    }
}

/// A field's async rules: the value they last asked about, their answer, the run under way.
pub(crate) struct Checking {
    asked: RefCell<Option<Rc<dyn Any>>>,
    /// `None` while the rules run.
    answer: RefCell<Option<FieldStatus>>,
    /// Bumped by an answer; read in render, so an answer re-renders the field.
    answered: Signal<u32>,
    /// Counts the runs, so an answer from an older one is dropped.
    run: Cell<u32>,
    task: Cell<Option<Task>>,
    /// Whether the run under way may end in an error, which a submit waits for.
    blocks: Cell<bool>,
    /// The debounce under way, which a waiting submit ends at once.
    hurry: RefCell<Option<Rc<Hurry>>>,
    waiters: RefCell<Vec<Waker>>,
    form: Option<(FormScope, usize)>,
}

impl Checking {
    /// Signals owned by `owner`, the field's scope.
    pub(crate) fn new(owner: ScopeId, form: Option<(FormScope, usize)>) -> Rc<Self> {
        let checking = Rc::new(Self {
            asked: RefCell::new(None),
            answer: RefCell::new(None),
            answered: Signal::new_in_scope(0, owner),
            run: Cell::new(0),
            task: Cell::new(None),
            blocks: Cell::new(false),
            hurry: RefCell::new(None),
            waiters: RefCell::new(Vec::new()),
            form,
        });
        if let Some((mut scope, key)) = form {
            scope.add_check(key, checking.clone());
        }
        checking
    }

    /// The answer for `value`, else `Pending` while `rules` ask about it. Reactive.
    pub(crate) fn check<V: Clone + PartialEq + 'static>(
        self: &Rc<Self>,
        value: &V,
        rules: &[AsyncRule<V>],
        locale: &'static Localization,
    ) -> FieldStatus {
        let _ = (self.answered)();
        let same = self
            .asked
            .borrow()
            .as_ref()
            .and_then(|asked| asked.downcast_ref::<V>())
            .is_some_and(|asked| asked == value);
        if !same {
            self.start(value.clone(), rules.to_vec(), locale);
        }
        self.answer.borrow().clone().unwrap_or(FieldStatus::Pending)
    }

    fn start<V: Clone + 'static>(
        self: &Rc<Self>,
        value: V,
        rules: Vec<AsyncRule<V>>,
        locale: &'static Localization,
    ) {
        self.stop();
        let run = self.run.get().wrapping_add(1);
        self.run.set(run);
        *self.asked.borrow_mut() = Some(Rc::new(value.clone()));
        let hurry = Rc::new(Hurry::default());
        *self.hurry.borrow_mut() = Some(hurry.clone());
        let ms = rules.iter().map(|rule| rule.debounce).max().unwrap_or(0);
        let blocks = rules.iter().any(|rule| !rule.warning);
        let wait = debounce(hurry, timer(), ms);
        let checking = self.clone();
        let task = spawn(async move {
            wait.await;
            let mut warning = None;
            let mut status = None;
            for rule in &rules {
                let Some(error) = (rule.run)(value.clone()).await else {
                    continue;
                };
                let message = error.message(locale);
                if !rule.warning {
                    status = Some(FieldStatus::Error(message));
                    break;
                }
                warning.get_or_insert(FieldStatus::Warning(message));
            }
            checking.answer_with(run, status.or(warning).unwrap_or_default());
        });
        self.task.set(Some(task));
        self.blocks.set(blocks);
    }

    fn answer_with(&self, run: u32, status: FieldStatus) {
        if self.run.get() != run {
            return;
        }
        self.task.set(None);
        if let Some((mut scope, key)) = self.form {
            scope.answer(key, &status);
        }
        *self.answer.borrow_mut() = Some(status);
        let mut answered = self.answered;
        answered += 1;
        self.wake();
    }

    /// Cancels the run under way; its answer never lands.
    fn stop(&self) {
        if let Some(task) = self.task.take() {
            task.cancel();
        }
        *self.answer.borrow_mut() = None;
        self.hurry.borrow_mut().take();
    }

    /// The field shows no value the rules may judge: the run under way ends, and a value that
    /// got no answer is asked again when it comes back.
    pub(crate) fn idle(&self) {
        if self.task.get().is_some() {
            self.stop();
            self.asked.borrow_mut().take();
            self.wake();
        }
    }

    /// Back to the start: no run, no answer kept.
    pub(crate) fn forget(&self) {
        self.stop();
        self.asked.borrow_mut().take();
        self.wake();
    }

    /// The field unmounts and its task goes with its scope: forgotten, not cancelled, as the
    /// task's id may already be another's.
    pub(crate) fn release(&self) {
        self.run.set(self.run.get().wrapping_add(1));
        self.task.take();
        self.wake();
    }

    /// Whether the rules are asking about the current value.
    pub(crate) fn pending(&self) -> bool {
        self.task.get().is_some()
    }

    /// Whether a submit waits for the rules asking now: one of them is no warning.
    pub(crate) fn blocking(&self) -> bool {
        self.pending() && self.blocks.get()
    }

    /// Ends the debounce, so the rules ask now.
    pub(crate) fn hurry(&self) {
        if let Some(hurry) = self.hurry.borrow().as_ref() {
            hurry.end();
        }
    }

    /// Ready once no run is under way.
    pub(crate) fn wait(self: &Rc<Self>) -> impl Future<Output = ()> + 'static {
        let checking = self.clone();
        poll_fn(move |context| match checking.pending() {
            true => {
                checking.waiters.borrow_mut().push(context.waker().clone());
                Poll::Pending
            }
            false => Poll::Ready(()),
        })
    }

    fn wake(&self) {
        let waiters = std::mem::take(&mut *self.waiters.borrow_mut());
        for waker in waiters {
            waker.wake();
        }
    }
}

/// A debounce a waiting submit can end early.
#[derive(Default)]
struct Hurry {
    done: Cell<bool>,
    waker: RefCell<Option<Waker>>,
}

impl Hurry {
    fn end(&self) {
        self.done.set(true);
        if let Some(waker) = self.waker.borrow_mut().take() {
            waker.wake();
        }
    }
}

/// Ready `ms` after the call, or once hurried. Without a timer (SSR) nothing would end it, so at once.
fn debounce(
    hurry: Rc<Hurry>,
    timer: Option<&'static dyn TimerApi>,
    ms: u64,
) -> impl Future<Output = ()> + 'static {
    let subscription = timer.filter(|_| ms > 0).map(|timer| {
        let fired = hurry.clone();
        timer.after(Duration::from_millis(ms), Box::new(move || fired.end()))
    });
    poll_fn(move |context| {
        if subscription.is_none() || hurry.done.get() {
            return Poll::Ready(());
        }
        *hurry.waker.borrow_mut() = Some(context.waker().clone());
        Poll::Pending
    })
}

#[cfg(test)]
mod tests {
    use std::task::Context;

    use super::*;

    #[test]
    fn without_a_timer_the_debounce_ends_at_once() {
        let mut wait = Box::pin(debounce(Rc::new(Hurry::default()), None, DEBOUNCE_MS));
        let mut context = Context::from_waker(Waker::noop());
        assert!(wait.as_mut().poll(&mut context).is_ready());
    }
}
