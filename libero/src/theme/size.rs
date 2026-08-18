#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Size {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    Xxl,
}

impl Size {
    pub(crate) const ALL: [Size; 6] = [Self::Xs, Self::Sm, Self::Md, Self::Lg, Self::Xl, Self::Xxl];

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
            Self::Xxl => "xxl",
        }
    }

    pub fn parse_dynamic(value: &str) -> Option<Self> {
        match value {
            "xs" => Some(Self::Xs),
            "sm" => Some(Self::Sm),
            "md" => Some(Self::Md),
            "lg" => Some(Self::Lg),
            "xl" => Some(Self::Xl),
            "xxl" => Some(Self::Xxl),
            _ => None,
        }
    }

    pub const fn breakpoint_value(&self) -> &'static str {
        match self {
            Self::Xs => "36rem",
            Self::Sm => "48rem",
            Self::Md => "62rem",
            Self::Lg => "75rem",
            Self::Xl => "88rem",
            Self::Xxl => "101rem",
        }
    }

    /// The `data-state` token representing this size - `"size-md"` etc.
    /// For components that gate size-specific CSS behind
    /// `[data-state~="..."]` instead of generating a new class per size.
    pub const fn state_name(&self) -> &'static str {
        match self {
            Self::Xs => "size-xs",
            Self::Sm => "size-sm",
            Self::Md => "size-md",
            Self::Lg => "size-lg",
            Self::Xl => "size-xl",
            Self::Xxl => "size-xxl",
        }
    }

    /// Same as [`state_name`](Self::state_name), but for a `radius` prop
    /// that scales independently of a component's own `size` (e.g.
    /// `Select`'s `radius` defaults to matching `size` but can be
    /// overridden separately) - a distinct token namespace so both can be
    /// present on the same element without colliding.
    pub const fn radius_state_name(&self) -> &'static str {
        match self {
            Self::Xs => "radius-xs",
            Self::Sm => "radius-sm",
            Self::Md => "radius-md",
            Self::Lg => "radius-lg",
            Self::Xl => "radius-xl",
            Self::Xxl => "radius-xxl",
        }
    }
}

impl From<&str> for Size {
    fn from(value: &str) -> Self {
        Self::parse_dynamic(value).unwrap_or_else(|| {
            debug_assert!(false, "unknown size `{value}`, falling back to md");
            Self::Md
        })
    }
}

impl From<String> for Size {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}
