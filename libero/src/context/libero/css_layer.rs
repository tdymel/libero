#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CssLayer {
    Framework,
    UserStatic,
    UserCustom,
}

impl CssLayer {
    pub const fn css_name(self) -> &'static str {
        match self {
            Self::Framework => "lsx-framework",
            Self::UserStatic => "lsx-user-static",
            Self::UserCustom => "lsx-user-custom",
        }
    }

    pub const fn order_css() -> &'static str {
        "@layer lsx-framework, lsx-user-static, lsx-user-custom;"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_layer_order_lists_every_layer_by_its_own_name() {
        let expected = format!(
            "@layer {}, {}, {};",
            CssLayer::Framework.css_name(),
            CssLayer::UserStatic.css_name(),
            CssLayer::UserCustom.css_name(),
        );

        assert_eq!(CssLayer::order_css(), expected);
    }
}
