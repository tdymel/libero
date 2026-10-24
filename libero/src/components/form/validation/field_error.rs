use std::{fmt, rc::Rc};

use crate::localization::Localization;

/// An error a field shows, worded for the active language. Kept as the error,
/// not as text, so a language switch re-words it without parsing again.
///
/// ```no_run
/// use libero::{components::FieldError, localization::Localization};
///
/// enum Taken { Email }
/// impl FieldError for Taken {
///     fn message(&self, locale: &Localization) -> String {
///         match locale.lang {
///             "de" => "Diese Adresse ist vergeben.".into(),
///             _ => "This address is taken.".into(),
///         }
///     }
/// }
/// ```
///
/// A domain error from a crate without libero gets its words through the
/// field's `.message(fn)` instead. There is no fallback to `Display`.
pub trait FieldError: 'static {
    fn message(&self, locale: &Localization) -> String;
}

/// Words picked per language.
impl<F: Fn(&Localization) -> String + 'static> FieldError for F {
    fn message(&self, locale: &Localization) -> String {
        self(locale)
    }
}

/// Text in one language, which does not follow a switch.
impl FieldError for String {
    fn message(&self, _: &Localization) -> String {
        self.clone()
    }
}

/// Text in one language, which does not follow a switch.
impl FieldError for &'static str {
    fn message(&self, _: &Localization) -> String {
        (*self).to_string()
    }
}

/// The error of a mapping that can't fail, such as `.choices(..)`.
impl FieldError for std::convert::Infallible {
    fn message(&self, _: &Localization) -> String {
        match *self {}
    }
}

/// What a `.message(fn)` holds: an error's words in a language.
pub(crate) type WordsFn<E> = dyn Fn(&E, &Localization) -> String;

/// An error worded by a `.message(fn)` given apart from it.
pub(crate) struct Worded<E> {
    pub error: E,
    pub words: Rc<WordsFn<E>>,
}

impl<E: 'static> FieldError for Worded<E> {
    fn message(&self, locale: &Localization) -> String {
        (self.words)(&self.error, locale)
    }
}

/// `error` in the active language: where every kept error is worded, at render.
pub(crate) fn word(error: &dyn FieldError) -> String {
    error.message(super::validator::locale())
}

/// Where a mapped field's text stands: nothing, not a domain value, or one.
#[derive(Clone, Default)]
pub(crate) enum Parse {
    /// Blank text, or no value; the store holds `None`.
    #[default]
    Empty,
    /// The store was reset to `None`; the text stays on screen.
    Invalid(Rc<dyn FieldError>),
    Valid,
}

impl Parse {
    pub fn is_invalid(&self) -> bool {
        matches!(self, Self::Invalid(_))
    }

    /// Both empty, both valid or both invalid, whatever the error.
    pub fn same_kind(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl PartialEq for Parse {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Empty, Self::Empty) | (Self::Valid, Self::Valid) => true,
            (Self::Invalid(a), Self::Invalid(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl fmt::Debug for Parse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "Empty",
            Self::Invalid(_) => "Invalid",
            Self::Valid => "Valid",
        })
    }
}
