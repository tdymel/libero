use std::{fmt, ops::Deref, rc::Rc};

/// What a `Form` or `Fieldset` can hold as its value.
pub trait FormValue: Clone + PartialEq + Default + 'static {}

impl<T: Clone + PartialEq + Default + 'static> FormValue for T {}

/// A value with neither `Clone` nor `PartialEq`, such as a secret, made fit for a form: clones
/// share it and two compare equal only when they are the same one.
///
/// ```
/// use libero::components::{ByAddress, Fields};
///
/// struct Password(String); // no Clone, no PartialEq
///
/// #[derive(Clone, PartialEq, Default, Fields)]
/// struct Login { password: Option<ByAddress<Password>> }
///
/// let password = ByAddress::new(Password("hunter2".into()));
/// assert!(password == password.clone());
/// assert!(password != ByAddress::new(Password("hunter2".into())));
/// let Password(text) = &*password;
/// assert_eq!(text, "hunter2");
/// ```
pub struct ByAddress<T>(Rc<T>);

impl<T> ByAddress<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(value))
    }

    /// The value back when `this` is its only holder, or `this` while a clone is alive.
    /// Associated, as `Rc::try_unwrap` is: a method would hide behind the value's own.
    pub fn try_unwrap(this: Self) -> Result<T, Self> {
        Rc::try_unwrap(this.0).map_err(Self)
    }
}

impl<T> Deref for ByAddress<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> Clone for ByAddress<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> PartialEq for ByAddress<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> From<T> for ByAddress<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: Default> Default for ByAddress<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: fmt::Debug> fmt::Debug for ByAddress<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Secret(&'static str);

    /// Todo 4114: the value comes back once no clone holds it.
    #[test]
    fn try_unwrap_returns_the_value_when_unshared() {
        let secret = ByAddress::new(Secret("hunter2"));
        let Ok(Secret(text)) = ByAddress::try_unwrap(secret) else {
            panic!("an unshared value unwraps");
        };
        assert_eq!(text, "hunter2");
    }

    #[test]
    fn try_unwrap_hands_the_holder_back_while_a_clone_lives() {
        let secret = ByAddress::new(Secret("hunter2"));
        let clone = secret.clone();
        let Err(secret) = ByAddress::try_unwrap(secret) else {
            panic!("a shared value stays held");
        };
        assert!(secret == clone);
        drop(clone);
        assert!(ByAddress::try_unwrap(secret).is_ok());
    }
}
