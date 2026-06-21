use crate::{sx::SxContext, theme::Theme};

#[derive(Clone, Default)]
pub struct LiberoContext {
    pub sx: SxContext,
    pub theme: Theme,
}
