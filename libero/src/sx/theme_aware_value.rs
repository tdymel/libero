use crate::theme::{ColorValue, CssVar, Size};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ThemeAwareValue {
    String(String),
    Number(String),
    Size(Size),
    ColorValue(ColorValue),
    CssVar(CssVar),
}

impl From<String> for ThemeAwareValue {
    fn from(value: String) -> Self {
        if let Some(size) = Size::parse_dynamic(value.as_str()) {
            return Self::Size(size);
        }

        if let Some(color) = ColorValue::parse(value.as_str()) {
            return Self::ColorValue(color);
        }

        if let Some(css_var) = CssVar::parse(value.as_str()) {
            return Self::CssVar(css_var);
        }

        Self::String(value)
    }
}

impl From<&str> for ThemeAwareValue {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}

impl From<Size> for ThemeAwareValue {
    fn from(value: Size) -> Self {
        Self::Size(value)
    }
}

impl From<ColorValue> for ThemeAwareValue {
    fn from(value: ColorValue) -> Self {
        Self::ColorValue(value)
    }
}

impl From<CssVar> for ThemeAwareValue {
    fn from(value: CssVar) -> Self {
        Self::CssVar(value)
    }
}

macro_rules! impl_int_into_theme_aware_value {
    ($($t:ty),* $(,)?) => {
        $(
            impl From<$t> for ThemeAwareValue {
                fn from(value: $t) -> Self {
                    Self::Number(value.to_string())
                }
            }
        )*
    };
}

impl_int_into_theme_aware_value!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

impl From<f32> for ThemeAwareValue {
    fn from(value: f32) -> Self {
        Self::Number(value.to_string())
    }
}

impl From<f64> for ThemeAwareValue {
    fn from(value: f64) -> Self {
        Self::Number(value.to_string())
    }
}

impl From<bool> for ThemeAwareValue {
    fn from(value: bool) -> Self {
        if value {
            Self::String("wrap".to_string())
        } else {
            Self::String("nowrap".to_string())
        }
    }
}
