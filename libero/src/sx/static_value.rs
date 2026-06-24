use crate::{
    Color, Size, ThemeColor,
    theme::{BORDER_RADIUS_CSS_VAR, FONT_SIZE_CSS_VAR, SPACING_CSS_VAR},
};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StaticValue {
    Integer(i64),
    Size(Size),
    Color(ThemeColor),
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
        // default to shade 6 (matching current behavior)
        StaticValue::Color(self.main())
    }
}

impl const IntoStaticValue for &'static str {
    fn into_static_value(self) -> StaticValue {
        match self {
            "primary.0" => StaticValue::Color(Color::Primary.shade(0)),
            "primary.1" => StaticValue::Color(Color::Primary.shade(1)),
            "primary.2" => StaticValue::Color(Color::Primary.shade(2)),
            "primary.3" => StaticValue::Color(Color::Primary.shade(3)),
            "primary.4" => StaticValue::Color(Color::Primary.shade(4)),
            "primary.5" => StaticValue::Color(Color::Primary.shade(5)),
            "primary.6" => StaticValue::Color(Color::Primary.shade(6)),
            "primary.7" => StaticValue::Color(Color::Primary.shade(7)),
            "primary.8" => StaticValue::Color(Color::Primary.shade(8)),
            "primary.9" => StaticValue::Color(Color::Primary.shade(9)),
            "secondary.0" => StaticValue::Color(Color::Secondary.shade(0)),
            "secondary.1" => StaticValue::Color(Color::Secondary.shade(1)),
            "secondary.2" => StaticValue::Color(Color::Secondary.shade(2)),
            "secondary.3" => StaticValue::Color(Color::Secondary.shade(3)),
            "secondary.4" => StaticValue::Color(Color::Secondary.shade(4)),
            "secondary.5" => StaticValue::Color(Color::Secondary.shade(5)),
            "secondary.6" => StaticValue::Color(Color::Secondary.shade(6)),
            "secondary.7" => StaticValue::Color(Color::Secondary.shade(7)),
            "secondary.8" => StaticValue::Color(Color::Secondary.shade(8)),
            "secondary.9" => StaticValue::Color(Color::Secondary.shade(9)),
            "xs" => StaticValue::Size(Size::Xs),
            "sm" => StaticValue::Size(Size::Sm),
            "md" => StaticValue::Size(Size::Md),
            "lg" => StaticValue::Size(Size::Lg),
            "xl" => StaticValue::Size(Size::Xl),
            "primary" => StaticValue::Color(Color::Primary.main()),
            "secondary" => StaticValue::Color(Color::Secondary.main()),
            _ => StaticValue::Text(self),
        }
    }
}

impl const IntoStaticValue for ThemeColor {
    fn into_static_value(self) -> StaticValue {
        StaticValue::Color(self)
    }
}

fn uses_spacing_scale(prop_name: &str) -> bool {
    matches!(
        prop_name,
        "gap"
            | "padding"
            | "padding-top"
            | "padding-right"
            | "padding-bottom"
            | "padding-left"
            | "margin"
            | "margin-top"
            | "margin-right"
            | "margin-bottom"
            | "margin-left"
    )
}

fn uses_border_radius_scale(prop_name: &str) -> bool {
    matches!(
        prop_name,
        "border-radius"
            | "border-top-left-radius"
            | "border-top-right-radius"
            | "border-bottom-right-radius"
            | "border-bottom-left-radius"
    )
}

fn size_css_value(prop_name: &str, size: Size) -> String {
    let css_var = match prop_name {
        "font-size" => FONT_SIZE_CSS_VAR,
        _ if uses_border_radius_scale(prop_name) => BORDER_RADIUS_CSS_VAR,
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

fn color_css_value(color: ThemeColor) -> String {
    color.to_string()
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
            Self::Color(color) => write!(f, "{}", color),
            Self::Text(value) => write!(f, "{}", value),
        }
    }
}
