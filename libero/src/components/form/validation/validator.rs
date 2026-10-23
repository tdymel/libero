use std::any::TypeId;
use std::rc::Rc;

use super::catalog::IsEmpty;
use crate::{
    components::form::{FieldError, FieldPath, FieldStatus},
    localization::Localization,
};

/// What a rule's check yields: nothing, or the error to word.
pub(crate) type Check<V> = dyn Fn(&V) -> Result<(), Rc<dyn FieldError>>;

/// A rule plus what to show when it fails. Built from any `Fn(&V) -> bool`
/// through [`Rule::error`] or [`Rule::warn`], or from a [`checked`](super::checked) rule.
pub struct Validator<V: 'static> {
    check: Rc<Check<V>>,
    /// The rule's type, and its `.message` words' type, when they capture
    /// nothing: two such rules of one type behave the same, so they compare equal.
    stateless: Option<(TypeId, TypeId)>,
    /// A `bool` rule's fixed status; a `checked` rule's is blank, by kind.
    status: FieldStatus,
    on: Vec<String>,
}

impl<V: 'static> Validator<V> {
    /// `Valid` when the rule holds, else this validator's warning or error,
    /// worded for the active language.
    pub fn validate(&self, value: &V) -> FieldStatus {
        match self.failure(value) {
            None => FieldStatus::Valid,
            Some(error) => self.worded(error.message(locale())),
        }
    }

    /// A rule from its check: what [`checked`](super::checked) builds.
    pub(crate) fn from_check(
        check: Rc<Check<V>>,
        stateless: Option<(TypeId, TypeId)>,
        warning: bool,
    ) -> Self {
        Self {
            check,
            stateless,
            status: match warning {
                true => FieldStatus::Warning(String::new()),
                false => FieldStatus::Error(String::new()),
            },
            on: Vec::new(),
        }
    }

    /// The error the rule failed with, unworded.
    pub(crate) fn failure(&self, value: &V) -> Option<Rc<dyn FieldError>> {
        (self.check)(value).err()
    }

    /// `message` as this rule's kind: a warning or an error.
    pub(crate) fn worded(&self, message: String) -> FieldStatus {
        match self.status {
            FieldStatus::Warning(_) => FieldStatus::Warning(message),
            _ => FieldStatus::Error(message),
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

/// The active language, or English outside a running app (a unit test).
pub(crate) fn locale() -> &'static Localization {
    use dioxus::prelude::*;
    let in_scope = dioxus::core::Runtime::try_current()
        .is_some_and(|runtime| runtime.try_current_scope_id().is_some());
    in_scope
        .then(try_consume_context::<crate::context::LiberoContext>)
        .flatten()
        .map_or(&Localization::ENGLISH, |context| {
            *context.localization.read()
        })
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

    /// Holds for an empty value, so an optional field checks only what was typed.
    /// Opt-in: `not_empty` and every other rule still see the empty value.
    ///
    /// ```no_run
    /// use libero::components::{Rule, Validators, is_email};
    ///
    /// // An optional email: blank passes, `tom@` does not.
    /// let rules: Validators<String> = is_email.when_present().error("Not an email").into();
    /// # let _ = rules;
    /// ```
    fn when_present(self) -> impl Rule<V>
    where
        V: IsEmpty,
    {
        move |value: &V| value.is_empty_value() || self(value)
    }
}

impl<V: 'static, F: Fn(&V) -> bool + 'static> Rule<V> for F {}

fn validator<V: 'static, F: Fn(&V) -> bool + 'static>(
    rule: F,
    status: FieldStatus,
) -> Validator<V> {
    let text: Rc<dyn FieldError> = Rc::new(status.message().unwrap_or_default().to_string());
    Validator {
        check: Rc::new(move |value: &V| match rule(value) {
            true => Ok(()),
            false => Err(text.clone()),
        }),
        stateless: (size_of::<F>() == 0).then(|| (TypeId::of::<F>(), TypeId::of::<()>())),
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

    /// Adds `rule` after the ones already held, so a wrapper component can add its
    /// own rule to the caller's.
    ///
    /// ```no_run
    /// # use dioxus::prelude::*;
    /// use libero::components::{Rule, TextField, Validators, max_length};
    ///
    /// #[component]
    /// fn ShortNote(value: String, validate: Validators<String>) -> Element {
    ///     rsx! {
    ///         TextField {
    ///             label: "Note",
    ///             value,
    ///             validate: validate.with(max_length(80).error("At most 80 characters")),
    ///         }
    ///     }
    /// }
    /// ```
    pub fn with(mut self, rule: Validator<V>) -> Self {
        self.0.push(rule);
        self
    }

    /// Adds every rule in `rules`, in order, after the ones already held.
    ///
    /// ```no_run
    /// use libero::components::{Rule, Validators, is_email, not_empty};
    ///
    /// let base: Validators<String> = not_empty.error("Required").into();
    /// let email = base.extend([is_email.error("Not an email")]);
    /// # let _ = email;
    /// ```
    pub fn extend(mut self, rules: impl IntoIterator<Item = Validator<V>>) -> Self {
        self.0.extend(rules);
        self
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

    /// A wrapper's added rule keeps the stateless match, so its field still memoizes.
    #[test]
    fn with_and_extend_append_and_keep_stateless_equality() {
        let base = || -> Validators<String> { not_admin.error("Taken").into() };
        let short = || (|v: &String| v.len() > 2).warn("Short");

        let with = base().with(short());
        let extended = Validators::default().extend([not_admin.error("Taken"), short()]);
        assert!(with == base().with(short()));
        assert!(with == extended);
        assert!(with != base());
        assert_eq!(
            with.validate(&"ab".into()),
            FieldStatus::Warning("Short".into())
        );

        let min = 3;
        let capturing = base().with((move |v: &String| v.len() > min).error("Short"));
        assert!(capturing != capturing.clone());
    }

    #[test]
    fn when_present_passes_empty_and_checks_the_rest() {
        let rules: Validators<String> = (|v: &String| v.contains('@'))
            .when_present()
            .error("Not an email")
            .into();

        assert!(rules.validate(&"".into()).is_valid());
        assert!(rules.validate(&"  ".into()).is_valid());
        assert!(rules.validate(&"tom@libero".into()).is_valid());
        assert!(rules.validate(&"tom".into()).is_error());
        assert!(
            crate::components::form::not_empty
                .error("Required")
                .validate(&String::new())
                .is_error()
        );
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
