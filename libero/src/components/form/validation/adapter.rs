use std::{
    any::Any, any::TypeId, borrow::Cow, convert::Infallible, fmt::Display, rc::Rc, str::FromStr,
};

use super::{
    checked::Checked,
    checked_async::{AsyncRule, CheckedAsync, Checking},
    field_error::{FieldError, Parse, Worded, WordsFn},
    path::{FieldName, FieldPath, Step},
    validator::Validator,
};
use crate::{
    components::{
        common::{Choice, Choices},
        form::{FieldStatus, Validators},
    },
    localization::Localization,
};

/// A value a field edits, and when it holds nothing. A mapped field's `parse`
/// sees only the [`Inner`](Self::Inner) of a value that is not empty.
pub trait Wire: Clone + Default + 'static {
    type Inner: 'static;

    /// `None` when empty: blank text, `None`, an empty list.
    fn into_inner(self) -> Option<Self::Inner>;

    fn from_inner(inner: Self::Inner) -> Self;
}

/// Blank is empty; `parse` gets the text untrimmed.
impl Wire for String {
    type Inner = String;

    fn into_inner(self) -> Option<String> {
        (!self.trim().is_empty()).then_some(self)
    }

    fn from_inner(inner: String) -> Self {
        inner
    }
}

impl<X: Clone + 'static> Wire for Option<X> {
    type Inner = X;

    fn into_inner(self) -> Option<X> {
        self
    }

    fn from_inner(inner: X) -> Self {
        Some(inner)
    }
}

impl<X: Clone + 'static> Wire for Vec<X> {
    type Inner = Vec<X>;

    fn into_inner(self) -> Option<Vec<X>> {
        (!self.is_empty()).then_some(self)
    }

    fn from_inner(inner: Vec<X>) -> Self {
        inner
    }
}

/// A field's path bound to a domain type `T` through `format` and `parse`.
/// Becomes the field's `name` once every error of type `E` has its words:
/// `E` implements [`FieldError`], or `.message(fn)` gives them.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Fields, Form, TextField};
/// # use libero::localization::Localization;
/// #[derive(Clone, PartialEq)]
/// struct Iban(String);
/// enum IbanError { Format }
/// impl Iban {
///     fn new(text: String) -> Result<Iban, IbanError> {
///         let text = text.replace(' ', "");
///         match text.len() == 22 { true => Ok(Iban(text)), false => Err(IbanError::Format) }
///     }
///     fn text(&self) -> String { self.0.clone() }
/// }
/// fn iban_message(_: &IbanError, _: &Localization) -> String { "Not an IBAN.".into() }
///
/// #[derive(Clone, PartialEq, Default, Fields)]
/// struct Payee { iban: Option<Iban> }
///
/// # fn app() -> Element {
/// let payee = use_store(Payee::default);
/// rsx! {
///     Form { value: payee,
///         TextField { label: "IBAN", required: true,
///             name: Payee::FIELDS.iban().map(Iban::text, Iban::new).message(iban_message) }
///     }
/// }
/// # }
/// ```
pub struct Mapped<T: 'static, I: 'static, E: 'static, S = Unworded> {
    path: Cow<'static, str>,
    steps: Option<Cow<'static, [Step]>>,
    format: Rc<dyn Fn(&T) -> I>,
    parse: Rc<dyn Fn(I) -> Result<T, E>>,
    words: S,
    rules: Vec<MappedRule<T, E>>,
    /// The types of `format` and `parse` when both capture nothing.
    key: Option<(TypeId, TypeId)>,
    slot: Slot<T>,
    /// What a single pick shows while nothing is stored.
    fallback: Option<I>,
    /// What `format` and `parse` capture, compared in place of nothing captured.
    data: Option<Rc<dyn Same>>,
}

/// Where a mapped field's domain value sits in the form's value, and when it holds none.
pub(crate) struct Slot<T: 'static> {
    get: for<'a> fn(&'a dyn Any) -> Option<Option<&'a T>>,
    put: fn(&mut dyn Any, Option<T>) -> bool,
}

impl<T: 'static> Clone for Slot<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for Slot<T> {}

impl<T: 'static> Slot<T> {
    /// An `Option<T>`, `None` when empty.
    fn option() -> Self {
        fn get<T: 'static>(slot: &dyn Any) -> Option<Option<&T>> {
            slot.downcast_ref::<Option<T>>().map(Option::as_ref)
        }
        fn put<T: 'static>(slot: &mut dyn Any, value: Option<T>) -> bool {
            slot.downcast_mut::<Option<T>>()
                .map(|slot| *slot = value)
                .is_some()
        }
        Self {
            get: get::<T>,
            put: put::<T>,
        }
    }
}

impl<X: 'static> Slot<Vec<X>> {
    /// A list, empty when it holds nothing.
    fn list() -> Self {
        fn get<X: 'static>(slot: &dyn Any) -> Option<Option<&Vec<X>>> {
            slot.downcast_ref::<Vec<X>>()
                .map(|list| (!list.is_empty()).then_some(list))
        }
        fn put<X: 'static>(slot: &mut dyn Any, value: Option<Vec<X>>) -> bool {
            slot.downcast_mut::<Vec<X>>()
                .map(|slot| *slot = value.unwrap_or_default())
                .is_some()
        }
        Self {
            get: get::<X>,
            put: put::<X>,
        }
    }
}

/// Equality for data a mapping captures.
pub(crate) trait Same: 'static {
    fn same(&self, other: &dyn Same) -> bool;
    fn as_any(&self) -> &dyn Any;
}

impl<V: Clone + PartialEq + 'static> Same for Choices<V> {
    fn same(&self, other: &dyn Same) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| Choices::same(self, other))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// One of a mapped field's `.rules`: a rule with its own words, or a
/// [`checked`](super::checked) or [`checked_async`](super::checked_async) rule failing
/// with the field's error type, which the field's words cover.
pub enum MappedRule<T: 'static, E: 'static> {
    Worded(Validator<T>),
    Checked(Checked<T, E>),
    Async(CheckedAsync<T, E>),
}

impl<T: 'static, E: 'static> From<CheckedAsync<T, E>> for MappedRule<T, E> {
    fn from(rule: CheckedAsync<T, E>) -> Self {
        Self::Async(rule)
    }
}

impl<T: 'static, E: 'static> From<Validator<T>> for MappedRule<T, E> {
    fn from(rule: Validator<T>) -> Self {
        Self::Worded(rule)
    }
}

impl<T: 'static, E: 'static> From<Checked<T, E>> for MappedRule<T, E> {
    fn from(rule: Checked<T, E>) -> Self {
        Self::Checked(rule)
    }
}

/// No words given: `E` must be a [`FieldError`].
pub struct Unworded;

/// The words `.message(fn)` gave.
pub struct Words<E: 'static> {
    words: Rc<WordsFn<E>>,
    key: Option<TypeId>,
}

/// What words a mapped field's errors.
#[doc(hidden)]
pub trait Speak<E>: 'static {
    fn words(&self) -> Rc<WordsFn<E>>;
    fn key(&self) -> Option<TypeId>;
}

impl<E: FieldError> Speak<E> for Unworded {
    fn words(&self) -> Rc<WordsFn<E>> {
        Rc::new(E::message)
    }

    fn key(&self) -> Option<TypeId> {
        Some(TypeId::of::<()>())
    }
}

impl<E: 'static> Speak<E> for Words<E> {
    fn words(&self) -> Rc<WordsFn<E>> {
        self.words.clone()
    }

    fn key(&self) -> Option<TypeId> {
        self.key
    }
}

impl<R, T: 'static> FieldPath<R, Option<T>> {
    /// Binds a field that edits `I` (text, a date) to the domain type `T`
    /// this path holds. `parse` never sees an empty value; an error it
    /// returns shows on the field and resets the value to `None`.
    pub fn map<I: 'static, E: 'static, F, P>(self, format: F, parse: P) -> Mapped<T, I, E>
    where
        F: Fn(&T) -> I + 'static,
        P: Fn(I) -> Result<T, E> + 'static,
    {
        let key = (size_of::<F>() == 0 && size_of::<P>() == 0)
            .then(|| (TypeId::of::<F>(), TypeId::of::<P>()));
        let name = FieldName::<Option<T>>::from(self);
        let (path, steps) = name.into_parts();
        Mapped {
            path,
            steps,
            format: Rc::new(format),
            parse: Rc::new(parse),
            words: Unworded,
            rules: Vec::new(),
            key,
            slot: Slot::option(),
            fallback: None,
            data: None,
        }
    }

    /// Binds a single-pick field (`Select`, `NativeSelect`, `RadioGroup`,
    /// `SegmentedControl`) to the domain value `T` through `choices`; pass
    /// `choices.list()` as its `options`. A `SegmentedControl` shows the
    /// list's default, else its first entry, while nothing is stored.
    ///
    /// ```no_run
    /// # use dioxus::prelude::*;
    /// use libero::{choices, components::{Choices, Fields, Form, Select}};
    ///
    /// #[derive(Clone, Copy, PartialEq)]
    /// enum Country { De, At }
    ///
    /// fn country_label(country: &Country) -> String {
    ///     match country {
    ///         Country::De => "Germany",
    ///         Country::At => "Austria",
    ///     }
    ///     .into()
    /// }
    ///
    /// static COUNTRIES: Choices<Country> = choices!(Country, country_label, { De => "de", At => "at" });
    ///
    /// #[derive(Clone, PartialEq, Default, Fields)]
    /// struct Address { country: Option<Country> }
    ///
    /// # fn app() -> Element {
    /// let address = use_store(Address::default);
    /// rsx! {
    ///     Form { value: address,
    ///         Select { label: "Country", required: true,
    ///             options: COUNTRIES.list(),
    ///             name: Address::FIELDS.country().choices(&COUNTRIES) }
    ///     }
    /// }
    /// # }
    /// ```
    pub fn choices(self, choices: &Choices<T>) -> Mapped<T, Choice<T>, Infallible>
    where
        T: Clone + PartialEq,
    {
        let shown = choices.clone();
        let fallback = choices.fallback();
        let mut mapped = self.map(
            move |value: &T| shown.choice_or_unlisted(value),
            |choice: Choice<T>| Ok(choice.value),
        );
        mapped.choose(choices.clone(), fallback);
        mapped
    }

    /// [`map`](Self::map) through `T`'s [`Display`] and [`FromStr`], for a
    /// text field.
    pub fn parsed(self) -> Mapped<T, String, T::Err>
    where
        T: FromStr + Display,
        T::Err: 'static,
    {
        self.map(T::to_string, |text: String| text.parse::<T>())
    }
}

impl<R, X: Clone + PartialEq + 'static> FieldPath<R, Vec<X>> {
    /// Binds a multi-pick field (`MultiSelect`, `CheckboxGroup`) to the domain
    /// values `X` through `choices`; pass `choices.list()` as its `options`.
    /// `.rules` then run over `Vec<X>`.
    ///
    /// ```no_run
    /// # use dioxus::prelude::*;
    /// use libero::{choices, components::{CheckboxGroup, Choices, Fields, Form}};
    ///
    /// #[derive(Clone, Copy, PartialEq)]
    /// enum Topic { News, Offers }
    ///
    /// fn topic_label(topic: &Topic) -> String {
    ///     match topic {
    ///         Topic::News => "News",
    ///         Topic::Offers => "Offers",
    ///     }
    ///     .into()
    /// }
    ///
    /// static TOPICS: Choices<Topic> = choices!(Topic, topic_label, { News => "news", Offers => "offers" });
    ///
    /// #[derive(Clone, PartialEq, Default, Fields)]
    /// struct Mailing { topics: Vec<Topic> }
    ///
    /// # fn app() -> Element {
    /// let mailing = use_store(Mailing::default);
    /// rsx! {
    ///     Form { value: mailing,
    ///         CheckboxGroup { label: "Topics",
    ///             options: TOPICS.list(),
    ///             name: Mailing::FIELDS.topics().choices(&TOPICS) }
    ///     }
    /// }
    /// # }
    /// ```
    pub fn choices(self, choices: &Choices<X>) -> Mapped<Vec<X>, Vec<Choice<X>>, Infallible> {
        let shown = choices.clone();
        let (path, steps) = FieldName::<Vec<X>>::from(self).into_parts();
        let format = move |values: &Vec<X>| {
            values
                .iter()
                .map(|value| shown.choice_or_unlisted(value))
                .collect()
        };
        let parse =
            |picked: Vec<Choice<X>>| Ok(picked.into_iter().map(|choice| choice.value).collect());
        let mut mapped = Mapped {
            path,
            steps,
            format: Rc::new(format),
            parse: Rc::new(parse),
            words: Unworded,
            rules: Vec::new(),
            key: None,
            slot: Slot::list(),
            fallback: None,
            data: None,
        };
        mapped.choose(choices.clone(), None);
        mapped
    }
}

impl<T: 'static, I: 'static, E: 'static, S> Mapped<T, I, E, S> {
    /// A mapping through `choices`: equal while the list is.
    fn choose<V: Clone + PartialEq + 'static>(&mut self, choices: Choices<V>, fallback: Option<I>) {
        self.key = Some((TypeId::of::<Choices<V>>(), TypeId::of::<Choice<V>>()));
        self.data = Some(Rc::new(choices));
        self.fallback = fallback;
    }
}

impl<T: 'static, I: 'static, E: 'static> Mapped<T, I, E, Unworded> {
    /// Words every error of type `E` on this field: the parse error and
    /// `checked` rules alike. For an `E` with no [`FieldError`] of its own.
    pub fn message<W>(self, words: W) -> Mapped<T, I, E, Words<E>>
    where
        W: Fn(&E, &Localization) -> String + 'static,
    {
        Mapped {
            path: self.path,
            steps: self.steps,
            format: self.format,
            parse: self.parse,
            words: Words {
                key: (size_of::<W>() == 0).then(TypeId::of::<W>),
                words: Rc::new(words),
            },
            rules: self.rules,
            key: self.key,
            slot: self.slot,
            fallback: self.fallback,
            data: self.data,
        }
    }
}

impl<T: 'static, I: 'static, E: 'static, S> Mapped<T, I, E, S> {
    /// Rules over the domain value, run only when the text parsed; the
    /// field's own `validate` runs before, over what it edits. Adds to the
    /// rules given before, so a list of mixed kinds is two calls.
    pub fn rules<R: Into<MappedRule<T, E>>>(mut self, rules: impl IntoIterator<Item = R>) -> Self {
        self.rules.extend(rules.into_iter().map(Into::into));
        self
    }
}

impl<W, T, I, E, S> From<Mapped<T, I, E, S>> for FieldName<W>
where
    W: Wire<Inner = I>,
    T: Clone + PartialEq + 'static,
    I: 'static,
    E: 'static,
    S: Speak<E>,
{
    fn from(mapped: Mapped<T, I, E, S>) -> Self {
        erase(mapped, W::into_inner, W::from_inner, Some(W::default()))
    }
}

/// A `SegmentedControl` always shows a pick: the stored one, else the list's fallback.
impl<V, E, S> From<Mapped<V, Choice<V>, E, S>> for FieldName<Choice<V>>
where
    V: Clone + PartialEq + 'static,
    E: 'static,
    S: Speak<E>,
{
    fn from(mapped: Mapped<V, Choice<V>, E, S>) -> Self {
        let fallback = mapped.fallback.clone();
        erase(mapped, Some, |choice| choice, fallback)
    }
}

fn erase<W, T, I, E, S>(
    mapped: Mapped<T, I, E, S>,
    unwrap: fn(W) -> Option<I>,
    wrap: fn(I) -> W,
    empty: Option<W>,
) -> FieldName<W>
where
    W: Clone + 'static,
    T: Clone + PartialEq + 'static,
    I: 'static,
    E: 'static,
    S: Speak<E>,
{
    let key = mapped
        .key
        .zip(mapped.words.key())
        .map(|((format, parse), words)| [format, parse, words]);
    let words = mapped.words.words();
    let mut rules: Vec<Validator<T>> = Vec::new();
    let mut checks: Vec<AsyncRule<T>> = Vec::new();
    for rule in mapped.rules {
        match rule {
            MappedRule::Worded(rule) => rules.push(rule),
            MappedRule::Checked(rule) => rules.push(rule.worded(words.clone(), mapped.words.key())),
            MappedRule::Async(rule) => checks.push(rule.worded(words.clone(), mapped.words.key())),
        }
    }
    let adapter: Rc<dyn Adapt<W>> = Rc::new(Adapter {
        format: mapped.format,
        parse: mapped.parse,
        words,
        rules: rules.into(),
        checks: checks.into(),
        key,
        slot: mapped.slot,
        data: mapped.data,
        unwrap,
        wrap,
        empty,
    });
    FieldName::from_parts(mapped.path, mapped.steps, Some(adapter))
}

/// A mapped field's conversions, erased to what the field edits.
pub(crate) trait Adapt<W>: 'static {
    /// What the field shows for the stored value. `None` when the slot is
    /// not the mapped type.
    fn read(&self, slot: &dyn Any) -> Option<W>;
    /// Parses `wire` into the slot: the value, or `None` when empty or
    /// invalid. Also hands back a copy of what it wrote.
    fn write(&self, slot: &mut dyn Any, wire: W) -> Option<(Parse, Rc<dyn Any>)>;
    /// Whether the slot still holds what `write` handed back.
    fn holds(&self, slot: &dyn Any, wrote: &dyn Any) -> bool;
    /// `Valid` when the slot holds a value, else `Empty`.
    fn state(&self, slot: &dyn Any) -> Parse;
    /// The `.rules` over the stored value, when there is one.
    fn check(&self, slot: &dyn Any, locale: &Localization) -> Option<FieldStatus>;
    /// Whether any `.rules` are async.
    fn checks(&self) -> bool;
    /// The async `.rules`' answer for the stored value, `Pending` until it comes.
    fn check_async(
        &self,
        slot: &dyn Any,
        checking: &Rc<Checking>,
        locale: &'static Localization,
    ) -> Option<FieldStatus>;
    fn same(&self, other: &dyn Any) -> bool;
    fn as_any(&self) -> &dyn Any;
}

struct Adapter<W, T: 'static, I: 'static, E: 'static> {
    format: Rc<dyn Fn(&T) -> I>,
    parse: Rc<dyn Fn(I) -> Result<T, E>>,
    words: Rc<WordsFn<E>>,
    rules: Validators<T>,
    checks: Rc<[AsyncRule<T>]>,
    key: Option<[TypeId; 3]>,
    slot: Slot<T>,
    data: Option<Rc<dyn Same>>,
    /// What the field edits, to `parse`'s input: `None` when empty.
    unwrap: fn(W) -> Option<I>,
    wrap: fn(I) -> W,
    /// What the field shows while nothing is stored.
    empty: Option<W>,
}

impl<W, T, I, E> Adapt<W> for Adapter<W, T, I, E>
where
    W: Clone + 'static,
    T: Clone + PartialEq + 'static,
    I: 'static,
    E: 'static,
{
    fn read(&self, slot: &dyn Any) -> Option<W> {
        match (self.slot.get)(slot)? {
            Some(value) => Some((self.wrap)((self.format)(value))),
            None => self.empty.clone(),
        }
    }

    fn write(&self, slot: &mut dyn Any, wire: W) -> Option<(Parse, Rc<dyn Any>)> {
        let (next, parse) = match (self.unwrap)(wire).map(|inner| (self.parse)(inner)) {
            None => (None, Parse::Empty),
            Some(Ok(value)) => (Some(value), Parse::Valid),
            Some(Err(error)) => {
                let error: Rc<dyn FieldError> = Rc::new(Worded {
                    error,
                    words: self.words.clone(),
                });
                (None, Parse::Invalid(error))
            }
        };
        if !(self.slot.put)(slot, next.clone()) {
            return None;
        }
        Some((parse, Rc::new(next)))
    }

    fn holds(&self, slot: &dyn Any, wrote: &dyn Any) -> bool {
        (self.slot.get)(slot)
            .zip(wrote.downcast_ref::<Option<T>>())
            .is_some_and(|(slot, wrote)| slot == wrote.as_ref())
    }

    fn state(&self, slot: &dyn Any) -> Parse {
        match (self.slot.get)(slot) {
            Some(Some(_)) => Parse::Valid,
            _ => Parse::Empty,
        }
    }

    fn check(&self, slot: &dyn Any, locale: &Localization) -> Option<FieldStatus> {
        let value = (self.slot.get)(slot)??;
        if self.rules.is_empty() {
            return None;
        }
        let mut warning = None;
        for rule in self.rules.iter() {
            let Some(error) = rule.failure(value) else {
                continue;
            };
            match rule.worded(error.message(locale)) {
                error @ FieldStatus::Error(_) => return Some(error),
                found => {
                    warning.get_or_insert(found);
                }
            }
        }
        Some(warning.unwrap_or_default())
    }

    fn checks(&self) -> bool {
        !self.checks.is_empty()
    }

    fn check_async(
        &self,
        slot: &dyn Any,
        checking: &Rc<Checking>,
        locale: &'static Localization,
    ) -> Option<FieldStatus> {
        let value = (self.slot.get)(slot)??;
        Some(checking.check(value, &self.checks, locale))
    }

    /// By the types of `format`, `parse` and the words when none captures
    /// anything, and by the rules: a `.map(..)` rebuilt per render is equal.
    fn same(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>().is_some_and(|other| {
            self.key.is_some()
                && self.key == other.key
                && self.rules == other.rules
                && self.checks == other.checks
                && match (&self.data, &other.data) {
                    (None, None) => true,
                    (Some(a), Some(b)) => a.same(&**b),
                    _ => false,
                }
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
