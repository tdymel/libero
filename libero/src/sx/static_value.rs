use crate::theme::SPACING_CSS_VAR;
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

fn uses_spacing_scale(prop_name: &str) -> bool {
    matches!(prop_name, "gap")
}

impl StaticValue {
    pub fn fmt_for_prop(&self, prop_name: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) if uses_spacing_scale(prop_name) => {
                write!(f, "calc({} * var({}))", value, SPACING_CSS_VAR)
            }
            Self::Integer(value) => write!(f, "{}px", value),
            Self::Text(value) => write!(f, "{}", value),
        }
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
