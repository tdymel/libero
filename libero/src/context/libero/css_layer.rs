#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CssLayer {
    /// The theme's reset and its `body` rules. First, so an app's own base
    /// styles - Tailwind v4's `@layer base`, any `@layer reset` - override
    /// them without `!important`. Only the theme sheet
    /// (`Stylesheet::from(&Theme)`) writes here; nothing registers into it.
    Base,
    Framework,
    UserStatic,
    UserCustom,
}

impl CssLayer {
    pub const fn css_name(self) -> &'static str {
        match self {
            Self::Base => "lsx-base",
            Self::Framework => "lsx-framework",
            Self::UserStatic => "lsx-user-static",
            Self::UserCustom => "lsx-user-custom",
        }
    }

    pub const fn order_css() -> &'static str {
        "@layer lsx-base, lsx-framework, lsx-user-static, lsx-user-custom;"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_layer_order_lists_every_layer_by_its_own_name() {
        let expected = format!(
            "@layer {}, {}, {}, {};",
            CssLayer::Base.css_name(),
            CssLayer::Framework.css_name(),
            CssLayer::UserStatic.css_name(),
            CssLayer::UserCustom.css_name(),
        );

        assert_eq!(CssLayer::order_css(), expected);
    }
}
