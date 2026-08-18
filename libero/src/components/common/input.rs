use crate::{
    components::{ClassList, States, Variables},
    sx::{StaticSx, Sx},
    theme::Size,
};

use crate::sx::ThemeAwareValue;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Input<T: 'static> {
    #[default]
    None,
    Value(T),
    Static(&'static T),
}

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

impl From<Sx> for Input<Sx> {
    fn from(value: Sx) -> Self {
        Self::Value(value)
    }
}

impl From<Option<Sx>> for Input<Sx> {
    fn from(value: Option<Sx>) -> Self {
        match value {
            Some(value) => Self::Value(value),
            None => Self::None,
        }
    }
}

impl From<&'static StaticSx> for Input<Sx> {
    fn from(value: &'static StaticSx) -> Self {
        Self::Static(value)
    }
}

impl From<ClassList> for Input<ClassList> {
    fn from(value: ClassList) -> Self {
        Self::Value(value)
    }
}

impl From<Option<ClassList>> for Input<ClassList> {
    fn from(value: Option<ClassList>) -> Self {
        match value {
            Some(value) => Self::Value(value),
            None => Self::None,
        }
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

impl From<&str> for Input<ClassList> {
    fn from(value: &str) -> Self {
        Self::Value(ClassList::from(value))
    }
}

impl From<String> for Input<ClassList> {
    fn from(value: String) -> Self {
        Self::Value(ClassList::from(value))
    }
}

impl From<Size> for Input<Size> {
    fn from(value: Size) -> Self {
        Self::Value(value)
    }
}

impl From<&str> for Input<Size> {
    fn from(value: &str) -> Self {
        Self::Value(Size::from(value))
    }
}

impl From<String> for Input<Size> {
    fn from(value: String) -> Self {
        Self::Value(Size::from(value))
    }
}

impl From<f32> for Input<f32> {
    fn from(value: f32) -> Self {
        Self::Value(value)
    }
}

impl From<f64> for Input<f64> {
    fn from(value: f64) -> Self {
        Self::Value(value)
    }
}

impl From<States> for Input<States> {
    fn from(value: States) -> Self {
        Self::Value(value)
    }
}

impl From<Option<States>> for Input<States> {
    fn from(value: Option<States>) -> Self {
        match value {
            Some(value) => Self::Value(value),
            None => Self::None,
        }
    }
}

impl From<Variables> for Input<Variables> {
    fn from(value: Variables) -> Self {
        Self::Value(value)
    }
}

impl From<Option<Variables>> for Input<Variables> {
    fn from(value: Option<Variables>) -> Self {
        match value {
            Some(value) => Self::Value(value),
            None => Self::None,
        }
    }
}

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
