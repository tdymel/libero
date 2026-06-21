use crate::{
    Color, Size,
    theme::{FONT_SIZE_CSS_VAR, PRIMARY_COLOR_CSS_VAR, SECONDARY_COLOR_CSS_VAR, SPACING_CSS_VAR},
};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StaticValue {
    Integer(i64),
    Size(Size),
    Color(Color),
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

impl const IntoStaticValue for Size {
    fn into_static_value(self) -> StaticValue {
        StaticValue::Size(self)
    }
}

impl const IntoStaticValue for Color {
    fn into_static_value(self) -> StaticValue {
        StaticValue::Color(self)
    }
}

impl const IntoStaticValue for &'static str {
    fn into_static_value(self) -> StaticValue {
        match self {
            "xs" => StaticValue::Size(Size::Xs),
            "sm" => StaticValue::Size(Size::Sm),
            "md" => StaticValue::Size(Size::Md),
            "lg" => StaticValue::Size(Size::Lg),
            "xl" => StaticValue::Size(Size::Xl),
            "primary" => StaticValue::Color(Color::Primary),
            "secondary" => StaticValue::Color(Color::Secondary),
            _ => StaticValue::Text(self),
        }
    }
}

fn uses_spacing_scale(prop_name: &str) -> bool {
    matches!(prop_name, "gap")
}

fn size_css_value(prop_name: &str, size: Size) -> String {
    let css_var = match prop_name {
        "font-size" => FONT_SIZE_CSS_VAR,
        _ => SPACING_CSS_VAR,
    };

    match size {
        Size::Xs => format!("var({}-xs)", css_var),
        Size::Sm => format!("var({}-sm)", css_var),
        Size::Md => format!("var({}-md)", css_var),
        Size::Lg => format!("var({}-lg)", css_var),
        Size::Xl => format!("var({}-xl)", css_var),
    }
}

fn color_css_value(color: Color) -> String {
    match color {
        Color::Primary => format!("var({}-6)", PRIMARY_COLOR_CSS_VAR),
        Color::Secondary => format!("var({}-6)", SECONDARY_COLOR_CSS_VAR),
    }
}

impl StaticValue {
    pub fn fmt_for_prop(&self, prop_name: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) if uses_spacing_scale(prop_name) => {
                write!(f, "calc({} * var({}))", value, SPACING_CSS_VAR)
            }
            Self::Integer(value) => write!(f, "{}px", value),
            Self::Size(size) => write!(f, "{}", size_css_value(prop_name, *size)),
            Self::Color(color) => write!(f, "{}", color_css_value(*color)),
            Self::Text(value) => write!(f, "{}", value),
        }
    }
}

impl fmt::Display for StaticValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(f, "{}", value),
            Self::Size(size) => write!(f, "{:?}", size),
            Self::Color(color) => write!(f, "{:?}", color),
            Self::Text(value) => write!(f, "{}", value),
        }
    }
}
