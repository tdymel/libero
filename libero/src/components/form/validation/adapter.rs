use std::{
    any::Any, any::TypeId, borrow::Cow, fmt::Display, marker::PhantomData, rc::Rc, str::FromStr,
};

use super::{
    checked::Checked,
    field_error::{FieldError, Parse, Worded, WordsFn},
    path::{FieldName, FieldPath, Step},
    validator::Validator,
};
use crate::{
    components::form::{FieldStatus, Validators},
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
}

/// One of a mapped field's `.rules`: a rule with its own words, or a
/// [`checked`](super::checked) rule failing with the field's error type,
/// which the field's words cover.
pub enum MappedRule<T: 'static, E: 'static> {
    Worded(Validator<T>),
    Checked(Checked<T, E>),
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
        }
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
        let key = mapped
            .key
            .zip(mapped.words.key())
            .map(|((format, parse), words)| [format, parse, words]);
        let words = mapped.words.words();
        let rules: Vec<Validator<T>> = mapped
            .rules
            .into_iter()
            .map(|rule| match rule {
                MappedRule::Worded(rule) => rule,
                MappedRule::Checked(rule) => rule.worded(words.clone(), mapped.words.key()),
            })
            .collect();
        let adapter: Rc<dyn Adapt<W>> = Rc::new(Adapter {
            format: mapped.format,
            parse: mapped.parse,
            words,
            rules: rules.into(),
            key,
            _wire: PhantomData::<fn() -> W>,
        });
        FieldName::from_parts(mapped.path, mapped.steps, Some(adapter))
    }
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
    fn same(&self, other: &dyn Any) -> bool;
    fn as_any(&self) -> &dyn Any;
}

struct Adapter<W, T: 'static, I: 'static, E: 'static> {
    format: Rc<dyn Fn(&T) -> I>,
    parse: Rc<dyn Fn(I) -> Result<T, E>>,
    words: Rc<WordsFn<E>>,
    rules: Validators<T>,
    key: Option<[TypeId; 3]>,
    _wire: PhantomData<fn() -> W>,
}

impl<W, T, I, E> Adapt<W> for Adapter<W, T, I, E>
where
    W: Wire<Inner = I>,
    T: Clone + PartialEq + 'static,
    I: 'static,
    E: 'static,
{
    fn read(&self, slot: &dyn Any) -> Option<W> {
        let value = slot.downcast_ref::<Option<T>>()?;
        Some(
            value
                .as_ref()
                .map_or_else(W::default, |value| W::from_inner((self.format)(value))),
        )
    }

    fn write(&self, slot: &mut dyn Any, wire: W) -> Option<(Parse, Rc<dyn Any>)> {
        let slot = slot.downcast_mut::<Option<T>>()?;
        let (next, parse) = match wire.into_inner().map(|inner| (self.parse)(inner)) {
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
        *slot = next.clone();
        Some((parse, Rc::new(next)))
    }

    fn holds(&self, slot: &dyn Any, wrote: &dyn Any) -> bool {
        slot.downcast_ref::<Option<T>>()
            .zip(wrote.downcast_ref::<Option<T>>())
            .is_some_and(|(slot, wrote)| slot == wrote)
    }

    fn state(&self, slot: &dyn Any) -> Parse {
        match slot.downcast_ref::<Option<T>>() {
            Some(Some(_)) => Parse::Valid,
            _ => Parse::Empty,
        }
    }

    fn check(&self, slot: &dyn Any, locale: &Localization) -> Option<FieldStatus> {
        let value = slot.downcast_ref::<Option<T>>()?.as_ref()?;
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

    /// By the types of `format`, `parse` and the words when none captures
    /// anything, and by the rules: a `.map(..)` rebuilt per render is equal.
    fn same(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>().is_some_and(|other| {
            self.key.is_some() && self.key == other.key && self.rules == other.rules
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
