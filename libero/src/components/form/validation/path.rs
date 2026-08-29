use std::{borrow::Cow, fmt, marker::PhantomData};

use dioxus::dioxus_core::{AttributeValue, IntoAttributeValue};

/// A field's place inside a value of type `Root`, holding a `T`. Doubles as
/// the field's `name`, so what posts and what a composite rule names are one
/// string - `address.zip` for a nested field.
///
/// Built by `#[derive(Fields)]` (`Signup::FIELDS.address().zip()`) or by
/// [`path!`](crate::path) for a type that cannot derive.
pub struct FieldPath<Root, T> {
    path: Cow<'static, str>,
    _types: PhantomData<fn(&Root) -> &T>,
}

impl<Root, T> FieldPath<Root, T> {
    pub const fn new(path: &'static str) -> Self {
        Self {
            path: Cow::Borrowed(path),
            _types: PhantomData,
        }
    }

    pub fn owned(path: String) -> Self {
        Self {
            path: Cow::Owned(path),
            _types: PhantomData,
        }
    }

    /// `name` under `prefix`. What `#[derive(Fields)]` emits for each field.
    #[doc(hidden)]
    pub fn at(prefix: &str, name: &str) -> Self {
        Self::owned(join(prefix, name))
    }

    pub fn as_str(&self) -> &str {
        &self.path
    }

    /// A path relative to `T`, re-rooted under this one: `address` + `zip`.
    pub fn join<U>(&self, inner: FieldPath<T, U>) -> FieldPath<Root, U> {
        FieldPath::owned(join(&self.path, &inner.path))
    }
}

pub(crate) fn join(prefix: &str, path: &str) -> String {
    match (prefix.is_empty(), path.is_empty()) {
        (true, _) => path.to_string(),
        (_, true) => prefix.to_string(),
        _ => format!("{prefix}.{path}"),
    }
}

impl<Root, T> Clone for FieldPath<Root, T> {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            _types: PhantomData,
        }
    }
}

impl<Root, T> PartialEq for FieldPath<Root, T> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl<Root, T> fmt::Debug for FieldPath<Root, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "FieldPath({})", self.path)
    }
}

impl<Root, T> fmt::Display for FieldPath<Root, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.path)
    }
}

impl<Root, T> From<FieldPath<Root, T>> for String {
    fn from(path: FieldPath<Root, T>) -> Self {
        path.path.into_owned()
    }
}

/// So `name: Signup::FIELDS.email()` works on a field whose props extend
/// `input`, where `name` is a plain attribute.
impl<Root, T> IntoAttributeValue for FieldPath<Root, T> {
    fn into_value(self) -> AttributeValue {
        AttributeValue::Text(self.path.into_owned())
    }
}

/// Implemented by `#[derive(Fields)]`: the typed paths of a struct's fields,
/// rooted at `R`.
pub trait Fields: Sized {
    type Paths<R>;

    fn paths<R>(prefix: String) -> Self::Paths<R>;
}

/// A typed path for a type that cannot `#[derive(Fields)]`. The field access
/// is compiled, so a typo is an error:
///
/// ```ignore
/// let zip = path!(Signup => address.zip); // FieldPath<Signup, String>
/// ```
#[macro_export]
macro_rules! path {
    ($root:ty => $first:ident $(. $rest:ident)*) => {{
        fn typed<R, T>(_: fn(&R) -> &T, path: &'static str) -> $crate::components::FieldPath<R, T> {
            $crate::components::FieldPath::new(path)
        }
        typed(
            |root: &$root| &root.$first $(.$rest)*,
            concat!(stringify!($first) $(, ".", stringify!($rest))*),
        )
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Address {
        zip: String,
    }
    struct Signup {
        address: Address,
        email: String,
    }

    #[test]
    fn the_macro_builds_dotted_paths_and_knows_the_type() {
        let zip: FieldPath<Signup, String> = crate::path!(Signup => address.zip);
        let email = crate::path!(Signup => email);
        assert_eq!(zip.as_str(), "address.zip");
        assert_eq!(email.as_str(), "email");

        // Only there to read the fields, which the macro's closure also does.
        let signup = Signup {
            address: Address { zip: "1".into() },
            email: "e".into(),
        };
        assert_eq!(signup.address.zip.len() + signup.email.len(), 2);
    }

    #[test]
    fn join_reroots_a_relative_path() {
        let address = crate::path!(Signup => address);
        let zip = crate::path!(Address => zip);
        assert_eq!(address.join(zip).as_str(), "address.zip");
    }
}
