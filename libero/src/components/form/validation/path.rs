use std::{any::Any, borrow::Cow, fmt, marker::PhantomData};

use dioxus::dioxus_core::{AttributeValue, IntoAttributeValue};

/// One type-erased field access, emitted by `#[derive(Fields)]` and [`path!`](crate::path).
#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct Step {
    pub get: fn(&dyn Any) -> Option<&dyn Any>,
    pub get_mut: fn(&mut dyn Any) -> Option<&mut dyn Any>,
    /// Where the step leads in the form store's subscriptions.
    pub key: StepKey,
}

/// A step's place in a [`Store`](dioxus::prelude::Store)'s subscription tree,
/// so a field subscribes to its own part of the form's value only.
#[doc(hidden)]
#[derive(Clone, Copy)]
pub enum StepKey {
    /// The field's position in its struct - the key `#[derive(Store)]` uses,
    /// so a write through a derived store selector reaches the field too.
    Index(u16),
    /// A whole path spelled out, from [`path!`](crate::path), which cannot
    /// know field positions. Hashed by the store.
    Name(&'static str),
}

/// Follows `steps` from `root`. `None` when a step meets a type it was not
/// built for - a path rooted at another type than the form's value.
pub(crate) fn resolve<'a>(root: &'a dyn Any, steps: &[Step]) -> Option<&'a dyn Any> {
    steps.iter().try_fold(root, |value, step| (step.get)(value))
}

pub(crate) fn resolve_mut<'a>(root: &'a mut dyn Any, steps: &[Step]) -> Option<&'a mut dyn Any> {
    let mut value = root;
    for step in steps {
        value = (step.get_mut)(value)?;
    }
    Some(value)
}

/// A field's place inside a `Root`, holding a `T`; also its posted `name`
/// (`address.zip`). Built by `#[derive(Fields)]` or [`path!`](crate::path).
pub struct FieldPath<Root, T> {
    path: Cow<'static, str>,
    steps: Cow<'static, [Step]>,
    _types: PhantomData<fn(&Root) -> &T>,
}

impl<Root, T> FieldPath<Root, T> {
    /// A path by its spelling alone. It posts and names fields, but carries no
    /// field access, so it cannot bind a field to a form's value.
    pub const fn new(path: &'static str) -> Self {
        Self::from_parts(path, &[])
    }

    /// What [`path!`](crate::path) emits.
    #[doc(hidden)]
    pub const fn from_parts(path: &'static str, steps: &'static [Step]) -> Self {
        Self {
            path: Cow::Borrowed(path),
            steps: Cow::Borrowed(steps),
            _types: PhantomData,
        }
    }

    /// `key` under this path, one step further in. What `#[derive(Fields)]`
    /// emits for each field.
    #[doc(hidden)]
    pub fn child<U>(&self, key: &str, step: Step) -> FieldPath<Root, U> {
        let mut steps = self.steps.to_vec();
        steps.push(step);
        FieldPath {
            path: Cow::Owned(join(&self.path, key)),
            steps: Cow::Owned(steps),
            _types: PhantomData,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.path
    }

    /// A path relative to `T`, re-rooted under this one: `address` + `zip`.
    pub fn join<U>(&self, inner: FieldPath<T, U>) -> FieldPath<Root, U> {
        FieldPath {
            path: Cow::Owned(join(&self.path, &inner.path)),
            steps: Cow::Owned([&*self.steps, &*inner.steps].concat()),
            _types: PhantomData,
        }
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
            steps: self.steps.clone(),
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

impl<Root, T> IntoAttributeValue for FieldPath<Root, T> {
    fn into_value(self) -> AttributeValue {
        AttributeValue::Text(self.path.into_owned())
    }
}

/// A field's `name`, typed by its value. A [`FieldPath`] also binds it inside
/// a `Form`; a plain string only posts. The root is checked at render.
pub struct FieldName<T> {
    path: Cow<'static, str>,
    steps: Option<Cow<'static, [Step]>>,
    _type: PhantomData<fn() -> T>,
}

impl<T> FieldName<T> {
    pub fn as_str(&self) -> &str {
        &self.path
    }

    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }

    /// The field access, when the name came from a path.
    pub(crate) fn steps(&self) -> Option<&[Step]> {
        self.steps.as_deref()
    }
}

impl<T> Default for FieldName<T> {
    fn default() -> Self {
        Self {
            path: Cow::Borrowed(""),
            steps: None,
            _type: PhantomData,
        }
    }
}

impl<T> Clone for FieldName<T> {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            steps: self.steps.clone(),
            _type: PhantomData,
        }
    }
}

/// By spelling: two names that post the same are the same name.
impl<T> PartialEq for FieldName<T> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.steps.is_some() == other.steps.is_some()
    }
}

impl<T> fmt::Debug for FieldName<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "FieldName({})", self.path)
    }
}

impl<T> fmt::Display for FieldName<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.path)
    }
}

impl<Root, T> From<FieldPath<Root, T>> for FieldName<T> {
    fn from(path: FieldPath<Root, T>) -> Self {
        // A spelling with no steps is `FieldPath::new`'s, which cannot bind.
        // Only the root, spelled empty, binds with no steps: the whole value.
        let bindable = path.path.is_empty() || !path.steps.is_empty();
        Self {
            steps: bindable.then_some(path.steps),
            path: path.path,
            _type: PhantomData,
        }
    }
}

impl<T> From<&str> for FieldName<T> {
    fn from(path: &str) -> Self {
        Self::from(path.to_string())
    }
}

impl<T> From<String> for FieldName<T> {
    fn from(path: String) -> Self {
        Self {
            path: Cow::Owned(path),
            steps: None,
            _type: PhantomData,
        }
    }
}

/// Implemented by `#[derive(Fields)]`: the typed paths of a struct's fields,
/// rooted at `R`.
pub trait Fields: Sized {
    type Paths<R>;

    /// The paths of `Self`'s fields, under `base`.
    fn paths<R>(base: FieldPath<R, Self>) -> Self::Paths<R>;
}

/// A typed path for a type that cannot `#[derive(Fields)]`. The field access
/// is compiled, so a typo is an error, and the path binds like a derived one:
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Fields, FieldPath};
/// # fn app() -> Element {
/// # #[derive(Clone, PartialEq, Default, Fields)] struct Address { zip: String }
/// # #[derive(Clone, PartialEq, Default, Fields)] struct Signup { #[fields(nested)] address: Address }
/// # use libero::path;
/// let zip = path!(Signup => address.zip); // FieldPath<Signup, String>
/// # let _: FieldPath<Signup, String> = zip;
/// # rsx! {}
/// # }
/// ```
#[macro_export]
macro_rules! path {
    ($root:ty => $first:ident $(. $rest:ident)*) => {{
        fn typed<R, T>(
            _: fn(&R) -> &T,
            path: &'static str,
            steps: &'static [$crate::components::Step],
        ) -> $crate::components::FieldPath<R, T> {
            $crate::components::FieldPath::from_parts(path, steps)
        }
        fn get(value: &dyn ::std::any::Any) -> ::std::option::Option<&dyn ::std::any::Any> {
            value
                .downcast_ref::<$root>()
                .map(|root| &root.$first $(.$rest)* as &dyn ::std::any::Any)
        }
        fn get_mut(
            value: &mut dyn ::std::any::Any,
        ) -> ::std::option::Option<&mut dyn ::std::any::Any> {
            value
                .downcast_mut::<$root>()
                .map(|root| &mut root.$first $(.$rest)* as &mut dyn ::std::any::Any)
        }
        const STEPS: &[$crate::components::Step] = &[$crate::components::Step {
            get,
            get_mut,
            key: $crate::components::StepKey::Name(concat!(stringify!($first) $(, ".", stringify!($rest))*)),
        }];
        typed(
            |root: &$root| &root.$first $(.$rest)*,
            concat!(stringify!($first) $(, ".", stringify!($rest))*),
            STEPS,
        )
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Address {
        zip: String,
    }
    #[derive(Default)]
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
    }

    #[test]
    fn a_path_reads_and_writes_what_it_names() {
        let mut signup = Signup::default();
        let zip = crate::path!(Signup => address.zip);

        let slot = resolve_mut(&mut signup, &zip.steps).expect("resolves");
        *slot.downcast_mut::<String>().expect("a String") = "10115".into();
        assert_eq!(signup.address.zip, "10115");

        let read = resolve(&signup, &zip.steps).and_then(|v| v.downcast_ref::<String>());
        assert_eq!(read.map(String::as_str), Some("10115"));
        assert!(signup.email.is_empty());
    }

    #[test]
    fn a_path_on_another_root_resolves_to_nothing() {
        let zip = crate::path!(Signup => address.zip);
        assert!(resolve(&Address::default(), &zip.steps).is_none());
    }

    #[test]
    fn join_reroots_a_relative_path() {
        let address = crate::path!(Signup => address);
        let zip = crate::path!(Address => zip);
        let joined = address.join(zip);
        assert_eq!(joined.as_str(), "address.zip");
        assert_eq!(joined.steps.len(), 2);
    }

    #[test]
    fn a_path_by_spelling_alone_names_but_does_not_bind() {
        let email: FieldName<String> = FieldPath::<Signup, String>::new("email").into();
        assert_eq!(email.as_str(), "email");
        assert!(email.steps().is_none());

        // The root a derive builds on is spelled empty and binds the whole value.
        let root: FieldName<Signup> = FieldPath::<Signup, Signup>::new("").into();
        assert!(root.steps().is_some_and(<[Step]>::is_empty));
    }
}
