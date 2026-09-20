use std::any::TypeId;
use std::rc::Rc;

use crate::components::form::{FieldPath, FieldStatus};

/// A rule plus what to show when it fails. Built from any `Fn(&V) -> bool`
/// through [`Rule::error`] or [`Rule::warn`].
pub struct Validator<V: 'static> {
    check: Rc<dyn Fn(&V) -> bool>,
    /// The rule's type when it captures nothing: two such rules of one type
    /// behave the same, so they compare equal.
    stateless: Option<TypeId>,
    status: FieldStatus,
    on: Vec<String>,
}

impl<V: 'static> Validator<V> {
    /// `Valid` when the rule holds, else this validator's warning or error.
    pub fn validate(&self, value: &V) -> FieldStatus {
        match (self.check)(value) {
            true => FieldStatus::Valid,
            false => self.status.clone(),
        }
    }

    /// The fields a composite rule concerns. Its status shows on each of them.
    pub fn on<const N: usize, T>(mut self, paths: [FieldPath<V, T>; N]) -> Self {
        self.on
            .extend(paths.into_iter().map(|path| path.as_str().to_string()));
        self
    }

    /// The field paths named through [`Validator::on`].
    pub fn targets(&self) -> &[String] {
        &self.on
    }
}

impl<V: 'static> Clone for Validator<V> {
    fn clone(&self) -> Self {
        Self {
            check: self.check.clone(),
            stateless: self.stateless,
            status: self.status.clone(),
            on: self.on.clone(),
        }
    }
}

/// Any `Fn(&V) -> bool` is a rule: a plain `fn`, a catalog predicate or a
/// closure.
pub trait Rule<V: 'static>: Fn(&V) -> bool + Sized + 'static {
    fn error(self, message: impl Into<String>) -> Validator<V> {
        validator(self, FieldStatus::Error(message.into()))
    }

    fn warn(self, message: impl Into<String>) -> Validator<V> {
        validator(self, FieldStatus::Warning(message.into()))
    }

    /// Holds when both rules hold.
    fn and(self, other: impl Rule<V>) -> impl Rule<V> {
        move |value: &V| self(value) && other(value)
    }

    /// Holds when either rule holds.
    fn or(self, other: impl Rule<V>) -> impl Rule<V> {
        move |value: &V| self(value) || other(value)
    }
}

impl<V: 'static, F: Fn(&V) -> bool + 'static> Rule<V> for F {}

fn validator<V: 'static, F: Fn(&V) -> bool + 'static>(
    rule: F,
    status: FieldStatus,
) -> Validator<V> {
    Validator {
        check: Rc::new(rule),
        stateless: (size_of::<F>() == 0).then(TypeId::of::<F>),
        status,
        on: Vec::new(),
    }
}

/// What a `validate` prop holds.
pub struct Validators<V: 'static>(Vec<Validator<V>>);

impl<V: 'static> Validators<V> {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Validator<V>> {
        self.0.iter()
    }

    /// What a field hands `use_field`: `None` without rules, so a field with
    /// none tracks nothing.
    pub(crate) fn check(&self, value: &V) -> Option<FieldStatus> {
        (!self.0.is_empty()).then(|| self.validate(value))
    }

    /// The first error, else the first warning, else `Valid`.
    pub fn validate(&self, value: &V) -> FieldStatus {
        let mut warning = None;
        for validator in &self.0 {
            match validator.validate(value) {
                FieldStatus::Valid => {}
                error @ FieldStatus::Error(_) => return error,
                found @ FieldStatus::Warning(_) => {
                    warning.get_or_insert(found);
                }
            }
        }
        warning.unwrap_or_default()
    }
}

impl<V: 'static> Default for Validators<V> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<V: 'static> Clone for Validators<V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// Equal when every rule captures nothing and matches by type, message and
/// targets, so a field with `not_empty` memoizes. A capturing closure never is.
impl<V: 'static> PartialEq for Validators<V> {
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len()
            && self.0.iter().zip(&other.0).all(|(a, b)| {
                a.stateless.is_some()
                    && a.stateless == b.stateless
                    && a.status == b.status
                    && a.on == b.on
            })
    }
}

impl<V: 'static, const N: usize> From<[Validator<V>; N]> for Validators<V> {
    fn from(validators: [Validator<V>; N]) -> Self {
        Self(validators.into())
    }
}

impl<V: 'static> From<Vec<Validator<V>>> for Validators<V> {
    fn from(validators: Vec<Validator<V>>) -> Self {
        Self(validators)
    }
}

impl<V: 'static> From<Validator<V>> for Validators<V> {
    fn from(validator: Validator<V>) -> Self {
        Self(vec![validator])
    }
}

/// Error beats warning beats valid; a tie keeps `first`.
pub(crate) fn worst(first: FieldStatus, second: FieldStatus) -> FieldStatus {
    let rank = |status: &FieldStatus| match status {
        FieldStatus::Valid => 0,
        FieldStatus::Warning(_) => 1,
        FieldStatus::Error(_) => 2,
    };
    match rank(&second) > rank(&first) {
        true => second,
        false => first,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn not_admin(name: &String) -> bool {
        name != "admin"
    }

    #[test]
    fn a_plain_fn_and_a_closure_are_both_rules() {
        let by_fn = not_admin.error("Taken");
        let by_closure = (|name: &String| name.len() > 2).warn("Short");

        assert_eq!(
            by_fn.validate(&"admin".into()),
            FieldStatus::Error("Taken".into())
        );
        assert_eq!(
            by_closure.validate(&"ab".into()),
            FieldStatus::Warning("Short".into())
        );
        assert!(by_fn.validate(&"tom".into()).is_valid());
    }

    #[test]
    fn and_or_compose_before_the_message() {
        let both = (|n: &i32| *n > 0).and(|n: &i32| *n < 10).error("1 to 9");
        let either = (|n: &i32| *n == 0).or(|n: &i32| *n > 100).error("0 or big");

        assert!(both.validate(&5).is_valid());
        assert!(both.validate(&10).is_error());
        assert!(either.validate(&0).is_valid());
        assert!(either.validate(&50).is_error());
    }

    #[test]
    fn the_first_error_wins_and_a_warning_only_without_one() {
        let rules: Validators<String> = [
            (|v: &String| v.len() > 5).warn("Short"),
            (|v: &String| !v.is_empty()).error("Required"),
            (|v: &String| v.contains('@')).error("Not an email"),
        ]
        .into();

        assert_eq!(
            rules.validate(&"".into()),
            FieldStatus::Error("Required".into())
        );
        assert_eq!(
            rules.validate(&"a@b".into()),
            FieldStatus::Warning("Short".into())
        );
        assert!(rules.validate(&"tom@libero".into()).is_valid());
    }

    #[test]
    fn only_rules_that_capture_nothing_compare_equal() {
        let rules = || -> Validators<String> { [not_admin.error("Taken")].into() };
        let min = 3;
        let capturing: Validators<String> =
            [(move |v: &String| v.len() > min).error("Short")].into();

        assert!(Validators::<String>::default() == Validators::default());
        assert!(rules() == rules());
        assert!(rules() != [not_admin.error("Other")].into());
        assert!(rules() != [(|v: &String| v.is_empty()).error("Taken")].into());
        assert!(capturing != capturing.clone());
    }

    #[test]
    fn worst_ranks_error_over_warning_and_keeps_the_first_on_a_tie() {
        let warning = FieldStatus::Warning("w".into());
        let error = FieldStatus::Error("e".into());

        assert_eq!(worst(warning.clone(), error.clone()), error);
        assert_eq!(worst(FieldStatus::Valid, warning.clone()), warning);
        assert_eq!(
            worst(FieldStatus::Error("mine".into()), error),
            FieldStatus::Error("mine".into())
        );
    }
}
