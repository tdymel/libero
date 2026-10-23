use std::{any::TypeId, rc::Rc};

use super::{
    field_error::{FieldError, Worded, WordsFn},
    validator::{Validator, Validators},
};
use crate::localization::Localization;

/// A rule that says why it fails, built by [`checked`]. Its error `E` still
/// needs words: its own [`FieldError`], [`message`](Self::message), or the
/// mapped field's `.message` for the same `E`.
pub struct Checked<V: 'static, E: 'static> {
    rule: Rc<CheckFn<V, E>>,
    /// The rule's type when it captures nothing.
    key: Option<TypeId>,
    warning: bool,
}

type CheckFn<V, E> = dyn Fn(&V) -> Result<(), E>;

/// A rule that returns the domain's own error, and the field shows that
/// error's words. For a check on a value that already parsed:
///
/// ```no_run
/// # use libero::components::{checked, FieldError, Validators};
/// # use libero::localization::Localization;
/// enum Common { Password }
/// impl FieldError for Common {
///     fn message(&self, _: &Localization) -> String { "This password is common.".into() }
/// }
/// fn not_common(text: &String) -> Result<(), Common> {
///     match text.as_str() { "password1" => Err(Common::Password), _ => Ok(()) }
/// }
/// let rules: Validators<String> = [checked(not_common).warning()].into();
/// ```
///
/// An error with no [`FieldError`], a domain crate's, takes
/// [`message`](Checked::message), or the mapped field's `.message` when it is
/// that field's error type. Without either it does not compile.
pub fn checked<V: 'static, E: 'static, F>(rule: F) -> Checked<V, E>
where
    F: Fn(&V) -> Result<(), E> + 'static,
{
    Checked {
        key: (size_of::<F>() == 0).then(TypeId::of::<F>),
        rule: Rc::new(rule),
        warning: false,
    }
}

impl<V: 'static, E: 'static> Checked<V, E> {
    /// Shows the error as a warning, which never blocks a submit.
    pub fn warning(mut self) -> Self {
        self.warning = true;
        self
    }

    /// Words the error, for an `E` with no [`FieldError`] of its own.
    pub fn message<W>(self, words: W) -> Validator<V>
    where
        W: Fn(&E, &Localization) -> String + 'static,
    {
        let key = (size_of::<W>() == 0).then(TypeId::of::<W>);
        self.worded(Rc::new(words), key)
    }

    /// The rule with its words; equal to another by both types when neither captures.
    pub(crate) fn worded(self, words: Rc<WordsFn<E>>, words_key: Option<TypeId>) -> Validator<V> {
        let rule = self.rule;
        let check = Rc::new(move |value: &V| {
            rule(value).map_err(|error| {
                Rc::new(Worded {
                    error,
                    words: words.clone(),
                }) as Rc<dyn FieldError>
            })
        });
        Validator::from_check(check, self.key.zip(words_key), self.warning)
    }
}

impl<V: 'static, E: FieldError> From<Checked<V, E>> for Validator<V> {
    fn from(checked: Checked<V, E>) -> Self {
        checked.worded(Rc::new(E::message), Some(TypeId::of::<()>()))
    }
}

impl<V: 'static, E: FieldError> From<Checked<V, E>> for Validators<V> {
    fn from(checked: Checked<V, E>) -> Self {
        Validator::from(checked).into()
    }
}

impl<V: 'static, E: FieldError, const N: usize> From<[Checked<V, E>; N]> for Validators<V> {
    fn from(rules: [Checked<V, E>; N]) -> Self {
        rules
            .into_iter()
            .map(Validator::from)
            .collect::<Vec<_>>()
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::form::FieldStatus;

    enum Common {
        Password,
    }

    impl FieldError for Common {
        fn message(&self, _: &Localization) -> String {
            "Common".into()
        }
    }

    #[allow(clippy::ptr_arg)] // A rule over a `String` takes `&String`.
    fn not_common(text: &String) -> Result<(), Common> {
        match text.as_str() {
            "password1" => Err(Common::Password),
            _ => Ok(()),
        }
    }

    struct TooLong;

    #[allow(clippy::ptr_arg)] // A rule over a `String` takes `&String`.
    fn short(text: &String) -> Result<(), TooLong> {
        match text.len() > 3 {
            true => Err(TooLong),
            false => Ok(()),
        }
    }

    fn too_long(_: &TooLong, _: &Localization) -> String {
        "Too long".into()
    }

    #[test]
    fn a_checked_rule_shows_its_errors_words_as_an_error_or_a_warning() {
        let error: Validators<String> = [checked(not_common)].into();
        let warning: Validators<String> = [checked(not_common).warning()].into();
        let foreign: Validators<String> = [checked(short).message(too_long)].into();

        assert_eq!(
            error.validate(&"password1".into()),
            FieldStatus::Error("Common".into())
        );
        assert_eq!(
            warning.validate(&"password1".into()),
            FieldStatus::Warning("Common".into())
        );
        assert!(warning.validate(&"x7#Lq".into()).is_valid());
        assert_eq!(
            foreign.validate(&"abcd".into()),
            FieldStatus::Error("Too long".into())
        );
    }

    #[test]
    fn checked_rules_compare_by_their_types() {
        let rules = || -> Validators<String> { [checked(not_common)].into() };
        let worded = || -> Validators<String> { [checked(short).message(too_long)].into() };

        assert!(rules() == rules());
        assert!(worded() == worded());
        assert!(rules() != [checked(not_common).warning()].into());
        let other: Validators<String> =
            [checked(short).message(|_: &TooLong, _: &Localization| "Other".to_string())].into();
        assert!(worded() != other);
    }
}
