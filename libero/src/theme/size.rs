use crate::common::ConstStr;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
}

impl Size {
    pub const fn parse(value: &'static str) -> Option<Self> {
        match value.as_bytes() {
            b"xs" => Some(Self::Xs),
            b"sm" => Some(Self::Sm),
            b"md" => Some(Self::Md),
            b"lg" => Some(Self::Lg),
            b"xl" => Some(Self::Xl),
            _ => None,
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
        }
    }

    pub const fn breakpoint_value(&self) -> &'static str {
        match self {
            Self::Xs => "36em",
            Self::Sm => "48em",
            Self::Md => "62em",
            Self::Lg => "75em",
            Self::Xl => "88em",
        }
    }

    pub(crate) const fn push_var_name(self, mut css: ConstStr, prefix: &'static str) -> ConstStr {
        css = css.push_str(prefix);
        css = css.push_str(self.as_str());
        css
    }

    pub(crate) const fn push_css_var(self, mut css: ConstStr, prefix: &'static str) -> ConstStr {
        css = css.push_str("var(");
        css = self.push_var_name(css, prefix);
        css = css.push_char(')');
        css
    }
}
