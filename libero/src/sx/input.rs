use crate::{
    sx::{ClassList, States, StaticSx, Sx, ThemeAwareValue, Variables},
    tokens::{Size, SizeCss},
};

/// A styling prop's value: unset, owned, or a `static` (compared by address first).
#[derive(Clone, Debug, Default)]
pub enum Input<T: 'static> {
    #[default]
    None,
    Value(T),
    Static(&'static T),
}

/// Hand-written only for the `Static` fast path; otherwise as the derive.
impl<T: PartialEq + 'static> PartialEq for Input<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Value(a), Self::Value(b)) => a == b,
            // An unchanged render passes the same static: skip the deep `Sx` compare.
            (Self::Static(a), Self::Static(b)) => std::ptr::eq(*a, *b) || a == b,
            _ => false,
        }
    }
}

impl<T: Eq + 'static> Eq for Input<T> {}

impl<T: 'static> Input<T> {
    pub fn as_ref(&self) -> Option<&T> {
        match self {
            Self::None => None,
            Self::Value(value) => Some(value),
            Self::Static(value) => Some(value),
        }
    }

    pub fn into_option(self) -> Option<T>
    where
        T: Clone,
    {
        match self {
            Self::None => None,
            Self::Value(value) => Some(value),
            Self::Static(value) => Some(value.clone()),
        }
    }

    /// The value if set, else `default`.
    pub fn copied_or(&self, default: T) -> T
    where
        T: Copy,
    {
        self.as_ref().copied().unwrap_or(default)
    }

    pub fn copied_or_default(&self) -> T
    where
        T: Copy + Default,
    {
        self.copied_or(T::default())
    }

    pub fn unwrap_or_default(self) -> T
    where
        T: Default + Clone,
    {
        match self {
            Self::None => T::default(),
            Self::Value(value) => value,
            Self::Static(value) => value.clone(),
        }
    }
}

impl Input<ThemeAwareValue> {
    /// Resolves to a CSS value, reading a bare `Size` against `scale`.
    pub(crate) fn resolve(&self, scale: Option<SizeCss>) -> Option<String> {
        self.as_ref().and_then(|value| value.resolve(scale))
    }
}

/// `&str`/`String`/`T`/`Option<T>` -> `Input<T>` for a `str_enum!` type -
/// `#[props(into)]` can't chain that on its own.
macro_rules! input_from_str {
    ($ty:ty) => {
        $crate::sx::input::input_from!($ty);

        impl From<&str> for $crate::sx::Input<$ty> {
            fn from(value: &str) -> Self {
                Self::Value(<$ty>::from(value))
            }
        }

        impl From<String> for $crate::sx::Input<$ty> {
            fn from(value: String) -> Self {
                Self::Value(<$ty>::from(value.as_str()))
            }
        }
    };
}

pub(crate) use input_from_str;

/// `T`/`Option<T>` -> `Input<T>`, which `#[props(into)]` can't chain.
macro_rules! input_from {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl From<$ty> for $crate::sx::Input<$ty> {
                fn from(value: $ty) -> Self {
                    Self::Value(value)
                }
            }

            impl From<Option<$ty>> for $crate::sx::Input<$ty> {
                fn from(value: Option<$ty>) -> Self {
                    match value {
                        Some(value) => Self::Value(value),
                        None => Self::None,
                    }
                }
            }
        )+
    };
}

pub(crate) use input_from;

input_from!(Sx, f32, f64, usize, States, Variables);

impl From<&'static StaticSx> for Input<Sx> {
    fn from(value: &'static StaticSx) -> Self {
        Self::Static(value)
    }
}

impl From<Option<String>> for Input<ClassList> {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(value) => Self::Value(ClassList::from(value)),
            None => Self::None,
        }
    }
}

input_from_str!(ClassList);

input_from_str!(Size);

impl<T> From<T> for Input<ThemeAwareValue>
where
    T: Into<ThemeAwareValue>,
{
    fn from(value: T) -> Self {
        Self::Value(value.into())
    }
}

impl<T> From<Option<T>> for Input<ThemeAwareValue>
where
    T: Into<ThemeAwareValue>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => Self::Value(value.into()),
            None => Self::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sx::{StaticSx, sx};

    static PADDING: StaticSx = StaticSx::new(|| sx().padding("lg"));
    static SAME_PADDING: StaticSx = StaticSx::new(|| sx().padding("lg"));

    #[test]
    fn the_same_static_is_equal_to_itself() {
        assert_eq!(Input::Static(&*PADDING), Input::Static(&*PADDING));
    }

    /// Distinct statics still fall through to the content compare.
    #[test]
    fn two_statics_with_equal_content_are_equal() {
        assert_eq!(Input::Static(&*PADDING), Input::Static(&*SAME_PADDING));
    }

    /// Derive parity: a different variant is never equal.
    #[test]
    fn a_static_never_equals_an_owned_value() {
        assert_ne!(Input::Static(&*PADDING), Input::Value(sx().padding("lg")));
    }

    #[test]
    fn owned_values_compare_by_content() {
        assert_eq!(
            Input::Value(sx().padding("lg")),
            Input::Value(sx().padding("lg"))
        );
        assert_ne!(
            Input::Value(sx().padding("lg")),
            Input::Value(sx().color("red"))
        );
    }

    #[test]
    fn none_equals_only_none() {
        assert_eq!(Input::<Sx>::None, Input::<Sx>::None);
        assert_ne!(Input::None, Input::Value(sx().padding("lg")));
    }
}
