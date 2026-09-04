#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CssLayer {
    /// Rendered into `lsx-framework`, but sorted ahead of every other sheet
    /// in it: `Box`'s default focus ring. A component's own `:focus-visible`
    /// has the same specificity, so it wins only by coming later in the
    /// layer. Without this the order inside the layer was the CSS hash's.
    ///
    /// Not a cascade layer of its own: across layers specificity stops
    /// counting, so a component's plain `outline: none` would erase the ring.
    FrameworkDefault,
    Framework,
    UserStatic,
    UserCustom,
}

impl CssLayer {
    pub const fn css_name(self) -> &'static str {
        match self {
            Self::FrameworkDefault | Self::Framework => "lsx-framework",
            Self::UserStatic => "lsx-user-static",
            Self::UserCustom => "lsx-user-custom",
        }
    }

    /// Prefix of a registry entry's node key. Unlike [`Self::css_name`] it
    /// tells the two framework ranks apart, so the same CSS on both is two
    /// keys, not one key twice.
    pub const fn key_name(self) -> &'static str {
        match self {
            Self::FrameworkDefault => "lsx-framework-default",
            layer => layer.css_name(),
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
