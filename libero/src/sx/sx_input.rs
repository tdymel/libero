use super::{StaticSx, Sx};

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SxInput {
    #[default]
    None,
    Owned(Sx),
    Static(&'static StaticSx),
}

impl SxInput {
    pub fn as_sx(&self) -> Option<&Sx> {
        match self {
            Self::None => None,
            Self::Owned(sx) => Some(sx),
            Self::Static(sx) => Some(sx),
        }
    }
}

impl From<Sx> for SxInput {
    fn from(value: Sx) -> Self {
        Self::Owned(value)
    }
}

impl From<&'static StaticSx> for SxInput {
    fn from(value: &'static StaticSx) -> Self {
        Self::Static(value)
    }
}
