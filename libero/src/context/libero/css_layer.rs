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
