//! Predicates every form needs. Each is a plain [`Rule`](super::Rule), so it
//! takes its message the same way a caller's own `fn` does:
//! `min_length(3).error("At least 3 characters")`.

/// A value that can be empty, for [`not_empty`].
pub trait IsEmpty {
    fn is_empty_value(&self) -> bool;
}

impl IsEmpty for String {
    /// Whitespace alone counts as empty.
    fn is_empty_value(&self) -> bool {
        self.trim().is_empty()
    }
}

impl<T> IsEmpty for Option<T> {
    fn is_empty_value(&self) -> bool {
        self.is_none()
    }
}

impl<T> IsEmpty for Vec<T> {
    fn is_empty_value(&self) -> bool {
        self.is_empty()
    }
}

impl IsEmpty for bool {
    /// `false` is empty, so a required checkbox reads the same as a required
    /// text field.
    fn is_empty_value(&self) -> bool {
        !self
    }
}

pub fn not_empty<V: IsEmpty>(value: &V) -> bool {
    !value.is_empty_value()
}

/// At least `length` characters, not bytes.
pub fn min_length(length: usize) -> impl Fn(&String) -> bool {
    move |value| value.chars().count() >= length
}

/// At most `length` characters, not bytes.
pub fn max_length(length: usize) -> impl Fn(&String) -> bool {
    move |value| value.chars().count() <= length
}

pub fn min<V: PartialOrd + 'static>(bound: V) -> impl Fn(&V) -> bool {
    move |value| *value >= bound
}

pub fn max<V: PartialOrd + 'static>(bound: V) -> impl Fn(&V) -> bool {
    move |value| *value <= bound
}

/// One `@` with something before it and a dotted domain after. A shape check,
/// not delivery - the only proof an address works is mail arriving.
// `&String`, not `&str`: a rule over a `String` field is `Fn(&String) -> bool`.
#[allow(clippy::ptr_arg)]
pub fn is_email(value: &String) -> bool {
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !domain.contains('@')
        && !value.chars().any(char::is_whitespace)
        && domain.split_once('.').is_some_and(|(host, rest)| {
            !host.is_empty() && !rest.is_empty() && !rest.ends_with('.')
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::form::{FieldStatus, Rule};

    #[test]
    fn not_empty_covers_text_options_lists_and_checkboxes() {
        assert!(!not_empty(&"   ".to_string()));
        assert!(not_empty(&"a".to_string()));
        assert!(!not_empty(&None::<u8>));
        assert!(not_empty(&vec![1]));
        assert!(!not_empty(&false));
    }

    #[test]
    fn lengths_count_characters() {
        assert!(min_length(2)(&"äö".to_string()));
        assert!(!max_length(1)(&"äö".to_string()));
    }

    #[test]
    fn bounds_are_inclusive() {
        assert!(min(18)(&18));
        assert!(!max(1.5)(&1.6));
    }

    #[test]
    fn email_is_a_shape_check() {
        for good in ["tom@libero.dev", "a.b+c@mail.example.org"] {
            assert!(is_email(&good.to_string()), "{good}");
        }
        for bad in [
            "",
            "tom",
            "@libero.dev",
            "tom@",
            "tom@libero",
            "tom@@libero.dev",
            "t om@x.de",
            "tom@x.",
        ] {
            assert!(!is_email(&bad.to_string()), "{bad}");
        }
    }

    #[test]
    fn a_catalog_predicate_takes_a_message_like_any_rule() {
        let rule = min_length(3).and(is_email).error("Bad");
        assert_eq!(rule.validate(&"x".into()), FieldStatus::Error("Bad".into()));
    }
}
