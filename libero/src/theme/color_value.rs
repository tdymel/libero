use crate::common::ConstStr;

use super::{Color, ColorShade};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorValue {
    Shade(Color, ColorShade),
    Contrast(Color),
}

impl ColorValue {
    pub const fn parse(value: &'static str) -> Option<Self> {
        if starts_with(value, "primary") {
            return Some(Self::Shade(Color::Primary, parse_shade(value, 7)));
        }

        if starts_with(value, "secondary") {
            return Some(Self::Shade(Color::Secondary, parse_shade(value, 9)));
        }

        None
    }

    pub const fn push_var_name(self, mut css: ConstStr) -> ConstStr {
        css = css.push_str("--lsx-");
        match self {
            Self::Shade(color, shade) => {
                css = css.push_str(color.as_str());
                css = css.push_char('-');
                css = css.push_str(shade.as_hundreds_str());
            }
            Self::Contrast(color) => {
                css = css.push_str(color.as_str());
                css = css.push_str("-contrast");
            }
        }
        css
    }

    pub const fn push_css_var(self, mut css: ConstStr) -> ConstStr {
        css = css.push_str("var(");
        css = self.push_var_name(css);
        css.push_char(')')
    }
}

const fn starts_with(value: &str, prefix: &str) -> bool {
    let value = value.as_bytes();
    let prefix = prefix.as_bytes();

    if prefix.len() > value.len() {
        return false;
    }

    let mut i = 0;
    while i < prefix.len() {
        if value[i] != prefix[i] {
            return false;
        }
        i += 1;
    }

    true
}

const fn parse_shade(value: &'static str, palette_len: usize) -> ColorShade {
    let bytes = value.as_bytes();
    if bytes.len() == palette_len {
        return ColorShade::S5;
    }

    if bytes.len() == palette_len + 2 && bytes[palette_len] == b'.' {
        let shade = bytes[palette_len + 1];
        return match shade {
            b'1' => ColorShade::S1,
            b'2' => ColorShade::S2,
            b'3' => ColorShade::S3,
            b'4' => ColorShade::S4,
            b'5' => ColorShade::S5,
            b'6' => ColorShade::S6,
            b'7' => ColorShade::S7,
            b'8' => ColorShade::S8,
            b'9' => ColorShade::S9,
            _ => panic!("invalid palette token"),
        };
    }

    panic!("invalid palette token")
}

#[cfg(test)]
mod tests {
    use crate::common::ConstStr;

    use super::*;

    #[test]
    fn color_value_happy_path() {
        const PRIMARY: ConstStr =
            ColorValue::Shade(Color::Primary, ColorShade::S1).push_css_var(ConstStr::new());
        const SECONDARY_CONTRAST: ConstStr =
            ColorValue::Contrast(Color::Secondary).push_css_var(ConstStr::new());
        const PRIMARY_VAR: ConstStr =
            ColorValue::Shade(Color::Primary, ColorShade::S1).push_var_name(ConstStr::new());

        assert_eq!(PRIMARY.as_str(), "var(--lsx-primary-100)");
        assert_eq!(SECONDARY_CONTRAST.as_str(), "var(--lsx-secondary-contrast)");
        assert_eq!(PRIMARY_VAR.as_str(), "--lsx-primary-100");
    }
}
