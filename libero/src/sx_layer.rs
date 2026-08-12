#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SxLayer {
    Framework,
    UserStatic,
    UserDynamic,
}

impl SxLayer {
    pub const fn css_name(self) -> &'static str {
        match self {
            Self::Framework => "lsx-framework",
            Self::UserStatic => "lsx-user-static",
            Self::UserDynamic => "lsx-user-dynamic",
        }
    }

    pub const fn order_css() -> &'static str {
        "@layer lsx-framework, lsx-user-static, lsx-user-dynamic;"
    }
}
