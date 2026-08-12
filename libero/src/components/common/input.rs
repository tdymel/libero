use crate::sx::{StaticSx, Sx, ThemeAwareValue};

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
