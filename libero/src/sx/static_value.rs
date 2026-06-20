use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StaticValue {
    Integer(i64),
    Text(&'static str),
}

pub const trait IntoStaticValue {
    fn into_static_value(self) -> StaticValue;
}

impl const IntoStaticValue for StaticValue {
    fn into_static_value(self) -> StaticValue {
        self
    }
}

impl const IntoStaticValue for i64 {
    fn into_static_value(self) -> StaticValue {
        StaticValue::Integer(self)
    }
}

impl const IntoStaticValue for &'static str {
    fn into_static_value(self) -> StaticValue {
        StaticValue::Text(self)
    }
}

impl fmt::Display for StaticValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(f, "{}", value),
            Self::Text(value) => write!(f, "{}", value),
        }
    }
}
