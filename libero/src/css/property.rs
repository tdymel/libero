use crate::{
    common::ConstStr,
    sx::{DeclarationProperty, Property},
};

impl DeclarationProperty {
    pub(crate) const fn push_name<const MAX_SIZE: usize>(
        self,
        css: ConstStr<MAX_SIZE>,
    ) -> ConstStr<MAX_SIZE> {
        match self {
            Self::Known(property) => css.push_str(property.as_str()),
            Self::Raw(property) => css.push_str(property),
        }
    }
}

impl Property {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Background => "background",
            Self::Width => "width",
            Self::Height => "height",
            Self::PaddingTop => "padding-top",
        }
    }
}
